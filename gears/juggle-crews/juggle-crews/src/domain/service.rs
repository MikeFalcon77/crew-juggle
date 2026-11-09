//! Crew use cases: sign-in, `/me`, create, join, invite codes, demo seeding.
//!
//! Every storage call names its scope. Three reads look across crews, and each
//! sits in one helper below that says why: which crew an account belongs to,
//! which crew owns an invite code, and the list of crews for the worker.
//! Everything else is bounded to one crew with `AccessScope::for_tenant`.
use super::error::{DomainError, Result};
use super::invite::{display_code, new_code, normalize_code};
use super::model::{Crew, CrewSummary, Me, Member, MemberSummary};
use super::repo::CrewRepo;
use chrono::Utc;
use juggle_crews_sdk::{
    AccountIdentity, AccountMembership, MembershipKind, Role, demo, lobby_tenant, subject_for,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use toolkit::domain_model;
use toolkit_security::{AccessScope, SecurityContext};
use uuid::Uuid;

/// How many signed-in identities the server keeps for `/me`.
const MAX_REMEMBERED: usize = 10_000;
/// How many accounts one crew may have.
const MAX_MEMBERS: usize = 20;

/// The crews use cases.
#[domain_model]
pub struct CrewsService {
    repo: Arc<dyn CrewRepo>,
    /// Subject id → identity, filled at sign-in. A `SecurityContext` carries
    /// only the subject id, and `/me` needs the email and name.
    identities: RwLock<HashMap<Uuid, AccountIdentity>>,
}

fn clean_name(value: &str, what: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 80 {
        return Err(DomainError::Invalid(format!(
            "{what} must be 1–80 characters"
        )));
    }
    Ok(value.into())
}

fn clean_zone(value: &str) -> Result<String> {
    value
        .trim()
        .parse::<chrono_tz::Tz>()
        .map(|z| z.to_string())
        .map_err(|_| DomainError::Invalid("Pick a time zone from the list".into()))
}

fn member(identity: &AccountIdentity, crew: Uuid, display_name: String, role: Role) -> Member {
    Member {
        uid: identity.uid.clone(),
        subject_id: subject_for(&identity.uid),
        crew_id: crew,
        display_name,
        email: identity.email.clone(),
        role,
        joined_at: Utc::now(),
    }
}

impl CrewsService {
    /// A service over the given storage.
    pub fn new(repo: Arc<dyn CrewRepo>) -> Self {
        Self {
            repo,
            identities: RwLock::new(HashMap::new()),
        }
    }

    // ---- the three reads across crews ----

    /// Which crew an account belongs to. Sign-in asks before any crew is known.
    async fn membership_of(&self, uid: &str) -> Result<Option<Member>> {
        self.repo
            .member_by_uid(&AccessScope::allow_all(), uid)
            .await
    }

    /// Which crew owns an invite code. Joining is the moment an account
    /// crosses into a crew it does not belong to yet.
    async fn crew_by_code(&self, code: &str) -> Result<Option<Crew>> {
        self.repo
            .crew_by_code(&AccessScope::allow_all(), code)
            .await
    }

    /// Every crew, for the internal worker that walks them one by one.
    pub async fn all_crew_ids(&self) -> Result<Vec<Uuid>> {
        self.repo.crew_ids(&AccessScope::allow_all()).await
    }

    // ---- sign-in ----

    /// Records a verified account and says where it lands: its crew, or a
    /// lobby tenant until it creates or joins one.
    pub async fn sign_in(&self, identity: AccountIdentity) -> Result<AccountMembership> {
        let membership = self.membership_of(&identity.uid).await?;
        let (tenant_id, kind) = match membership {
            Some(member) => (member.crew_id, MembershipKind::Member),
            None => (lobby_tenant(&identity.uid), MembershipKind::Guest),
        };
        let subject_id = subject_for(&identity.uid);
        self.remember(subject_id, identity);
        Ok(AccountMembership {
            subject_id,
            tenant_id,
            kind,
        })
    }

    fn remember(&self, subject: Uuid, identity: AccountIdentity) {
        let mut map = self.identities.write().expect("identity cache poisoned");
        if map.len() >= MAX_REMEMBERED {
            map.clear();
        }
        map.insert(subject, identity);
    }

    fn identity(&self, ctx: &SecurityContext) -> Result<AccountIdentity> {
        self.identities
            .read()
            .expect("identity cache poisoned")
            .get(&ctx.subject_id())
            .cloned()
            .ok_or(DomainError::Unauthorized)
    }

    // ---- one crew ----

    /// The crew with this id, or `None`.
    pub async fn crew(&self, crew: Uuid) -> Result<Option<Crew>> {
        self.repo.crew(&AccessScope::for_tenant(crew)).await
    }

    /// Members of one crew, in the order they joined.
    pub async fn members(&self, crew: Uuid) -> Result<Vec<Member>> {
        self.repo.members(&AccessScope::for_tenant(crew)).await
    }

    /// Whether `subject` belongs to `crew`. The nil crew has no members.
    pub async fn is_member(&self, subject: Uuid, crew: Uuid) -> Result<bool> {
        if crew.is_nil() {
            return Ok(false);
        }
        self.repo
            .is_member(&AccessScope::for_tenant(crew), subject)
            .await
    }

    async fn view(&self, identity: AccountIdentity) -> Result<Me> {
        let crew = match self.membership_of(&identity.uid).await? {
            None => None,
            Some(member) => {
                let crew = self
                    .crew(member.crew_id)
                    .await?
                    .ok_or(DomainError::NotFound)?;
                let members = self.members(crew.id).await?;
                Some(CrewSummary {
                    id: crew.id,
                    name: crew.name,
                    timezone: crew.timezone,
                    invite_code: display_code(&crew.invite_code),
                    role: member.role,
                    members: members
                        .into_iter()
                        .map(|m| MemberSummary {
                            uid: m.uid,
                            display_name: m.display_name,
                            role: m.role,
                        })
                        .collect(),
                })
            }
        };
        let name = identity
            .name
            .clone()
            .unwrap_or_else(|| identity.email.split('@').next().unwrap_or("").into());
        Ok(Me {
            uid: identity.uid,
            email: identity.email,
            name,
            crew,
        })
    }

    /// Who the caller is and their crew.
    pub async fn me(&self, ctx: &SecurityContext) -> Result<Me> {
        self.view(self.identity(ctx)?).await
    }

    async fn ensure_free(&self, identity: &AccountIdentity) -> Result<()> {
        if self.membership_of(&identity.uid).await?.is_some() {
            return Err(DomainError::Conflict("You are already in a crew"));
        }
        Ok(())
    }

    /// Creates a crew with the caller as its owner.
    pub async fn create(
        &self,
        ctx: &SecurityContext,
        name: &str,
        display_name: &str,
        timezone: &str,
    ) -> Result<Me> {
        let identity = self.identity(ctx)?;
        let (name, display_name, timezone) = (
            clean_name(name, "Crew name")?,
            clean_name(display_name, "Your name")?,
            clean_zone(timezone)?,
        );
        self.ensure_free(&identity).await?;
        let crew = Crew {
            id: Uuid::new_v4(),
            name,
            timezone,
            invite_code: new_code()?,
            created_at: Utc::now(),
        };
        let owner = member(&identity, crew.id, display_name, Role::Owner);
        self.repo
            .create_crew(&AccessScope::for_tenant(crew.id), crew, owner)
            .await?;
        self.view(identity).await
    }

    /// Joins the crew that owns `code`.
    pub async fn join(&self, ctx: &SecurityContext, code: &str, display_name: &str) -> Result<Me> {
        let identity = self.identity(ctx)?;
        let display_name = clean_name(display_name, "Your name")?;
        self.ensure_free(&identity).await?;
        let code = normalize_code(code);
        let crew = match code.len() {
            8 => self.crew_by_code(&code).await?,
            _ => None,
        }
        .ok_or_else(|| DomainError::Invalid("That invite code does not match any crew".into()))?;
        if self.members(crew.id).await?.len() >= MAX_MEMBERS {
            return Err(DomainError::Conflict("This crew is full (20 members)"));
        }
        self.repo
            .add_member(
                &AccessScope::for_tenant(crew.id),
                member(&identity, crew.id, display_name, Role::Member),
            )
            .await?;
        self.view(identity).await
    }

    async fn owned_crew(&self, ctx: &SecurityContext) -> Result<(AccountIdentity, Crew)> {
        let identity = self.identity(ctx)?;
        let member = self
            .membership_of(&identity.uid)
            .await?
            .ok_or(DomainError::Forbidden("Join a crew first"))?;
        if member.role != Role::Owner {
            return Err(DomainError::Forbidden("Only the crew owner can do that"));
        }
        let crew = self
            .crew(member.crew_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        Ok((identity, crew))
    }

    /// Owner only: a new invite code; the old one stops working.
    pub async fn rotate_invite(&self, ctx: &SecurityContext) -> Result<Me> {
        let (identity, crew) = self.owned_crew(ctx).await?;
        let crew = Crew {
            invite_code: new_code()?,
            ..crew
        };
        self.repo
            .update_crew(&AccessScope::for_tenant(crew.id), &crew)
            .await?;
        self.view(identity).await
    }

    /// Owner only: rename the crew or move its calendar to another zone.
    pub async fn update(&self, ctx: &SecurityContext, name: &str, timezone: &str) -> Result<Me> {
        let (identity, crew) = self.owned_crew(ctx).await?;
        let crew = Crew {
            name: clean_name(name, "Crew name")?,
            timezone: clean_zone(timezone)?,
            ..crew
        };
        self.repo
            .update_crew(&AccessScope::for_tenant(crew.id), &crew)
            .await?;
        self.view(identity).await
    }

    /// Demo mode: recreates the demo crews and their accounts from scratch.
    pub async fn seed_demo(&self, timezone: &str) -> Result<()> {
        let timezone = clean_zone(timezone)?;
        for crew in demo::CREWS {
            let id = demo::crew_id(crew.key);
            let scope = AccessScope::for_tenant(id);
            self.repo.delete_crew(&scope).await?;
            let identity = |key: &str| AccountIdentity {
                uid: demo::uid(key),
                email: format!("{key}@demo.crew-juggle.test"),
                name: None,
            };
            let cast: Vec<_> = demo::USERS.iter().filter(|u| u.crew == crew.key).collect();
            let owner = cast
                .iter()
                .find(|u| u.owner)
                .ok_or(DomainError::Invalid("demo crew without owner".into()))?;
            let record = Crew {
                id,
                name: crew.name.into(),
                timezone: timezone.clone(),
                invite_code: new_code()?,
                created_at: Utc::now(),
            };
            let first = member(&identity(owner.key), id, owner.name.into(), Role::Owner);
            self.repo.create_crew(&scope, record, first).await?;
            for user in cast.iter().filter(|u| !u.owner) {
                let joined = member(&identity(user.key), id, user.name.into(), Role::Member);
                self.repo.add_member(&scope, joined).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;

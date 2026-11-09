//! The crews rules against an in-memory repository.
use super::*;
use crate::domain::repo::CrewRepo;
use async_trait::async_trait;
use std::sync::Mutex;
use toolkit_security::pep_properties;

/// Keeps crews and members in memory and applies scopes the way the secure
/// ORM does: a row is visible when the scope admits its crew.
#[derive(Default)]
struct Memory {
    crews: Mutex<Vec<Crew>>,
    members: Mutex<Vec<Member>>,
}

fn admits(scope: &AccessScope, crew: Uuid) -> bool {
    scope.is_unconstrained() || scope.contains_uuid(pep_properties::OWNER_TENANT_ID, crew)
}

#[async_trait]
impl CrewRepo for Memory {
    async fn member_by_uid(&self, scope: &AccessScope, uid: &str) -> Result<Option<Member>> {
        let members = self.members.lock().unwrap();
        Ok(members
            .iter()
            .find(|m| m.uid == uid && admits(scope, m.crew_id))
            .cloned())
    }
    async fn crew_by_code(&self, scope: &AccessScope, code: &str) -> Result<Option<Crew>> {
        let crews = self.crews.lock().unwrap();
        Ok(crews
            .iter()
            .find(|c| c.invite_code == code && admits(scope, c.id))
            .cloned())
    }
    async fn crew_ids(&self, scope: &AccessScope) -> Result<Vec<Uuid>> {
        let crews = self.crews.lock().unwrap();
        Ok(crews
            .iter()
            .filter(|c| admits(scope, c.id))
            .map(|c| c.id)
            .collect())
    }
    async fn crew(&self, scope: &AccessScope) -> Result<Option<Crew>> {
        let crews = self.crews.lock().unwrap();
        Ok(crews.iter().find(|c| admits(scope, c.id)).cloned())
    }
    async fn members(&self, scope: &AccessScope) -> Result<Vec<Member>> {
        let members = self.members.lock().unwrap();
        Ok(members
            .iter()
            .filter(|m| admits(scope, m.crew_id))
            .cloned()
            .collect())
    }
    async fn is_member(&self, scope: &AccessScope, subject: Uuid) -> Result<bool> {
        let members = self.members.lock().unwrap();
        Ok(members
            .iter()
            .any(|m| m.subject_id == subject && admits(scope, m.crew_id)))
    }
    async fn create_crew(&self, scope: &AccessScope, crew: Crew, owner: Member) -> Result<()> {
        assert!(admits(scope, crew.id) && admits(scope, owner.crew_id));
        self.crews.lock().unwrap().push(crew);
        self.members.lock().unwrap().push(owner);
        Ok(())
    }
    async fn add_member(&self, scope: &AccessScope, member: Member) -> Result<()> {
        assert!(admits(scope, member.crew_id));
        self.members.lock().unwrap().push(member);
        Ok(())
    }
    async fn update_crew(&self, scope: &AccessScope, crew: &Crew) -> Result<()> {
        let mut crews = self.crews.lock().unwrap();
        if let Some(stored) = crews
            .iter_mut()
            .find(|c| c.id == crew.id && admits(scope, c.id))
        {
            *stored = crew.clone();
        }
        Ok(())
    }
    async fn delete_crew(&self, scope: &AccessScope) -> Result<()> {
        self.crews.lock().unwrap().retain(|c| !admits(scope, c.id));
        self.members
            .lock()
            .unwrap()
            .retain(|m| !admits(scope, m.crew_id));
        Ok(())
    }
}

fn service() -> CrewsService {
    CrewsService::new(Arc::new(Memory::default()))
}

fn account(name: &str) -> AccountIdentity {
    AccountIdentity {
        uid: format!("uid-{name}"),
        email: format!("{name}@example.test"),
        name: Some(name.into()),
    }
}

/// Signs an account in and returns the context the gateway would build.
async fn signed_in(service: &CrewsService, name: &str) -> SecurityContext {
    let membership = service.sign_in(account(name)).await.unwrap();
    SecurityContext::builder()
        .subject_id(membership.subject_id)
        .subject_tenant_id(membership.tenant_id)
        .subject_type(membership.kind.subject_type())
        .build()
        .unwrap()
}

#[test]
fn names_and_zones_are_validated() {
    assert!(clean_name("  ", "Crew name").is_err());
    assert_eq!(clean_name(" Flat 4B ", "Crew name").unwrap(), "Flat 4B");
    assert!(clean_zone("Mars/Olympus").is_err());
    assert_eq!(clean_zone("Europe/Berlin").unwrap(), "Europe/Berlin");
}

#[tokio::test]
async fn a_new_account_lands_in_its_lobby_until_it_creates_a_crew() {
    let crews = service();
    let pip = account("pip");
    let first = crews.sign_in(pip.clone()).await.unwrap();
    assert_eq!(first.kind, MembershipKind::Guest);
    assert_eq!(first.tenant_id, lobby_tenant(&pip.uid));

    let ctx = signed_in(&crews, "pip").await;
    let me = crews
        .create(&ctx, "Flat 4B", "Pip", "Europe/Berlin")
        .await
        .unwrap();
    let crew = me.crew.unwrap();
    assert_eq!(crew.role, Role::Owner);
    assert_eq!(crew.invite_code.len(), 9, "shown as XXXX-XXXX");

    let again = crews.sign_in(pip).await.unwrap();
    assert_eq!(
        (again.kind, again.tenant_id),
        (MembershipKind::Member, crew.id)
    );
}

#[tokio::test]
async fn joining_needs_a_matching_code_and_one_crew_per_account() {
    let crews = service();
    let pip = signed_in(&crews, "pip").await;
    let code = crews
        .create(&pip, "Flat 4B", "Pip", "Europe/Berlin")
        .await
        .unwrap()
        .crew
        .unwrap()
        .invite_code;
    let zed = signed_in(&crews, "zed").await;

    let wrong = crews.join(&zed, "AAAA-AAAA", "Zed").await;
    assert!(matches!(wrong, Err(DomainError::Invalid(_))), "{wrong:?}");

    let joined = crews.join(&zed, &code.to_lowercase(), "Zed").await.unwrap();
    assert_eq!(joined.crew.unwrap().members.len(), 2);

    let twice = crews.join(&zed, &code, "Zed").await;
    assert!(matches!(twice, Err(DomainError::Conflict(_))), "{twice:?}");
    let create = crews.create(&zed, "Another", "Zed", "Europe/Berlin").await;
    assert!(
        matches!(create, Err(DomainError::Conflict(_))),
        "{create:?}"
    );
}

#[tokio::test]
async fn a_crew_holds_at_most_twenty_accounts() {
    let crews = service();
    let owner = signed_in(&crews, "owner").await;
    let code = crews
        .create(&owner, "Big", "Owner", "UTC")
        .await
        .unwrap()
        .crew
        .unwrap()
        .invite_code;
    for i in 1..20 {
        let ctx = signed_in(&crews, &format!("m{i}")).await;
        crews.join(&ctx, &code, "Member").await.unwrap();
    }
    let late = signed_in(&crews, "late").await;
    let full = crews.join(&late, &code, "Late").await;
    assert!(matches!(full, Err(DomainError::Conflict(_))), "{full:?}");
}

#[tokio::test]
async fn only_the_owner_rotates_the_code_or_renames_the_crew() {
    let crews = service();
    let pip = signed_in(&crews, "pip").await;
    let before = crews
        .create(&pip, "Flat 4B", "Pip", "Europe/Berlin")
        .await
        .unwrap()
        .crew
        .unwrap();
    let zed = signed_in(&crews, "zed").await;
    crews.join(&zed, &before.invite_code, "Zed").await.unwrap();

    assert!(matches!(
        crews.rotate_invite(&zed).await,
        Err(DomainError::Forbidden(_))
    ));
    assert!(matches!(
        crews.update(&zed, "Mine", "UTC").await,
        Err(DomainError::Forbidden(_))
    ));

    let rotated = crews.rotate_invite(&pip).await.unwrap().crew.unwrap();
    assert_ne!(rotated.invite_code, before.invite_code);
    let renamed = crews
        .update(&pip, "Flat 4C", "Europe/Paris")
        .await
        .unwrap()
        .crew
        .unwrap();
    assert_eq!(
        (renamed.name.as_str(), renamed.timezone.as_str()),
        ("Flat 4C", "Europe/Paris")
    );

    let stranger = signed_in(&crews, "stranger").await;
    let old = crews.join(&stranger, &before.invite_code, "S").await;
    assert!(
        matches!(old, Err(DomainError::Invalid(_))),
        "the old code stops working"
    );
}

#[tokio::test]
async fn me_needs_a_signed_in_identity_and_never_shows_another_crew() {
    let crews = service();
    let unknown = SecurityContext::builder()
        .subject_id(Uuid::new_v4())
        .subject_tenant_id(Uuid::new_v4())
        .build()
        .unwrap();
    assert!(matches!(
        crews.me(&unknown).await,
        Err(DomainError::Unauthorized)
    ));

    let pip = signed_in(&crews, "pip").await;
    let flat = crews
        .create(&pip, "Flat 4B", "Pip", "Europe/Berlin")
        .await
        .unwrap()
        .crew
        .unwrap();
    let nova = signed_in(&crews, "nova").await;
    let club = crews
        .create(&nova, "Robotics Club", "Nova", "Europe/Berlin")
        .await
        .unwrap()
        .crew
        .unwrap();
    assert_ne!(flat.id, club.id);
    let me = crews.me(&nova).await.unwrap().crew.unwrap();
    assert_eq!(me.members.len(), 1);
    assert!(me.members.iter().all(|m| m.uid == "uid-nova"));
    assert!(
        crews
            .is_member(subject_for("uid-nova"), club.id)
            .await
            .unwrap()
    );
    assert!(
        !crews
            .is_member(subject_for("uid-nova"), flat.id)
            .await
            .unwrap()
    );
    assert!(
        !crews
            .is_member(subject_for("uid-nova"), Uuid::nil())
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn the_demo_seed_rebuilds_both_demo_crews() {
    let crews = service();
    crews.seed_demo("Europe/Berlin").await.unwrap();
    crews.seed_demo("Europe/Berlin").await.unwrap();
    let flat = demo::crew_id("flat-4b");
    let members = crews.members(flat).await.unwrap();
    assert_eq!(
        members
            .iter()
            .map(|m| m.display_name.as_str())
            .collect::<Vec<_>>(),
        ["Pip", "Zed"]
    );
    assert_eq!(crews.all_crew_ids().await.unwrap().len(), 2);
    assert!(crews.seed_demo("Mars/Olympus").await.is_err());
}

//! A one-crew directory for tests, so a gear's policy and use cases run
//! without the crews gear.
use async_trait::async_trait;
use juggle_crews_sdk::{
    AccountIdentity, AccountMembership, CrewDirectoryV1, CrewInfo, CrewMember, CrewsClientV1,
    CrewsError, Role, is_worker, subject_for,
};
use toolkit_security::SecurityContext;
use uuid::Uuid;

/// One crew with fixed members; answers both crews contracts.
pub struct FixedCrew {
    /// The crew id.
    pub id: Uuid,
    /// Its members.
    pub members: Vec<CrewMember>,
}

impl FixedCrew {
    pub fn new(id: Uuid, people: &[(&str, &str)]) -> Self {
        Self {
            id,
            members: people
                .iter()
                .map(|(uid, name)| CrewMember {
                    uid: (*uid).into(),
                    subject_id: subject_for(uid),
                    display_name: (*name).into(),
                    role: Role::Member,
                })
                .collect(),
        }
    }

    /// A signed-in context for one of the members.
    pub fn context(&self, uid: &str) -> SecurityContext {
        SecurityContext::builder()
            .subject_id(subject_for(uid))
            .subject_tenant_id(self.id)
            .subject_type("member")
            .build()
            .expect("valid test identity")
    }

    fn admits(&self, ctx: &SecurityContext) -> bool {
        ctx.subject_tenant_id() == self.id
            && (is_worker(ctx)
                || self
                    .members
                    .iter()
                    .any(|m| m.subject_id == ctx.subject_id()))
    }
}

#[async_trait]
impl CrewsClientV1 for FixedCrew {
    async fn crew(&self, ctx: &SecurityContext) -> Result<CrewInfo, CrewsError> {
        if !self.admits(ctx) {
            return Err(CrewsError::Forbidden);
        }
        Ok(CrewInfo {
            id: self.id,
            name: "Test crew".into(),
            timezone: "Europe/Berlin".into(),
        })
    }
    async fn members(&self, ctx: &SecurityContext) -> Result<Vec<CrewMember>, CrewsError> {
        if !self.admits(ctx) {
            return Err(CrewsError::Forbidden);
        }
        Ok(self.members.clone())
    }
    async fn crew_ids(&self, ctx: &SecurityContext) -> Result<Vec<Uuid>, CrewsError> {
        if !is_worker(ctx) {
            return Err(CrewsError::Forbidden);
        }
        Ok(vec![self.id])
    }
}

#[async_trait]
impl CrewDirectoryV1 for FixedCrew {
    async fn is_member(&self, subject: Uuid, crew: Uuid) -> Result<bool, CrewsError> {
        Ok(crew == self.id && self.members.iter().any(|m| m.subject_id == subject))
    }
    async fn sign_in(&self, _: AccountIdentity) -> Result<AccountMembership, CrewsError> {
        Err(CrewsError::Unavailable)
    }
}

impl FixedCrew {
    /// The crews policy over this directory, as the plugin would decide it.
    pub fn policy(self: &std::sync::Arc<Self>) -> juggle_crews_sdk::CrewPolicy {
        juggle_crews_authz_plugin::in_process::crew_policy(self.clone())
    }
}

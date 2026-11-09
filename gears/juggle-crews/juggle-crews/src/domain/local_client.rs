//! The crews contract, served in-process to the gears that ask through ClientHub.
use super::error::DomainError;
use super::service::CrewsService;
use async_trait::async_trait;
use juggle_crews_sdk::{
    AccountIdentity, AccountMembership, CrewDirectoryV1, CrewInfo, CrewMember, CrewsClientV1,
    CrewsError, is_worker,
};
use std::sync::Arc;
use toolkit::domain_model;
use toolkit_security::SecurityContext;
use uuid::Uuid;

/// Implements both SDK traits over the crews service.
#[domain_model]
pub struct LocalClient {
    service: Arc<CrewsService>,
}

impl LocalClient {
    /// A client over the given service.
    pub fn new(service: Arc<CrewsService>) -> Self {
        Self { service }
    }

    /// Members and the worker may read a crew; nobody else.
    async fn admitted(&self, ctx: &SecurityContext) -> Result<Uuid, CrewsError> {
        let crew = ctx.subject_tenant_id();
        if is_worker(ctx)
            || self
                .service
                .is_member(ctx.subject_id(), crew)
                .await
                .map_err(unavailable)?
        {
            return Ok(crew);
        }
        Err(CrewsError::Forbidden)
    }
}

fn unavailable(error: DomainError) -> CrewsError {
    tracing::warn!(%error, "crews storage failed");
    CrewsError::Unavailable
}

#[async_trait]
impl CrewsClientV1 for LocalClient {
    async fn crew(&self, ctx: &SecurityContext) -> Result<CrewInfo, CrewsError> {
        let crew = self.admitted(ctx).await?;
        let crew = self
            .service
            .crew(crew)
            .await
            .map_err(unavailable)?
            .ok_or(CrewsError::NotFound)?;
        Ok(CrewInfo {
            id: crew.id,
            name: crew.name,
            timezone: crew.timezone,
        })
    }

    async fn members(&self, ctx: &SecurityContext) -> Result<Vec<CrewMember>, CrewsError> {
        let crew = self.admitted(ctx).await?;
        Ok(self
            .service
            .members(crew)
            .await
            .map_err(unavailable)?
            .into_iter()
            .map(|m| CrewMember {
                uid: m.uid,
                subject_id: m.subject_id,
                display_name: m.display_name,
                role: m.role,
            })
            .collect())
    }

    async fn crew_ids(&self, ctx: &SecurityContext) -> Result<Vec<Uuid>, CrewsError> {
        if !is_worker(ctx) {
            return Err(CrewsError::Forbidden);
        }
        self.service.all_crew_ids().await.map_err(unavailable)
    }
}

#[async_trait]
impl CrewDirectoryV1 for LocalClient {
    async fn is_member(&self, subject: Uuid, crew: Uuid) -> Result<bool, CrewsError> {
        self.service
            .is_member(subject, crew)
            .await
            .map_err(unavailable)
    }

    async fn sign_in(&self, identity: AccountIdentity) -> Result<AccountMembership, CrewsError> {
        self.service.sign_in(identity).await.map_err(unavailable)
    }
}

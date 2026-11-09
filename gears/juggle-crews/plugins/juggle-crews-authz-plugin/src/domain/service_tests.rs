//! The policy through the real PEP: a `PolicyEnforcer` that asks this plugin.
use super::*;
use async_trait::async_trait;
use authz_resolver_sdk::ResourceType;
use juggle_crews_sdk::{
    AccountIdentity, AccountMembership, CrewPolicy, CrewsError, integration_context,
    secrets_provisioner_context, worker_context,
};
use toolkit_security::SecurityContext;

struct Directory {
    members: Vec<(Uuid, Uuid)>,
}

#[async_trait]
impl CrewDirectoryV1 for Directory {
    async fn is_member(&self, subject: Uuid, crew: Uuid) -> Result<bool, CrewsError> {
        Ok(self.members.contains(&(subject, crew)))
    }
    async fn sign_in(&self, _: AccountIdentity) -> Result<AccountMembership, CrewsError> {
        Err(CrewsError::Unavailable)
    }
}

fn policy(members: Vec<(Uuid, Uuid)>) -> CrewPolicy {
    crate::in_process::crew_policy(Arc::new(Directory { members }))
}

fn account(subject: Uuid, crew: Uuid) -> SecurityContext {
    SecurityContext::builder()
        .subject_id(subject)
        .subject_tenant_id(crew)
        .subject_type("member")
        .build()
        .unwrap()
}

#[tokio::test]
async fn a_member_gets_exactly_their_crew_and_strangers_get_nothing() {
    let (pip, flat, club) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let policy = policy(vec![(pip, flat)]);

    let scope = policy.scope(&account(pip, flat)).await.unwrap();
    assert!(scope.contains_uuid(pep_properties::OWNER_TENANT_ID, flat));
    assert!(!scope.contains_uuid(pep_properties::OWNER_TENANT_ID, club));

    // Claiming someone else's crew in the token does not help.
    assert!(policy.scope(&account(pip, club)).await.is_err());
    assert!(policy.scope(&account(Uuid::new_v4(), flat)).await.is_err());
}

#[tokio::test]
async fn the_worker_may_enter_any_single_crew_but_not_the_nil_tenant() {
    let crew = Uuid::new_v4();
    let policy = policy(vec![]);
    let scope = policy.scope(&worker_context(crew)).await.unwrap();
    assert!(scope.contains_uuid(pep_properties::OWNER_TENANT_ID, crew));
    assert!(policy.scope(&worker_context(Uuid::nil())).await.is_err());
}

#[tokio::test]
async fn a_member_posing_as_a_service_is_not_the_worker() {
    let crew = Uuid::new_v4();
    let impostor = SecurityContext::builder()
        .subject_id(Uuid::new_v4())
        .subject_tenant_id(crew)
        .subject_type("service")
        .build()
        .unwrap();
    assert!(policy(vec![]).scope(&impostor).await.is_err());
}

#[tokio::test]
async fn other_resources_are_denied() {
    const OTHER: ResourceType = ResourceType::from_static(
        "gts.cf.juggle.other.v1~",
        &[pep_properties::OWNER_TENANT_ID],
    );
    let (pip, flat) = (Uuid::new_v4(), Uuid::new_v4());
    let enforcer = crate::in_process::enforcer(Arc::new(Directory {
        members: vec![(pip, flat)],
    }));
    let ctx = account(pip, flat);
    assert!(
        enforcer
            .access_scope(&ctx, &CREW_DATA, "access", None)
            .await
            .is_ok()
    );
    assert!(
        enforcer
            .access_scope(&ctx, &OTHER, "access", None)
            .await
            .is_err()
    );
    assert!(
        enforcer
            .access_scope(&ctx, &CREW_DATA, "delete", None)
            .await
            .is_err()
    );
}

const OAGW_PROXY: ResourceType = ResourceType::from_static(
    "gts.cf.core.oagw.proxy.v1~",
    &[pep_properties::OWNER_TENANT_ID],
);
const SECRET: ResourceType = ResourceType::from_static(
    "gts.cf.core.credstore.secret.v1~cf.core.credstore.generic.v1~",
    &[pep_properties::OWNER_TENANT_ID],
);

#[tokio::test]
async fn the_integration_identity_may_call_out_and_read_keys_but_not_write_them() {
    let enforcer = crate::in_process::enforcer(Arc::new(Directory { members: vec![] }));
    let ctx = integration_context();
    let scope = enforcer
        .access_scope(&ctx, &OAGW_PROXY, "invoke", None)
        .await
        .unwrap();
    assert!(scope.contains_uuid(pep_properties::OWNER_TENANT_ID, PLATFORM_TENANT));
    assert!(
        enforcer
            .access_scope(&ctx, &SECRET, "read", None)
            .await
            .is_ok()
    );
    assert!(
        enforcer
            .access_scope(&ctx, &SECRET, "create", None)
            .await
            .is_err()
    );
    assert!(
        enforcer
            .access_scope(&ctx, &CREW_DATA, "access", None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn only_the_provisioner_writes_keys() {
    let enforcer = crate::in_process::enforcer(Arc::new(Directory { members: vec![] }));
    let ctx = secrets_provisioner_context();
    assert!(
        enforcer
            .access_scope(&ctx, &SECRET, "create", None)
            .await
            .is_ok()
    );
    assert!(
        enforcer
            .access_scope(&ctx, &OAGW_PROXY, "invoke", None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn members_and_the_worker_never_reach_platform_services() {
    let (pip, flat) = (Uuid::new_v4(), Uuid::new_v4());
    let enforcer = crate::in_process::enforcer(Arc::new(Directory {
        members: vec![(pip, flat)],
    }));
    let member = account(pip, flat);
    assert!(
        enforcer
            .access_scope(&member, &OAGW_PROXY, "invoke", None)
            .await
            .is_err()
    );
    assert!(
        enforcer
            .access_scope(&member, &SECRET, "read", None)
            .await
            .is_err()
    );
    let worker = worker_context(flat);
    assert!(
        enforcer
            .access_scope(&worker, &OAGW_PROXY, "invoke", None)
            .await
            .is_err()
    );
    let platform_worker = worker_context(PLATFORM_TENANT);
    assert!(
        enforcer
            .access_scope(&platform_worker, &CREW_DATA, "access", None)
            .await
            .is_err()
    );
}

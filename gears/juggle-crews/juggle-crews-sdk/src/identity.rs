//! Stable ids for accounts, lobbies and the internal worker.
use toolkit_security::SecurityContext;
use uuid::Uuid;

fn v5(name: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, name.as_bytes())
}

/// Stable subject id for an identity-provider account.
pub fn subject_for(uid: &str) -> Uuid {
    v5(&format!("crew-juggle/account/{uid}"))
}

/// Tenant of a signed-in account that has not created or joined a crew yet.
/// No policy grants access to it, so such a caller reaches only the crews API.
pub fn lobby_tenant(uid: &str) -> Uuid {
    v5(&format!("crew-juggle/lobby/{uid}"))
}

/// Subject id of the internal worker that sends reminders and scans receipts.
pub fn worker_subject() -> Uuid {
    v5("crew-juggle/internal-worker")
}

/// Identity for background work (reminders, receipt processing) inside one crew.
pub fn worker_context(crew: Uuid) -> SecurityContext {
    SecurityContext::builder()
        .subject_id(worker_subject())
        .subject_tenant_id(crew)
        .subject_type("service")
        .build()
        .expect("constant worker identity is valid")
}

/// True for the internal worker identity, never for a signed-in account.
pub fn is_worker(ctx: &SecurityContext) -> bool {
    ctx.subject_id() == worker_subject() && ctx.subject_type() == Some("service")
}

/// The platform's own tenant, where app-wide integrations live: the OpenAI
/// upstream in oagw and its key in credstore. No crew is ever this tenant.
pub const PLATFORM_TENANT: Uuid = toolkit_security::constants::DEFAULT_TENANT_ID;

/// Subject id of the integration identity that calls outside services.
pub fn integration_subject() -> Uuid {
    v5("crew-juggle/integration")
}

/// Identity for calls to outside services through oagw. It belongs to no
/// crew, so a crew's data never travels with it and no crew owns the key.
pub fn integration_context() -> SecurityContext {
    service_context(integration_subject())
}

/// Subject id of the identity that writes app-wide secrets at start-up.
pub fn secrets_provisioner_subject() -> Uuid {
    v5("crew-juggle/secrets-provisioner")
}

/// Identity that writes app-wide secrets into credstore at start-up.
pub fn secrets_provisioner_context() -> SecurityContext {
    service_context(secrets_provisioner_subject())
}

fn service_context(subject: Uuid) -> SecurityContext {
    SecurityContext::builder()
        .subject_id(subject)
        .subject_tenant_id(PLATFORM_TENANT)
        .subject_type("service")
        .build()
        .expect("constant service identity is valid")
}

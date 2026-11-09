//! Who may access which crew's data.
//!
//! | Caller                                  | Resource / action          | Answer                    |
//! |-----------------------------------------|----------------------------|---------------------------|
//! | member of the crew in its context       | crew data / access         | yes, only that crew       |
//! | internal worker, any one non-nil crew   | crew data / access         | yes, only that crew       |
//! | integration identity, platform tenant   | oagw (any); credstore read | yes, platform tenant only |
//! | secrets provisioner, platform tenant    | credstore (any)            | yes, platform tenant only |
//! | anyone else, or anything else           |                            | no                        |
//!
//! The last row also closes the platform's own REST surfaces in this server
//! (`/oagw/v1/*`, `/credstore/v1/*`) to signed-in members.
use authz_resolver_sdk::{
    Constraint, EqPredicate, EvaluationRequest, EvaluationResponse, EvaluationResponseContext,
    Predicate,
};
use juggle_crews_sdk::{
    CREW_DATA, CrewDirectoryV1, PLATFORM_TENANT, integration_subject, policy::ACCESS,
    secrets_provisioner_subject, worker_subject,
};
use std::sync::Arc;
use toolkit::domain_model;
use toolkit_security::pep_properties;
use uuid::Uuid;

/// Decides crew data access from crew membership.
#[domain_model]
pub struct CrewAccessPolicy {
    directory: Arc<dyn CrewDirectoryV1>,
}

fn deny() -> EvaluationResponse {
    EvaluationResponse {
        decision: false,
        context: EvaluationResponseContext::default(),
    }
}

/// What the integration identity uses in oagw: upstreams, routes, the proxy.
const OAGW_RESOURCES: [&str; 3] = [
    oagw_sdk::PROXY_SCHEMA,
    oagw_sdk::UPSTREAM_SCHEMA,
    oagw_sdk::ROUTE_SCHEMA,
];

/// Allows the request, limited to `tenant` when the asking gear can apply that.
fn allow(request: &EvaluationRequest, tenant: Uuid) -> EvaluationResponse {
    let supported = request
        .context
        .supported_properties
        .iter()
        .any(|p| p == pep_properties::OWNER_TENANT_ID);
    let constraints = if supported {
        vec![Constraint {
            predicates: vec![Predicate::Eq(EqPredicate::new(
                pep_properties::OWNER_TENANT_ID,
                tenant,
            ))],
        }]
    } else {
        vec![]
    };
    EvaluationResponse {
        decision: true,
        context: EvaluationResponseContext {
            constraints,
            ..EvaluationResponseContext::default()
        },
    }
}

/// The two service identities that act in the platform tenant.
fn platform_service(request: &EvaluationRequest, tenant: Uuid) -> EvaluationResponse {
    let resource = request.resource.resource_type.as_str();
    let subject = request.subject.id;
    // Every credstore secret type derives from the generic secret type.
    let secret = resource.starts_with(credstore_sdk::SECRET_RESOURCE_TYPE);
    let allowed = if subject == integration_subject() {
        OAGW_RESOURCES.contains(&resource) || (secret && request.action.name == "read")
    } else if subject == secrets_provisioner_subject() {
        secret
    } else {
        false
    };
    if allowed {
        allow(request, tenant)
    } else {
        deny()
    }
}

impl CrewAccessPolicy {
    /// A policy that asks `directory` about membership.
    pub fn new(directory: Arc<dyn CrewDirectoryV1>) -> Self {
        Self { directory }
    }

    /// Answers one access question.
    pub async fn evaluate(&self, request: &EvaluationRequest) -> EvaluationResponse {
        let tenant = request
            .subject
            .properties
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<Uuid>().ok());
        let service = request.subject.subject_type.as_deref() == Some("service");
        if service && tenant == Some(PLATFORM_TENANT) {
            return platform_service(request, PLATFORM_TENANT);
        }
        if request.resource.resource_type != CREW_DATA.name() || request.action.name != ACCESS {
            return deny();
        }
        // The crew is the tenant the sign-in plugin put on the caller's context.
        let Some(crew) = tenant.filter(|c| !c.is_nil()) else {
            return deny();
        };
        let worker = service && request.subject.id == worker_subject();
        let allowed = worker
            || self
                .directory
                .is_member(request.subject.id, crew)
                .await
                .unwrap_or_else(|error| {
                    tracing::warn!(%error, "membership check failed; denying");
                    false
                });
        if !allowed {
            return deny();
        }
        EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate::new(
                        pep_properties::OWNER_TENANT_ID,
                        crew,
                    ))],
                }],
                ..EvaluationResponseContext::default()
            },
        }
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;

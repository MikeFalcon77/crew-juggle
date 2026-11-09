//! Domain errors as RFC 9457 problems. `error_code` and `error_domain` are
//! what the web app keys its behaviour on, so they never change.
use crate::domain::DomainError;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use toolkit_canonical_errors::Problem;

/// A domain error on its way to the client.
pub struct RestError(pub DomainError);

impl From<DomainError> for RestError {
    fn from(error: DomainError) -> Self {
        Self(error)
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            DomainError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthenticated"),
            DomainError::Forbidden(_) => (StatusCode::FORBIDDEN, "access_denied"),
            DomainError::Invalid(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
            DomainError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            DomainError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            DomainError::Unavailable(cause) => {
                tracing::error!(%cause, "crews storage failure");
                (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
            }
        };
        let message = self.0.to_string();
        let problem = Problem {
            problem_type: "about:blank".into(),
            title: message.clone(),
            status: Some(status.as_u16()),
            detail: message,
            instance: None,
            trace_id: None,
            context: serde_json::json!({}),
            error_code: Some(code.into()),
            error_domain: Some("juggle-crews.v1".into()),
        };
        (
            status,
            [
                (header::CONTENT_TYPE, "application/problem+json"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Json(problem),
        )
            .into_response()
    }
}

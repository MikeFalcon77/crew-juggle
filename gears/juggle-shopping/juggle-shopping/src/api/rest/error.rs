//! Domain errors as RFC 9457 problems; `error_code` is what the web app reads.
use crate::domain::Error;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use toolkit_canonical_errors::Problem;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Error::Forbidden => (StatusCode::FORBIDDEN, "access_denied"),
            Error::Conflict => (StatusCode::CONFLICT, "revision_conflict"),
            Error::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Error::Storage => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            Error::Invalid(_) | Error::Model(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
        };
        let message = self.to_string();
        let problem = Problem {
            problem_type: "about:blank".into(),
            title: message.clone(),
            status: Some(status.as_u16()),
            detail: message,
            instance: None,
            trace_id: None,
            context: serde_json::json!({}),
            error_code: Some(code.into()),
            error_domain: None,
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

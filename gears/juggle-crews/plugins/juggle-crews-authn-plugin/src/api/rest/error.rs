//! Errors of the sign-in routes, in the crews API's problem format.
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use toolkit_canonical_errors::Problem;

/// What can go wrong on the sign-in routes.
pub enum SignInRouteError {
    /// Demo sign-in on a server that is not in demo mode.
    NotDemo,
    /// A character that is not in the demo cast.
    UnknownCharacter,
}

impl IntoResponse for SignInRouteError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::NotDemo => (StatusCode::NOT_FOUND, "not_found", "Not found"),
            Self::UnknownCharacter => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "Pick one of the demo characters",
            ),
        };
        let problem = Problem {
            problem_type: "about:blank".into(),
            title: message.into(),
            status: Some(status.as_u16()),
            detail: message.into(),
            instance: None,
            trace_id: None,
            context: serde_json::json!({}),
            error_code: Some(code.into()),
            // The routes keep the crews API's paths and error domain.
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

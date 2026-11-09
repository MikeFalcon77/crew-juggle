use crate::domain::Error;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use toolkit_canonical_errors::Problem;
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Forbidden => (StatusCode::FORBIDDEN, "access_denied"),
            Self::Invalid(_) => (StatusCode::BAD_REQUEST, "request_failed"),
            Self::Conflict => (StatusCode::CONFLICT, "revision_conflict"),
            Self::ConfirmationRequired => (StatusCode::CONFLICT, "confirmation_required"),
            Self::NotFound => (StatusCode::NOT_FOUND, "request_failed"),
            Self::Storage(_) => (StatusCode::SERVICE_UNAVAILABLE, "request_failed"),
        };
        let message = self.to_string();
        let problem = Problem {
            problem_type: "about:blank".into(),
            title: message.clone(),
            status: Some(status.as_u16()),
            detail: message.clone(),
            instance: None,
            trace_id: None,
            context: serde_json::json!({}),
            error_code: Some(code.into()),
            error_domain: Some("juggle-routines.v1".into()),
        };
        // Existing clients continue to receive their message/code in addition to RFC-9457.
        let mut body = serde_json::to_value(problem).expect("Problem contains serializable fields");
        body["message"] = message.into();
        body["code"] = code.into();
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}

//! Handlers of the sign-in routes.
use super::dto::{DemoSessionRequest, DemoTokenDto, DemoUserDto, SessionConfigDto};
use super::error::SignInRouteError;
use crate::infra::demo_tokens::DemoTokens;
use axum::{
    Extension, Json,
    http::header,
    response::{IntoResponse, Response},
};
use juggle_crews_sdk::demo;
use std::sync::Arc;

/// What the routes need: demo tokens, present only in demo mode.
#[derive(Clone)]
pub struct SignInState {
    /// `Some` in demo mode.
    pub demo: Option<Arc<DemoTokens>>,
}

fn no_store(value: impl serde::Serialize) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}

/// `GET /juggle-crews/v1/config`
pub async fn config(Extension(state): Extension<SignInState>) -> Response {
    let demo_users = match state.demo {
        None => vec![],
        Some(_) => demo::USERS
            .iter()
            .map(|u| DemoUserDto {
                key: u.key.into(),
                name: u.name.into(),
                crew: demo::CREWS
                    .iter()
                    .find(|c| c.key == u.crew)
                    .map(|c| c.name.into()),
                tagline: u.tagline.into(),
            })
            .collect(),
    };
    let sign_in = if state.demo.is_some() {
        "demo"
    } else {
        "google"
    };
    no_store(SessionConfigDto {
        sign_in: sign_in.into(),
        demo_users,
    })
}

/// `POST /juggle-crews/v1/demo/sessions`
pub async fn demo_session(
    Extension(state): Extension<SignInState>,
    Json(body): Json<DemoSessionRequest>,
) -> Result<Response, SignInRouteError> {
    let tokens = state.demo.as_ref().ok_or(SignInRouteError::NotDemo)?;
    let token = tokens
        .mint(&body.user)
        .ok_or(SignInRouteError::UnknownCharacter)?;
    Ok(no_store(DemoTokenDto { token }))
}

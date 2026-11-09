//! HTTP handlers: parse, call the service, map the answer. No rules here.
use super::dto::{CreateCrewRequest, JoinCrewRequest, MeDto, UpdateCrewRequest};
use super::error::RestError;
use crate::domain::CrewsService;
use axum::{
    Extension, Json,
    http::header,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use toolkit_security::SecurityContext;

/// Answers carry personal data, so browsers and proxies must not keep them.
fn no_store(me: MeDto) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(me)).into_response()
}

/// `GET /juggle-crews/v1/me`
pub async fn me(
    Extension(service): Extension<Arc<CrewsService>>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Response, RestError> {
    Ok(no_store(service.me(&ctx).await?.into()))
}

/// `POST /juggle-crews/v1/crews`
pub async fn create(
    Extension(service): Extension<Arc<CrewsService>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<CreateCrewRequest>,
) -> Result<Response, RestError> {
    let me = service
        .create(&ctx, &body.name, &body.display_name, &body.timezone)
        .await?;
    Ok(no_store(me.into()))
}

/// `POST /juggle-crews/v1/memberships`
pub async fn join(
    Extension(service): Extension<Arc<CrewsService>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<JoinCrewRequest>,
) -> Result<Response, RestError> {
    let me = service
        .join(&ctx, &body.invite_code, &body.display_name)
        .await?;
    Ok(no_store(me.into()))
}

/// `POST /juggle-crews/v1/crew/invite-code`
pub async fn rotate_invite(
    Extension(service): Extension<Arc<CrewsService>>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Response, RestError> {
    Ok(no_store(service.rotate_invite(&ctx).await?.into()))
}

/// `PUT /juggle-crews/v1/crew`
pub async fn update(
    Extension(service): Extension<Arc<CrewsService>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<UpdateCrewRequest>,
) -> Result<Response, RestError> {
    let me = service.update(&ctx, &body.name, &body.timezone).await?;
    Ok(no_store(me.into()))
}

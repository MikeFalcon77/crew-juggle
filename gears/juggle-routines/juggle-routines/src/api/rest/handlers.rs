//! HTTP handlers of the routines API: parse, call the use case, map the answer.
use super::dto;
use crate::domain::{self, Error};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::header,
    response::{IntoResponse, Response},
};
use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use toolkit_security::SecurityContext;

/// What the handlers need: the routines use cases.
pub struct App {
    /// Authorized crew use cases and the cross-gear SDK implementation.
    pub application: Arc<domain::application::Application>,
}
impl std::ops::Deref for App {
    type Target = domain::application::Application;
    fn deref(&self) -> &Self::Target {
        &self.application
    }
}
type ApiResult<T> = std::result::Result<Json<T>, Error>;
/// The caller's account id; fails unless they belong to the crew.
async fn actor(app: &App, ctx: &SecurityContext) -> std::result::Result<String, Error> {
    app.actor(ctx).await
}
/// `GET /juggle-routines/v1/me`
pub async fn me(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
) -> ApiResult<dto::MeView> {
    let uid = actor(&app, &ctx).await?;
    let (_, state) = app.service.read(&ctx).await?;
    Ok(Json(dto::MeView {
        uid,
        timezone: state.zone.name().into(),
        today: state.today(Utc::now()),
        vapid_public_key: app.vapid_public_key.clone(),
    }))
}
/// `GET /juggle-routines/v1/me/preferences`
pub async fn preferences(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
) -> ApiResult<dto::PreferencesView> {
    let uid = actor(&app, &ctx).await?;
    let (_, state) = app.service.read(&ctx).await?;
    let members = app
        .service
        .crews
        .members(&ctx)
        .await
        .map_err(|_| Error::Forbidden)?;
    Ok(Json(dto::PreferencesView {
        preferences: state.preferences_for(&uid).into(),
        members: members
            .iter()
            .map(|m| dto::MemberPreferences {
                uid: m.uid.clone(),
                name: m.display_name.clone(),
                preferences: state.preferences_for(&m.uid).into(),
            })
            .collect(),
    }))
}
/// `PUT /juggle-routines/v1/me/preferences`
pub async fn edit_preferences(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<dto::PreferencesEdit>,
) -> ApiResult<dto::Preferences> {
    let body: domain::PreferencesEdit = body.into();
    let actor = &actor(&app, &ctx).await?;
    Ok(Json(
        app.service
            .mutate(&ctx, |state| state.edit_preferences(actor, &body))
            .await?
            .into(),
    ))
}
/// `GET /juggle-routines/v1/day?date=YYYY-MM-DD`
pub async fn day(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Query(query): Query<dto::DayQuery>,
) -> ApiResult<dto::DayView> {
    let now = Utc::now();
    let day = app.service.day(&ctx, query.date, now).await?;
    Ok(Json(dto::DayView {
        day: day.into(),
        avatars: avatar_metadata(&app, &ctx).await?,
    }))
}
/// `PUT /juggle-routines/v1/occurrences/{date}/{id}`
pub async fn mark(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path((date, id)): Path<(NaiveDate, String)>,
    Json(body): Json<dto::Mark>,
) -> ApiResult<dto::Occurrence> {
    let body: domain::Mark = body.into();
    let actor = &actor(&app, &ctx).await?;
    let now = Utc::now();
    Ok(Json(
        app.service
            .mutate(&ctx, |state| state.mark(date, &id, actor, &body, now))
            .await?
            .into(),
    ))
}
/// `GET /juggle-routines/v1/schedule`
pub async fn schedule(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
) -> ApiResult<dto::ScheduleView> {
    let schedule = app.service.schedule(&ctx, Utc::now()).await?;
    Ok(Json(dto::ScheduleView {
        schedule: schedule.into(),
        avatars: avatar_metadata(&app, &ctx).await?,
    }))
}

/// `PUT /juggle-routines/v1/schedule`
pub async fn edit(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<dto::Edit>,
) -> ApiResult<dto::Version> {
    let body: domain::Edit = body.into();
    let actor = &actor(&app, &ctx).await?;
    let now = Utc::now();
    Ok(Json(
        app.service
            .mutate(&ctx, |state| state.edit(actor, &body, now))
            .await?
            .into(),
    ))
}
/// `GET /juggle-routines/v1/people`
pub async fn people(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
) -> ApiResult<dto::PeopleView> {
    let (_, state) = app.service.read(&ctx).await?;
    let today = state.today(Utc::now());
    Ok(Json(dto::PeopleView {
        people: state.people.into_iter().map(dto::Person::from).collect(),
        today,
        avatars: avatar_metadata(&app, &ctx).await?,
    }))
}
/// `PUT /juggle-routines/v1/people`
pub async fn edit_person(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<dto::PersonEdit>,
) -> ApiResult<dto::Person> {
    let body: domain::PersonEdit = body.into();
    let actor = &actor(&app, &ctx).await?;
    let now = Utc::now();
    Ok(Json(
        app.service
            .mutate(&ctx, |state| state.edit_person(actor, &body, now))
            .await?
            .into(),
    ))
}
/// `POST /juggle-routines/v1/push/subscriptions`
pub async fn subscribe(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<dto::Subscribe>,
) -> ApiResult<dto::SubscriptionCreated> {
    let id = app
        .subscribe(&ctx, body.endpoint, body.keys.p256dh, body.keys.auth)
        .await?;
    Ok(Json(dto::SubscriptionCreated { id }))
}
/// `DELETE /juggle-routines/v1/push/subscriptions/{id}`
pub async fn unsubscribe(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
) -> ApiResult<dto::Done> {
    app.unsubscribe(&ctx, &id).await?;
    Ok(Json(dto::Done { ok: true }))
}
/// `POST /juggle-routines/v1/push/test`
pub async fn test_push(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<dto::TestPush>,
) -> ApiResult<dto::Done> {
    app.test_push(&ctx, &body.id).await?;
    Ok(Json(dto::Done { ok: true }))
}
/// `PUT /juggle-routines/v1/schedule/{id}/appearance`
pub async fn appearance(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
    Json(body): Json<dto::AppearanceEdit>,
) -> ApiResult<dto::Appearance> {
    let body: domain::AppearanceEdit = body.into();
    let actor = &actor(&app, &ctx).await?;
    Ok(Json(
        app.service
            .mutate(&ctx, |s| s.edit_appearance(&id, actor, &body))
            .await?
            .into(),
    ))
}
/// `GET /juggle-routines/v1/people/{id}/avatar`
pub async fn avatar(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
) -> std::result::Result<Response, Error> {
    actor(&app, &ctx).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        app.avatar_image(&ctx, &id).await?,
    )
        .into_response())
}
/// `PUT /juggle-routines/v1/people/{id}/avatar`
pub async fn upload_avatar(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<dto::AvatarMeta> {
    let mut operation = None;
    let mut revision = None;
    let mut image = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::Invalid("Could not read the photo".into()))?
    {
        match field.name() {
            Some("operation_id") if operation.is_none() => {
                operation = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| Error::Invalid("Invalid operation id".into()))?
                        .parse()
                        .map_err(|_| Error::Invalid("Invalid operation id".into()))?,
                )
            }
            Some("expected_revision") if revision.is_none() => {
                revision = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| Error::Invalid("Invalid revision".into()))?
                        .parse()
                        .map_err(|_| Error::Invalid("Invalid revision".into()))?,
                )
            }
            Some("file") if image.is_none() && field.content_type() == Some("image/png") => {
                image = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| Error::Invalid("The photo is too large".into()))?
                        .to_vec(),
                )
            }
            _ => return Err(Error::Invalid("Unexpected photo upload fields".into())),
        }
    }
    let request = domain::AvatarEdit {
        operation_id: operation.ok_or_else(|| Error::Invalid("Missing operation id".into()))?,
        expected_revision: revision
            .ok_or_else(|| Error::Invalid("Missing photo revision".into()))?,
    };
    let bytes = image.ok_or_else(|| Error::Invalid("Pick a photo".into()))?;
    let normalized = app.normalizer.normalize(bytes).await?;
    Ok(Json(
        app.change_avatar(&ctx, &id, &request, Some(normalized))
            .await?
            .into(),
    ))
}
/// `DELETE /juggle-routines/v1/people/{id}/avatar`
pub async fn remove_avatar(
    Extension(app): Extension<Arc<App>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
    Json(body): Json<dto::AvatarEdit>,
) -> ApiResult<dto::AvatarMeta> {
    let body: domain::AvatarEdit = body.into();
    Ok(Json(
        app.change_avatar(&ctx, &id, &body, None).await?.into(),
    ))
}

async fn avatar_metadata(
    app: &App,
    ctx: &SecurityContext,
) -> domain::Result<std::collections::HashMap<String, dto::AvatarMeta>> {
    Ok(app
        .avatar_metadata(ctx)
        .await?
        .into_iter()
        .map(|(id, meta)| (id, meta.into()))
        .collect())
}

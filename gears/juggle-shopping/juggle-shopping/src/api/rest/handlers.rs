//! HTTP handlers of the shopping API: parse, call the use case, map the answer.
use super::dto::{
    CommandApplied, CommandRequest, Event, LedgerPage, LedgerQuery, ReceiptPhotos, StateView,
    UploadAccepted,
};
use crate::domain::{Error, Service};
use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine;
use std::sync::Arc;
use toolkit_security::SecurityContext;
use uuid::Uuid;

/// Most uploads are one photo; five covers a long receipt.
const MAX_PHOTOS: usize = 5;

fn no_store(value: impl serde::Serialize) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}

/// `GET /juggle-shopping/v1/state`
pub async fn state(
    Extension(service): Extension<Arc<Service>>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Response, Error> {
    let (revision, state, ai_configured, today) = service.overview(&ctx).await?;
    Ok(no_store(StateView::new(
        revision,
        state,
        ai_configured,
        today,
    )))
}

/// `POST /juggle-shopping/v1/commands`
pub async fn command(
    Extension(service): Extension<Arc<Service>>,
    Extension(ctx): Extension<SecurityContext>,
    Json(command): Json<CommandRequest>,
) -> Result<Response, Error> {
    service.command(&ctx, &command.into()).await?;
    Ok(no_store(CommandApplied { ok: true }))
}

/// `GET /juggle-shopping/v1/ledger`
pub async fn ledger(
    Extension(service): Extension<Arc<Service>>,
    Extension(ctx): Extension<SecurityContext>,
    Query(query): Query<LedgerQuery>,
) -> Result<Response, Error> {
    let (events, next) = service
        .ledger_page(&ctx, query.receipt_id.as_deref(), query.after.as_deref())
        .await?;
    Ok(no_store(LedgerPage {
        events: events.into_iter().map(Event::from).collect(),
        next,
    }))
}

/// `POST /juggle-shopping/v1/receipts` (multipart: `operation_id`, `photo` × 1–5)
pub async fn upload(
    Extension(service): Extension<Arc<Service>>,
    Extension(ctx): Extension<SecurityContext>,
    mut multipart: Multipart,
) -> Result<Response, Error> {
    let mut id = None;
    let mut photos = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::Invalid("Could not read the receipt photo".into()))?
    {
        match field.name() {
            Some("operation_id") if id.is_none() => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| Error::Invalid("Invalid operation id".into()))?;
                id = Some(
                    text.parse::<Uuid>()
                        .map_err(|_| Error::Invalid("Invalid operation id".into()))?,
                );
            }
            Some("photo") if photos.len() < MAX_PHOTOS => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| Error::Invalid("The photo is too large".into()))?;
                photos.push(bytes.to_vec());
            }
            _ => {
                return Err(Error::Invalid(
                    "Add at most five photos of one receipt".into(),
                ));
            }
        }
    }
    let id = id.ok_or_else(|| Error::Invalid("Missing operation id".into()))?;
    let receipt = service.upload_photos(&ctx, id, photos).await?;
    Ok((
        StatusCode::ACCEPTED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(UploadAccepted { id: receipt }),
    )
        .into_response())
}

/// `GET /juggle-shopping/v1/receipts/{id}/photos`
pub async fn receipt_photos(
    Extension(service): Extension<Arc<Service>>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<String>,
) -> Result<Response, Error> {
    let answer = match service.receipt_photos(&ctx, &id).await? {
        None => ReceiptPhotos {
            photos: vec![],
            expired: true,
        },
        Some(photos) => ReceiptPhotos {
            photos: photos
                .iter()
                .map(|p| {
                    format!(
                        "data:image/jpeg;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(p)
                    )
                })
                .collect(),
            expired: false,
        },
    };
    Ok(no_store(answer))
}

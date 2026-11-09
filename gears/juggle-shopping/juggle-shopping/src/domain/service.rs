//! Shopping use cases: read, change, upload, and notify the crew.
use super::error::{Error, Result};
use super::ledger::Event;
use super::model::{Command, Photo, Receipt, State};
use super::ports::{ActionFingerprint, Intelligence, PhotoNormalizer, Store};
use super::rules::today_in;
use base64::Engine;
use chrono::Utc;
use juggle_crews_sdk::{CrewMember, CrewsClientV1, policy::CrewPolicy};
use juggle_routines_sdk::{Notification, NotificationKind, RoutinesClientV1};
use std::sync::Arc;
use toolkit::domain_model;
use toolkit_security::{AccessScope, SecurityContext};
use uuid::Uuid;

/// The shopping use cases.
#[domain_model]
pub struct Service {
    /// Where the crew's shopping state and photos are stored.
    pub store: Arc<dyn Store>,
    /// Reads receipts and classifies products.
    pub model: Arc<dyn Intelligence>,
    /// Stable text form of commands, for idempotent replay.
    pub fingerprint: Arc<dyn ActionFingerprint>,
    /// Turns uploaded photos into stored JPEGs.
    pub photos: Arc<dyn PhotoNormalizer>,
    /// This gear is the PEP: every operation asks the PDP for the crew scope.
    pub policy: CrewPolicy,
    /// Crew zone and members, from the crews gear via ClientHub.
    pub crews: Arc<dyn CrewsClientV1>,
    /// Push to crewmates through the routines gear (it owns the devices).
    pub routines: Option<Arc<dyn RoutinesClientV1>>,
}
impl Service {
    pub async fn scope(&self, ctx: &SecurityContext) -> Result<AccessScope> {
        self.policy.scope(ctx).await.map_err(|_| Error::Forbidden)
    }
    /// The signed-in member behind `ctx`.
    pub async fn member(&self, ctx: &SecurityContext) -> Result<CrewMember> {
        self.crews
            .members(ctx)
            .await
            .map_err(|_| Error::Forbidden)?
            .into_iter()
            .find(|m| m.subject_id == ctx.subject_id())
            .ok_or(Error::Forbidden)
    }
    /// Display name recorded on items and receipts.
    pub async fn actor(&self, ctx: &SecurityContext) -> Result<String> {
        Ok(self.member(ctx).await?.display_name)
    }
    /// Today in the crew's zone.
    pub async fn today(&self, ctx: &SecurityContext) -> Result<String> {
        let crew = self.crews.crew(ctx).await.map_err(|_| Error::Forbidden)?;
        Ok(today_in(&crew.timezone))
    }
    pub async fn read(&self, ctx: &SecurityContext) -> Result<(i64, State)> {
        let scope = self.scope(ctx).await?;
        self.store.ensure(&scope).await?;
        self.store.read(&scope).await
    }
    pub async fn mutate<T>(
        &self,
        ctx: &SecurityContext,
        photos: &[Photo],
        f: impl Fn(&mut State) -> Result<T>,
    ) -> Result<T> {
        let scope = self.scope(ctx).await?;
        self.store.ensure(&scope).await?;
        for _ in 0..8 {
            let (revision, mut state) = self.store.read(&scope).await?;
            let before = state.clone();
            let result = f(&mut state)?;
            if photos.is_empty() && state == before {
                return Ok(result);
            }
            if self.store.commit(&scope, revision, &state, photos).await? {
                return Ok(result);
            }
        }
        Err(Error::Conflict)
    }
    pub async fn command(&self, ctx: &SecurityContext, command: &Command) -> Result<()> {
        let actor = self.actor(ctx).await?;
        let fingerprint = self.fingerprint.of(&command.action);
        self.mutate(ctx, &[], |state| {
            if let Some(previous) = state.operations.get(&command.operation_id.to_string()) {
                return if previous == &fingerprint {
                    Ok(())
                } else {
                    Err(Error::Conflict)
                };
            }
            state.apply(&command.action, &actor)?;
            state
                .operations
                .insert(command.operation_id.to_string(), fingerprint.clone());
            Ok(())
        })
        .await
    }
    pub async fn upload(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        digest: String,
        photos: Vec<Photo>,
    ) -> Result<String> {
        let member = self.member(ctx).await?;
        let actor = member.display_name.clone();
        let today = self.today(ctx).await?;
        let now = Utc::now();
        self.mutate(ctx, &photos, |state| {
            if let Some(existing) = state.receipts.iter().find(|r| r.id == id.to_string()) {
                return if existing.digest == digest {
                    Ok(existing.id.clone())
                } else {
                    Err(Error::Conflict)
                };
            }
            // Exact image retries never create another expense, even with a new operation ID.
            if let Some(existing) = state
                .receipts
                .iter()
                .find(|r| r.digest == digest && r.status != "deleted")
            {
                return Ok(existing.id.clone());
            }
            if state
                .receipts
                .iter()
                .filter(|r| r.status == "queued" || r.status == "processing")
                .count()
                >= 30
            {
                return Err(Error::Invalid(
                    "30 receipts are already waiting. Give the scanner a minute.".into(),
                ));
            }
            state.receipts.push(Receipt {
                id: id.to_string(),
                merchant: "New receipt".into(),
                date: today.clone(),
                date_missing: false,
                time: None,
                registered_by: actor.clone(),
                registered_account: Some(member.uid.clone()),
                currency: "EUR".into(),
                total: 0,
                items: vec![],
                status: "queued".into(),
                revision: 1,
                created_at: now,
                photo_expires_at: now + chrono::Duration::days(90),
                matched_ids: vec![],
                issues: vec![],
                eligible: state.list.iter().filter(|i| !i.done).cloned().collect(),
                digest: digest.clone(),
                attempts: 0,
                next_attempt_at: None,
                lease: None,
                lease_until: None,
                suspected_duplicate: None,
            });
            Ok(id.to_string())
        })
        .await
    }
    /// Tells the rest of the crew that a receipt was scanned. Best effort:
    /// a failed push never fails the receipt.
    pub(super) async fn announce(&self, ctx: &SecurityContext, receipt_id: &str) {
        let Some(routines) = &self.routines else {
            return;
        };
        let Ok((_, state)) = self.read(ctx).await else {
            return;
        };
        let Some(receipt) = state.receipts.iter().find(|r| r.id == receipt_id) else {
            return;
        };
        let body = match receipt.status.as_str() {
            "saved" => format!(
                "{} scanned a receipt from {}: {}.{:02} {}",
                receipt.registered_by,
                receipt.merchant,
                receipt.total / 100,
                (receipt.total % 100).abs(),
                receipt.currency
            ),
            "review" => format!(
                "{}'s receipt from {} needs a quick check",
                receipt.registered_by, receipt.merchant
            ),
            _ => return,
        };
        let message = Notification {
            title: "crew-juggle · shopping".into(),
            body,
            tag: format!("shopping/receipt/{receipt_id}"),
            url: "/shopping?tab=history".into(),
            kind: NotificationKind::Shopping,
            ttl_seconds: 3600,
        };
        if let Err(error) = routines
            .notify_crew(ctx, message, receipt.registered_account.clone())
            .await
        {
            tracing::debug!(%error, "receipt notification not sent");
        }
    }

    /// The crew's state with what the shopping page shows next to it:
    /// its revision, whether scanning works, and today's date.
    pub async fn overview(&self, ctx: &SecurityContext) -> Result<(i64, State, bool, String)> {
        self.actor(ctx).await?;
        let (revision, state) = self.read(ctx).await?;
        Ok((
            revision,
            state,
            self.model.configured(),
            self.today(ctx).await?,
        ))
    }

    /// One page of up to 200 ledger events after `after`, optionally only for
    /// one receipt, and the cursor of the next page.
    pub async fn ledger_page(
        &self,
        ctx: &SecurityContext,
        receipt_id: Option<&str>,
        after: Option<&str>,
    ) -> Result<(Vec<Event>, Option<String>)> {
        self.actor(ctx).await?;
        let (_, state) = self.read(ctx).await?;
        let start = match after {
            Some(id) => state
                .events
                .iter()
                .position(|e| e.id == id)
                .map(|i| i + 1)
                .ok_or_else(|| Error::Invalid("Invalid journal cursor".into()))?,
            None => 0,
        };
        let mut events: Vec<Event> = state
            .events
            .into_iter()
            .skip(start)
            .filter(|e| receipt_id.is_none_or(|id| e.receipt_id == id))
            .take(201)
            .collect();
        let more = events.len() > 200;
        events.truncate(200);
        let next = if more {
            events.last().map(|e| e.id.clone())
        } else {
            None
        };
        Ok((events, next))
    }

    /// Stores up to five photos of one receipt and queues it for scanning.
    pub async fn upload_photos(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        uploads: Vec<Vec<u8>>,
    ) -> Result<String> {
        self.actor(ctx).await?;
        self.scope(ctx).await?;
        if !self.model.configured() {
            return Err(Error::Invalid(
                "Receipt scanning is not set up. Set OPENAI_API_KEY and OPENAI_MODEL on the server.".into(),
            ));
        }
        if uploads.is_empty() {
            return Err(Error::Invalid("Add a photo of the receipt".into()));
        }
        let mut images = Vec::with_capacity(uploads.len());
        for bytes in uploads {
            images.push(self.photos.normalize(bytes).await?);
        }
        // Same photos, same digest: a retried upload never books twice.
        let mut digest = aws_lc_rs::digest::Context::new(&aws_lc_rs::digest::SHA256);
        for bytes in &images {
            digest.update(&(bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        }
        let digest = base64::engine::general_purpose::STANDARD.encode(digest.finish().as_ref());
        let expires_at = Utc::now() + chrono::Duration::days(90);
        let photos = images
            .into_iter()
            .enumerate()
            .map(|(ordinal, bytes)| Photo {
                receipt_id: id.to_string(),
                ordinal: ordinal as i32,
                bytes,
                expires_at,
            })
            .collect();
        self.upload(ctx, id, digest, photos).await
    }

    /// A receipt's photos, or `None` once they have expired.
    pub async fn receipt_photos(
        &self,
        ctx: &SecurityContext,
        receipt_id: &str,
    ) -> Result<Option<Vec<Vec<u8>>>> {
        self.actor(ctx).await?;
        let (_, state) = self.read(ctx).await?;
        let receipt = state
            .receipts
            .iter()
            .find(|r| r.id == receipt_id && r.status != "deleted")
            .ok_or(Error::NotFound)?;
        let now = Utc::now();
        if receipt.photo_expires_at <= now {
            return Ok(None);
        }
        let scope = self.scope(ctx).await?;
        Ok(Some(self.store.photos(&scope, receipt_id, now).await?))
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;

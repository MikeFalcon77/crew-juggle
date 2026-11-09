//! Fixtures shared by the shopping tests: a receipt draft, a model reading,
//! an in-memory store and a one-crew service.
use crate::domain::*;
use crate::infra::storage::codec;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use toolkit_security::AccessScope;
use uuid::Uuid;

pub(crate) const ZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

pub(crate) fn extracted(total: i64) -> Extraction {
    crate::infra::openai::extraction_from_json(serde_json::json!({"merchant":"Corner Market","date":"2026-10-31","time":"18:42","currency":"EUR","total_minor":total,"adjustments_minor":0,"restaurant":false,"needs_review":false,"issues":[],"items":[{"name":"Milk","translated_name":"Milk Meadow 3%, 1 l","raw_name":"FRESH MILK MEADOW 3% 1L","product_id":null,"variant":"Meadow 3%, 1 l","category":"food","subcategory":"Dairy & eggs","quantity":2,"unit":"pc","amount_minor":338,"uncertain":false}]})).unwrap()
}
pub(crate) fn draft() -> Receipt {
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-31T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    Receipt {
        id: Uuid::new_v4().to_string(),
        merchant: "New receipt".into(),
        date: today_in("Europe/Berlin"),
        date_missing: false,
        time: None,
        registered_by: "Pip".into(),
        registered_account: None,
        currency: "EUR".into(),
        total: 0,
        items: vec![],
        status: "queued".into(),
        revision: 1,
        created_at: now,
        photo_expires_at: now + chrono::Duration::days(90),
        matched_ids: vec![],
        issues: vec![],
        eligible: vec![],
        digest: "fixture".into(),
        attempts: 0,
        next_attempt_at: None,
        lease: None,
        lease_until: None,
        suspected_duplicate: None,
    }
}
/// One crew's state in memory; the photo store keeps uploads for the scanner.
pub(crate) struct Memory {
    pub(crate) state: Mutex<(i64, State)>,
    photos: Mutex<Vec<Photo>>,
}
impl Memory {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new((1, State::default())),
            photos: Mutex::new(vec![]),
        }
    }
}
#[async_trait::async_trait]
impl Store for Memory {
    async fn ensure(&self, _: &AccessScope) -> Result<()> {
        Ok(())
    }
    async fn read(&self, _: &AccessScope) -> Result<(i64, State)> {
        Ok(self.state.lock().unwrap().clone())
    }
    async fn commit(
        &self,
        _: &AccessScope,
        revision: i64,
        state: &State,
        photos: &[Photo],
    ) -> Result<bool> {
        let mut saved = self.state.lock().unwrap();
        if saved.0 != revision {
            return Ok(false);
        }
        *saved = (revision + 1, state.clone());
        self.photos.lock().unwrap().extend(photos.iter().cloned());
        Ok(true)
    }
    async fn photos(
        &self,
        _: &AccessScope,
        receipt: &str,
        _: chrono::DateTime<Utc>,
    ) -> Result<Vec<Vec<u8>>> {
        Ok(self
            .photos
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.receipt_id == receipt)
            .map(|p| p.bytes.clone())
            .collect())
    }
    async fn expire(&self, _: &AccessScope, _: chrono::DateTime<Utc>) -> Result<()> {
        Ok(())
    }
}

pub(crate) fn crew_service(store: Arc<Memory>) -> (Service, juggle_testing::FixedCrew) {
    let crew = Uuid::new_v4();
    let members = [("demo-pip", "Pip"), ("demo-zed", "Zed")];
    let fixed = juggle_testing::FixedCrew::new(crew, &members);
    let directory = Arc::new(juggle_testing::FixedCrew::new(crew, &members));
    (
        Service {
            store,
            model: Arc::new(crate::infra::demo_scanner::DemoScanner),
            fingerprint: Arc::new(codec::JsonFingerprint),
            photos: Arc::new(crate::infra::photos::JpegNormalizer),
            policy: directory.policy(),
            crews: directory,
            routines: None,
        },
        fixed,
    )
}

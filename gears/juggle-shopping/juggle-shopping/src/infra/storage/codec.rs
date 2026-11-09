//! The stored JSON shapes of the shopping state and ledger events.
//!
//! `js_state.payload` and `js_ledger.payload` have kept these shapes since
//! v0.1: field names, defaults and skipped fields must not change, or stored
//! crews stop loading. The domain types carry no serde; these mirrors do.
use crate::domain;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub name: String,
    pub category: String,
    pub subcategory: String,
    pub aliases: Vec<String>,
    pub variant: String,
    pub revision: u64,
    #[serde(default)]
    pub classified: bool,
    #[serde(default)]
    pub category_override: bool,
}
impl From<domain::Product> for Product {
    fn from(value: domain::Product) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category: value.category,
            subcategory: value.subcategory,
            aliases: value.aliases,
            variant: value.variant,
            revision: value.revision,
            classified: value.classified,
            category_override: value.category_override,
        }
    }
}
impl From<Product> for domain::Product {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category: value.category,
            subcategory: value.subcategory,
            aliases: value.aliases,
            variant: value.variant,
            revision: value.revision,
            classified: value.classified,
            category_override: value.category_override,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub id: String,
    pub product_id: Option<String>,
    pub name: String,
    pub quantity: String,
    pub category: String,
    pub done: bool,
    pub author: String,
    pub revision: u64,
}
impl From<domain::ListItem> for ListItem {
    fn from(value: domain::ListItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            name: value.name,
            quantity: value.quantity,
            category: value.category,
            done: value.done,
            author: value.author,
            revision: value.revision,
        }
    }
}
impl From<ListItem> for domain::ListItem {
    fn from(value: ListItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            name: value.name,
            quantity: value.quantity,
            category: value.category,
            done: value.done,
            author: value.author,
            revision: value.revision,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptItem {
    pub id: String,
    pub product_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    pub name: String,
    pub raw_name: String,
    pub quantity: f64,
    pub unit: String,
    pub amount: i64,
    pub category: String,
    pub subcategory: String,
    pub uncertain: bool,
}
impl From<domain::ReceiptItem> for ReceiptItem {
    fn from(value: domain::ReceiptItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            product_name: value.product_name,
            name: value.name,
            raw_name: value.raw_name,
            quantity: value.quantity,
            unit: value.unit,
            amount: value.amount,
            category: value.category,
            subcategory: value.subcategory,
            uncertain: value.uncertain,
        }
    }
}
impl From<ReceiptItem> for domain::ReceiptItem {
    fn from(value: ReceiptItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            product_name: value.product_name,
            name: value.name,
            raw_name: value.raw_name,
            quantity: value.quantity,
            unit: value.unit,
            amount: value.amount,
            category: value.category,
            subcategory: value.subcategory,
            uncertain: value.uncertain,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub id: String,
    pub merchant: String,
    pub date: String,
    #[serde(default)]
    pub date_missing: bool,
    pub currency: String,
    pub total: i64,
    pub items: Vec<ReceiptItem>,
    pub status: String,
    pub revision: u64,
    pub created_at: DateTime<Utc>,
    pub photo_expires_at: DateTime<Utc>,
    pub matched_ids: Vec<String>,
    pub issues: Vec<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub registered_by: String,
    /// Account of whoever uploaded it, so the crew notification can skip them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_account: Option<String>,
    #[serde(default)]
    pub eligible: Vec<ListItem>,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub next_attempt_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub lease: Option<String>,
    #[serde(default)]
    pub lease_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub suspected_duplicate: Option<String>,
}
impl From<domain::Receipt> for Receipt {
    fn from(value: domain::Receipt) -> Self {
        Self {
            id: value.id,
            merchant: value.merchant,
            date: value.date,
            date_missing: value.date_missing,
            currency: value.currency,
            total: value.total,
            items: value.items.into_iter().map(Into::into).collect(),
            status: value.status,
            revision: value.revision,
            created_at: value.created_at,
            photo_expires_at: value.photo_expires_at,
            matched_ids: value.matched_ids,
            issues: value.issues,
            time: value.time,
            registered_by: value.registered_by,
            registered_account: value.registered_account,
            eligible: value.eligible.into_iter().map(Into::into).collect(),
            digest: value.digest,
            attempts: value.attempts,
            next_attempt_at: value.next_attempt_at,
            lease: value.lease,
            lease_until: value.lease_until,
            suspected_duplicate: value.suspected_duplicate,
        }
    }
}
impl From<Receipt> for domain::Receipt {
    fn from(value: Receipt) -> Self {
        Self {
            id: value.id,
            merchant: value.merchant,
            date: value.date,
            date_missing: value.date_missing,
            currency: value.currency,
            total: value.total,
            items: value.items.into_iter().map(Into::into).collect(),
            status: value.status,
            revision: value.revision,
            created_at: value.created_at,
            photo_expires_at: value.photo_expires_at,
            matched_ids: value.matched_ids,
            issues: value.issues,
            time: value.time,
            registered_by: value.registered_by,
            registered_account: value.registered_account,
            eligible: value.eligible.into_iter().map(Into::into).collect(),
            digest: value.digest,
            attempts: value.attempts,
            next_attempt_at: value.next_attempt_at,
            lease: value.lease,
            lease_until: value.lease_until,
            suspected_duplicate: value.suspected_duplicate,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub id: String,
    pub text: String,
    pub status: String,
    pub error: Option<String>,
    pub author: String,
    pub lease: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
}
impl From<domain::Import> for Import {
    fn from(value: domain::Import) -> Self {
        Self {
            id: value.id,
            text: value.text,
            status: value.status,
            error: value.error,
            author: value.author,
            lease: value.lease,
            lease_until: value.lease_until,
        }
    }
}
impl From<Import> for domain::Import {
    fn from(value: Import) -> Self {
        Self {
            id: value.id,
            text: value.text,
            status: value.status,
            error: value.error,
            author: value.author,
            lease: value.lease,
            lease_until: value.lease_until,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectionRule {
    pub id: String,
    pub merchant: String,
    pub originals: Vec<String>,
    pub name: String,
    pub product_id: String,
    pub product_name: String,
    pub category: String,
    pub subcategory: String,
    pub revision: u64,
}
impl From<domain::CorrectionRule> for CorrectionRule {
    fn from(value: domain::CorrectionRule) -> Self {
        Self {
            id: value.id,
            merchant: value.merchant,
            originals: value.originals,
            name: value.name,
            product_id: value.product_id,
            product_name: value.product_name,
            category: value.category,
            subcategory: value.subcategory,
            revision: value.revision,
        }
    }
}
impl From<CorrectionRule> for domain::CorrectionRule {
    fn from(value: CorrectionRule) -> Self {
        Self {
            id: value.id,
            merchant: value.merchant,
            originals: value.originals,
            name: value.name,
            product_id: value.product_id,
            product_name: value.product_name,
            category: value.category,
            subcategory: value.subcategory,
            revision: value.revision,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub list: Vec<ListItem>,
    pub catalog: Vec<Product>,
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub imports: Vec<Import>,
    #[serde(default)]
    pub rules: Vec<CorrectionRule>,
    #[serde(default)]
    pub operations: BTreeMap<String, String>,
    #[serde(default)]
    pub classification_next_at: Option<DateTime<Utc>>,
}
impl From<domain::State> for State {
    fn from(value: domain::State) -> Self {
        Self {
            list: value.list.into_iter().map(Into::into).collect(),
            catalog: value.catalog.into_iter().map(Into::into).collect(),
            receipts: value.receipts.into_iter().map(Into::into).collect(),
            imports: value.imports.into_iter().map(Into::into).collect(),
            rules: value.rules.into_iter().map(Into::into).collect(),
            operations: value.operations,
            classification_next_at: value.classification_next_at,
        }
    }
}
impl From<State> for domain::State {
    fn from(value: State) -> Self {
        Self {
            list: value.list.into_iter().map(Into::into).collect(),
            catalog: value.catalog.into_iter().map(Into::into).collect(),
            receipts: value.receipts.into_iter().map(Into::into).collect(),
            imports: value.imports.into_iter().map(Into::into).collect(),
            rules: value.rules.into_iter().map(Into::into).collect(),
            operations: value.operations,
            classification_next_at: value.classification_next_at,
            events: Default::default(),
            persisted_event_count: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: String,
}
impl From<domain::NewItem> for NewItem {
    fn from(value: domain::NewItem) -> Self {
        Self {
            id: value.id,
            name: value.name,
            quantity: value.quantity,
        }
    }
}
impl From<NewItem> for domain::NewItem {
    fn from(value: NewItem) -> Self {
        Self {
            id: value.id,
            name: value.name,
            quantity: value.quantity,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Posting {
    pub account: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub item_id: Option<String>,
    pub currency: String,
    pub amount: i64,
}
impl From<domain::ledger::Posting> for Posting {
    fn from(value: domain::ledger::Posting) -> Self {
        Self {
            account: value.account,
            category: value.category,
            subcategory: value.subcategory,
            item_id: value.item_id,
            currency: value.currency,
            amount: value.amount,
        }
    }
}
impl From<Posting> for domain::ledger::Posting {
    fn from(value: Posting) -> Self {
        Self {
            account: value.account,
            category: value.category,
            subcategory: value.subcategory,
            item_id: value.item_id,
            currency: value.currency,
            amount: value.amount,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub id: String,
    pub event_id: String,
    pub merchant: String,
    pub date: String,
    pub time: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub actor: String,
    pub currency: String,
    pub total: i64,
    pub items: Vec<ReceiptItem>,
    pub status: String,
}
impl From<domain::ledger::Purchase> for Purchase {
    fn from(value: domain::ledger::Purchase) -> Self {
        Self {
            id: value.id,
            event_id: value.event_id,
            merchant: value.merchant,
            date: value.date,
            time: value.time,
            recorded_at: value.recorded_at,
            actor: value.actor,
            currency: value.currency,
            total: value.total,
            items: value.items.into_iter().map(Into::into).collect(),
            status: value.status,
        }
    }
}
impl From<Purchase> for domain::ledger::Purchase {
    fn from(value: Purchase) -> Self {
        Self {
            id: value.id,
            event_id: value.event_id,
            merchant: value.merchant,
            date: value.date,
            time: value.time,
            recorded_at: value.recorded_at,
            actor: value.actor,
            currency: value.currency,
            total: value.total,
            items: value.items.into_iter().map(Into::into).collect(),
            status: value.status,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub receipt_id: String,
    pub kind: String,
    pub recorded_at: DateTime<Utc>,
    pub purchase_date: String,
    pub purchase_time: Option<String>,
    pub actor: String,
    pub currency: String,
    pub reverses: Option<String>,
    pub purchase: Option<Purchase>,
    pub postings: Vec<Posting>,
}
impl From<domain::ledger::Event> for Event {
    fn from(value: domain::ledger::Event) -> Self {
        Self {
            id: value.id,
            receipt_id: value.receipt_id,
            kind: value.kind,
            recorded_at: value.recorded_at,
            purchase_date: value.purchase_date,
            purchase_time: value.purchase_time,
            actor: value.actor,
            currency: value.currency,
            reverses: value.reverses,
            purchase: value.purchase.map(Into::into),
            postings: value.postings.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<Event> for domain::ledger::Event {
    fn from(value: Event) -> Self {
        Self {
            id: value.id,
            receipt_id: value.receipt_id,
            kind: value.kind,
            recorded_at: value.recorded_at,
            purchase_date: value.purchase_date,
            purchase_time: value.purchase_time,
            actor: value.actor,
            currency: value.currency,
            reverses: value.reverses,
            purchase: value.purchase.map(Into::into),
            postings: value.postings.into_iter().map(Into::into).collect(),
        }
    }
}

/// Stored form of [`domain::Action`]; also the fingerprint of a command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Action {
    Add {
        items: Vec<NewItem>,
    },
    Mark {
        id: String,
        revision: u64,
        done: bool,
    },
    Remove {
        id: String,
        revision: u64,
    },
    Product {
        product: Product,
    },
    Rule {
        rule: CorrectionRule,
    },
    RuleDelete {
        id: String,
        revision: u64,
    },
    EditReceipt {
        receipt: Receipt,
        confirm_duplicate: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        allow_incomplete: bool,
    },
    DeleteReceipt {
        id: String,
        revision: u64,
    },
    Retry {
        id: String,
        revision: u64,
    },
    Import {
        id: Uuid,
        text: String,
    },
}
impl From<domain::Action> for Action {
    fn from(value: domain::Action) -> Self {
        use domain::Action as A;
        match value {
            A::Add { items } => Self::Add {
                items: items.into_iter().map(Into::into).collect(),
            },
            A::Mark { id, revision, done } => Self::Mark { id, revision, done },
            A::Remove { id, revision } => Self::Remove { id, revision },
            A::Product { product } => Self::Product {
                product: product.into(),
            },
            A::Rule { rule } => Self::Rule { rule: rule.into() },
            A::RuleDelete { id, revision } => Self::RuleDelete { id, revision },
            A::EditReceipt {
                receipt,
                confirm_duplicate,
                allow_incomplete,
            } => Self::EditReceipt {
                receipt: receipt.into(),
                confirm_duplicate,
                allow_incomplete,
            },
            A::DeleteReceipt { id, revision } => Self::DeleteReceipt { id, revision },
            A::Retry { id, revision } => Self::Retry { id, revision },
            A::Import { id, text } => Self::Import { id, text },
        }
    }
}
impl From<Action> for domain::Action {
    fn from(value: Action) -> Self {
        use Action as A;
        match value {
            A::Add { items } => Self::Add {
                items: items.into_iter().map(Into::into).collect(),
            },
            A::Mark { id, revision, done } => Self::Mark { id, revision, done },
            A::Remove { id, revision } => Self::Remove { id, revision },
            A::Product { product } => Self::Product {
                product: product.into(),
            },
            A::Rule { rule } => Self::Rule { rule: rule.into() },
            A::RuleDelete { id, revision } => Self::RuleDelete { id, revision },
            A::EditReceipt {
                receipt,
                confirm_duplicate,
                allow_incomplete,
            } => Self::EditReceipt {
                receipt: receipt.into(),
                confirm_duplicate,
                allow_incomplete,
            },
            A::DeleteReceipt { id, revision } => Self::DeleteReceipt { id, revision },
            A::Retry { id, revision } => Self::Retry { id, revision },
            A::Import { id, text } => Self::Import { id, text },
        }
    }
}

/// The stored JSON of a crew's state (without its ledger events).
pub fn encode_state(state: &domain::State) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(State::from(state.clone()))
}

/// A crew's state from its stored JSON. Older rows without newer fields load
/// with their defaults.
pub fn decode_state(value: serde_json::Value) -> serde_json::Result<domain::State> {
    serde_json::from_value::<State>(value).map(Into::into)
}

/// The stored JSON of one ledger event. The database trigger reads
/// `postings[].amount` and `postings[].currency` from it.
pub fn encode_event(event: &domain::ledger::Event) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(Event::from(event.clone()))
}

/// A ledger event from its stored JSON.
pub fn decode_event(value: serde_json::Value) -> serde_json::Result<domain::ledger::Event> {
    serde_json::from_value::<Event>(value).map(Into::into)
}

/// Commands are fingerprinted as their stored JSON, as they always were, so
/// an operation recorded by an older version still replays as the same one.
pub struct JsonFingerprint;

impl domain::ActionFingerprint for JsonFingerprint {
    fn of(&self, action: &domain::Action) -> String {
        serde_json::to_string(&Action::from(action.clone()))
            .expect("an action always serializes to JSON")
    }
}

#[cfg(test)]
#[path = "codec_tests.rs"]
mod codec_tests;

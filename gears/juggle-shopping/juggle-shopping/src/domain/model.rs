//! What the shopping domain works with: the list, the catalog, receipts,
//! correction rules and the crew's whole shopping state.
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use toolkit::domain_model;
use uuid::Uuid;

/// A kind of thing the crew buys, e.g. "Oat milk", with its category.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub category: String,
    pub subcategory: String,
    pub aliases: Vec<String>,
    pub variant: String,
    pub revision: u64,
    pub classified: bool,
    pub category_override: bool,
}
/// One line of the shopping list.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
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
/// One line of a scanned receipt; amounts are in minor units.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptItem {
    pub id: String,
    pub product_id: Option<String>,
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
/// An uploaded receipt, from the queue through scanning to the books.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt {
    pub id: String,
    pub merchant: String,
    pub date: String,
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
    pub time: Option<String>,
    pub registered_by: String,
    /// Account of whoever uploaded it, so the crew notification can skip them.
    pub registered_account: Option<String>,
    pub eligible: Vec<ListItem>,
    pub digest: String,
    pub attempts: u32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub lease: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
    pub suspected_duplicate: Option<String>,
}
/// A pasted shopping list waiting to be turned into items.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub id: String,
    pub text: String,
    pub status: String,
    pub error: Option<String>,
    pub author: String,
    pub lease: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
}
/// A crew's fix for one merchant's receipt line, learned from a review.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
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
/// Everything one crew has in shopping, saved as one document with a revision.
#[domain_model]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct State {
    pub list: Vec<ListItem>,
    pub catalog: Vec<Product>,
    pub receipts: Vec<Receipt>,
    pub imports: Vec<Import>,
    pub rules: Vec<CorrectionRule>,
    pub operations: BTreeMap<String, String>,
    pub events: Vec<crate::domain::ledger::Event>,
    pub persisted_event_count: usize,
    pub classification_next_at: Option<DateTime<Utc>>,
}
/// One stored photo of a receipt.
#[domain_model]
#[derive(Clone, PartialEq)]
pub struct Photo {
    pub receipt_id: String,
    pub ordinal: i32,
    pub bytes: Vec<u8>,
    pub expires_at: DateTime<Utc>,
}

/// An item to add to the list.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct NewItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: String,
}
/// What a command does.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
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
/// A change to the crew's shopping, applied once per operation id.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub operation_id: Uuid,
    pub action: Action,
}

/// A product the model read from a pasted list.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedProduct {
    pub name: String,
    pub product_id: Option<String>,
    pub category: String,
    pub subcategory: String,
    pub quantity: String,
    pub variant: String,
    pub original_name: String,
}
/// A receipt line the model read from a photo.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedItem {
    pub name: String,
    pub raw_name: String,
    pub product_id: Option<String>,
    pub variant: String,
    pub category: String,
    pub subcategory: String,
    pub quantity: f64,
    pub unit: String,
    pub amount_minor: i64,
    pub uncertain: bool,
    pub translated_name: String,
}
/// What the model read from a receipt.
#[domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Extraction {
    pub merchant: Option<String>,
    pub date: Option<String>,
    pub currency: Option<String>,
    pub total_minor: Option<i64>,
    pub adjustments_minor: i64,
    pub restaurant: bool,
    pub items: Vec<ExtractedItem>,
    pub needs_review: bool,
    pub issues: Vec<String>,
    pub time: Option<String>,
}

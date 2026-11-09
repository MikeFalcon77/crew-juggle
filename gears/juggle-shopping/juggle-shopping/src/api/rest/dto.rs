//! Wire shapes of the shopping API. They match what the web app reads and
//! sends, field by field: most types are camelCase, the command envelope and
//! the ledger query are snake_case.
//!
//! The camelCase types carry `#[allow(de0803_api_snake_case)]`: the web app has
//! read these names since v0.1, and renaming them would break every client.
use crate::domain;
use chrono::{DateTime, Utc};
use uuid::Uuid;

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingProduct)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingListItem)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingReceiptItem)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingReceipt)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingCorrectionRule)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingNewItem)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingPosting)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingPurchase)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingEvent)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
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

/// What a command does; `kind` names the variant.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(as = ShoppingAction)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Action {
    /// Add items to the list.
    Add {
        /// The new items.
        items: Vec<NewItem>,
    },
    /// Tick an item off, or back on.
    Mark {
        /// List item id.
        id: String,
        /// Revision the client last saw.
        revision: u64,
        /// Bought or not.
        done: bool,
    },
    /// Remove an item from the list.
    Remove {
        /// List item id.
        id: String,
        /// Revision the client last saw.
        revision: u64,
    },
    /// Rename or recategorize a product.
    Product {
        /// The product as it should be.
        product: Product,
    },
    /// Add or change a correction rule.
    Rule {
        /// The rule as it should be.
        rule: CorrectionRule,
    },
    /// Delete a correction rule.
    RuleDelete {
        /// Rule id.
        id: String,
        /// Revision the client last saw.
        revision: u64,
    },
    /// Save a reviewed receipt.
    EditReceipt {
        /// The receipt as it should be.
        receipt: Receipt,
        /// Book it even though it looks like a duplicate.
        confirm_duplicate: bool,
        /// Save a partial review without booking it.
        #[serde(default, skip_serializing_if = "is_false")]
        allow_incomplete: bool,
    },
    /// Delete a receipt; its purchase is reversed.
    DeleteReceipt {
        /// Receipt id.
        id: String,
        /// Revision the client last saw.
        revision: u64,
    },
    /// Scan a failed receipt again.
    Retry {
        /// Receipt id.
        id: String,
        /// Revision the client last saw.
        revision: u64,
    },
    /// Import a pasted shopping list.
    Import {
        /// Client-generated import id.
        id: Uuid,
        /// The pasted text.
        text: String,
    },
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

/// A change to the crew's shopping, applied once per `operation_id`.
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct CommandRequest {
    /// Client-generated id; a retried request with the same id is applied once.
    #[schema(value_type = String)]
    pub operation_id: Uuid,
    /// What to do.
    pub action: Action,
}

impl From<CommandRequest> for domain::Command {
    fn from(value: CommandRequest) -> Self {
        Self {
            operation_id: value.operation_id,
            action: value.action.into(),
        }
    }
}

/// A receipt as the app shows it; scanner bookkeeping stays on the server.
#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(as = ShoppingReceiptView)]
#[serde(rename_all = "camelCase")]
#[allow(unknown_lints, de0803_api_snake_case)]
pub struct ReceiptView {
    /// Receipt id.
    pub id: String,
    /// Store name.
    pub merchant: String,
    /// Purchase date, `YYYY-MM-DD`.
    pub date: String,
    /// The date could not be read and is a guess.
    pub date_missing: bool,
    /// ISO 4217 currency.
    pub currency: String,
    /// Total in minor units.
    pub total: i64,
    /// The receipt lines.
    pub items: Vec<ReceiptItem>,
    /// `queued`, `processing`, `saved`, `review` or `failed`.
    pub status: String,
    /// Incremented on every change.
    pub revision: u64,
    /// When it was uploaded.
    pub created_at: DateTime<Utc>,
    /// When its photos are deleted.
    pub photo_expires_at: DateTime<Utc>,
    /// List items the purchase ticked off.
    pub matched_ids: Vec<String>,
    /// What needs a human look.
    pub issues: Vec<String>,
    /// Purchase time as printed, if any.
    pub time: Option<String>,
    /// Who uploaded it.
    pub registered_by: String,
    /// Account of the uploader.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_account: Option<String>,
    /// Receipt this one may duplicate.
    pub suspected_duplicate: Option<String>,
}

impl From<domain::Receipt> for ReceiptView {
    fn from(r: domain::Receipt) -> Self {
        Self {
            id: r.id,
            merchant: r.merchant,
            date: r.date,
            date_missing: r.date_missing,
            currency: r.currency,
            total: r.total,
            items: r.items.into_iter().map(Into::into).collect(),
            status: r.status,
            revision: r.revision,
            created_at: r.created_at,
            photo_expires_at: r.photo_expires_at,
            matched_ids: r.matched_ids,
            issues: r.issues,
            time: r.time,
            registered_by: r.registered_by,
            registered_account: r.registered_account,
            suspected_duplicate: r.suspected_duplicate,
        }
    }
}

/// A pasted list that is still being imported, or that failed.
#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(as = ShoppingImportView)]
pub struct ImportView {
    /// Import id.
    pub id: String,
    /// The pasted text.
    pub text: String,
    /// `queued`, `processing` or `failed`.
    pub status: String,
    /// Why it failed.
    pub error: Option<String>,
}

impl From<domain::Import> for ImportView {
    fn from(i: domain::Import) -> Self {
        Self {
            id: i.id,
            text: i.text,
            status: i.status,
            error: i.error,
        }
    }
}

/// Everything the shopping page shows. `aiConfigured` keeps its v0.1 name.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
#[allow(unknown_lints, de0803_api_snake_case)]
pub struct StateView {
    /// Revision of the crew's shopping state.
    pub revision: i64,
    /// The shopping list.
    pub list: Vec<ListItem>,
    /// Products the crew has bought or listed.
    pub catalog: Vec<Product>,
    /// Correction rules for receipt lines.
    pub rules: Vec<CorrectionRule>,
    /// Receipts that are not deleted.
    pub receipts: Vec<ReceiptView>,
    /// Imports that are not done.
    pub imports: Vec<ImportView>,
    /// Purchases on the books.
    pub purchases: Vec<Purchase>,
    /// Whether receipts can be scanned on this server.
    #[serde(rename = "aiConfigured")]
    pub ai_configured: bool,
    /// Today in the crew's zone.
    pub today: String,
}

impl StateView {
    /// The page's view of a crew's state.
    pub fn new(revision: i64, state: domain::State, ai_configured: bool, today: String) -> Self {
        let purchases = domain::ledger::purchases(&state.events)
            .into_iter()
            .map(Into::into)
            .collect();
        Self {
            revision,
            list: state.list.into_iter().map(Into::into).collect(),
            catalog: state.catalog.into_iter().map(Into::into).collect(),
            rules: state.rules.into_iter().map(Into::into).collect(),
            receipts: state
                .receipts
                .into_iter()
                .filter(|r| r.status != "deleted")
                .map(Into::into)
                .collect(),
            imports: state
                .imports
                .into_iter()
                .filter(|i| i.status != "done")
                .map(Into::into)
                .collect(),
            purchases,
            ai_configured,
            today,
        }
    }
}

/// Which ledger events to list.
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct LedgerQuery {
    /// Only the events of this receipt.
    pub receipt_id: Option<String>,
    /// Continue after this event id.
    pub after: Option<String>,
}

/// One page of ledger events, oldest first.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct LedgerPage {
    /// Up to 200 events.
    pub events: Vec<Event>,
    /// Cursor for the next page, if there is one.
    pub next: Option<String>,
}

/// A receipt was accepted for scanning.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct UploadAccepted {
    /// The receipt id.
    pub id: String,
}

/// A receipt's photos, while they are kept.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct ReceiptPhotos {
    /// `data:image/jpeg;base64,…` URLs.
    pub photos: Vec<String>,
    /// The photos were deleted after their retention period.
    pub expired: bool,
}

/// The command was applied (or had been already).
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct CommandApplied {
    /// Always true.
    pub ok: bool,
}

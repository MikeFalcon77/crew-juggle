//! Shapes of the model's answers, and of the corrections sent to it.
use crate::domain;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedProduct {
    pub name: String,
    pub product_id: Option<String>,
    pub category: String,
    pub subcategory: String,
    pub quantity: String,
    pub variant: String,
    #[serde(default)]
    pub original_name: String,
}
impl From<domain::ExtractedProduct> for ExtractedProduct {
    fn from(value: domain::ExtractedProduct) -> Self {
        Self {
            name: value.name,
            product_id: value.product_id,
            category: value.category,
            subcategory: value.subcategory,
            quantity: value.quantity,
            variant: value.variant,
            original_name: value.original_name,
        }
    }
}
impl From<ExtractedProduct> for domain::ExtractedProduct {
    fn from(value: ExtractedProduct) -> Self {
        Self {
            name: value.name,
            product_id: value.product_id,
            category: value.category,
            subcategory: value.subcategory,
            quantity: value.quantity,
            variant: value.variant,
            original_name: value.original_name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
impl From<domain::ExtractedItem> for ExtractedItem {
    fn from(value: domain::ExtractedItem) -> Self {
        Self {
            name: value.name,
            raw_name: value.raw_name,
            product_id: value.product_id,
            variant: value.variant,
            category: value.category,
            subcategory: value.subcategory,
            quantity: value.quantity,
            unit: value.unit,
            amount_minor: value.amount_minor,
            uncertain: value.uncertain,
            translated_name: value.translated_name,
        }
    }
}
impl From<ExtractedItem> for domain::ExtractedItem {
    fn from(value: ExtractedItem) -> Self {
        Self {
            name: value.name,
            raw_name: value.raw_name,
            product_id: value.product_id,
            variant: value.variant,
            category: value.category,
            subcategory: value.subcategory,
            quantity: value.quantity,
            unit: value.unit,
            amount_minor: value.amount_minor,
            uncertain: value.uncertain,
            translated_name: value.translated_name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
impl From<domain::Extraction> for Extraction {
    fn from(value: domain::Extraction) -> Self {
        Self {
            merchant: value.merchant,
            date: value.date,
            currency: value.currency,
            total_minor: value.total_minor,
            adjustments_minor: value.adjustments_minor,
            restaurant: value.restaurant,
            items: value.items.into_iter().map(Into::into).collect(),
            needs_review: value.needs_review,
            issues: value.issues,
            time: value.time,
        }
    }
}
impl From<Extraction> for domain::Extraction {
    fn from(value: Extraction) -> Self {
        Self {
            merchant: value.merchant,
            date: value.date,
            currency: value.currency,
            total_minor: value.total_minor,
            adjustments_minor: value.adjustments_minor,
            restaurant: value.restaurant,
            items: value.items.into_iter().map(Into::into).collect(),
            needs_review: value.needs_review,
            issues: value.issues,
            time: value.time,
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

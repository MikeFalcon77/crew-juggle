//! Append-only financial facts. UI totals are projections of postings, never receipt drafts.
use crate::domain::{Error, Receipt, ReceiptItem, Result};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use uuid::Uuid;

/// One side of a balanced ledger entry.
#[toolkit::domain_model]
#[derive(Clone, Debug, PartialEq)]
pub struct Posting {
    pub account: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub item_id: Option<String>,
    pub currency: String,
    pub amount: i64,
}
/// A booked purchase as the shopping history shows it.
#[toolkit::domain_model]
#[derive(Clone, Debug, PartialEq)]
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
/// An append-only ledger event: a purchase recorded or reversed.
#[toolkit::domain_model]
#[derive(Clone, Debug, PartialEq)]
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
impl Event {
    pub fn recorded(receipt: &Receipt, actor: &str) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let mut postings: Vec<_> = receipt
            .items
            .iter()
            .map(|i| Posting {
                account: "expense".into(),
                category: Some(i.category.clone()),
                subcategory: Some(i.subcategory.clone()),
                item_id: Some(i.id.clone()),
                currency: receipt.currency.clone(),
                amount: i.amount,
            })
            .collect();
        // We do not invent a bank/card account. This is the explicit unallocated payment counter-account.
        postings.push(Posting {
            account: "payment:unallocated".into(),
            category: None,
            subcategory: None,
            item_id: None,
            currency: receipt.currency.clone(),
            amount: -receipt.total,
        });
        let event = Self {
            id: id.clone(),
            receipt_id: receipt.id.clone(),
            kind: "purchase_recorded".into(),
            recorded_at: now,
            purchase_date: receipt.date.clone(),
            purchase_time: receipt.time.clone(),
            actor: actor.into(),
            currency: receipt.currency.clone(),
            reverses: None,
            purchase: Some(Purchase {
                id: receipt.id.clone(),
                event_id: id,
                merchant: receipt.merchant.clone(),
                date: receipt.date.clone(),
                time: receipt.time.clone(),
                recorded_at: now,
                actor: actor.into(),
                currency: receipt.currency.clone(),
                total: receipt.total,
                items: receipt.items.clone(),
                status: "saved".into(),
            }),
            postings,
        };
        event.validate()?;
        Ok(event)
    }
    pub fn reversed(&self, actor: &str) -> Result<Self> {
        let event = Self {
            id: Uuid::new_v4().to_string(),
            receipt_id: self.receipt_id.clone(),
            kind: "purchase_reversed".into(),
            recorded_at: Utc::now(),
            purchase_date: self.purchase_date.clone(),
            purchase_time: self.purchase_time.clone(),
            actor: actor.into(),
            currency: self.currency.clone(),
            reverses: Some(self.id.clone()),
            purchase: None,
            postings: self
                .postings
                .iter()
                .map(|p| Posting {
                    amount: -p.amount,
                    ..p.clone()
                })
                .collect(),
        };
        event.validate()?;
        Ok(event)
    }
    pub fn validate(&self) -> Result<()> {
        if self.postings.len() < 2
            || self
                .postings
                .iter()
                .any(|p| p.currency != self.currency || p.amount.unsigned_abs() > 100_000_000)
            || self
                .postings
                .iter()
                .map(|p| i128::from(p.amount))
                .sum::<i128>()
                != 0
        {
            return Err(Error::Invalid("Purchase postings do not balance".into()));
        }
        Ok(())
    }
}
pub fn purchases(events: &[Event]) -> Vec<Purchase> {
    let reversed: HashSet<_> = events
        .iter()
        .filter_map(|e| e.reverses.as_deref())
        .collect();
    events
        .iter()
        .filter(|e| !reversed.contains(e.id.as_str()))
        .filter_map(|e| {
            let mut purchase = e.purchase.clone()?;
            purchase.total = e
                .postings
                .iter()
                .filter(|p| p.account == "expense")
                .map(|p| p.amount)
                .sum();
            for item in &mut purchase.items {
                if let Some(posting) = e
                    .postings
                    .iter()
                    .find(|p| p.account == "expense" && p.item_id.as_deref() == Some(&item.id))
                {
                    item.amount = posting.amount;
                    if let Some(category) = &posting.category {
                        item.category = category.clone();
                    }
                    if let Some(subcategory) = &posting.subcategory {
                        item.subcategory = subcategory.clone();
                    }
                }
            }
            Some(purchase)
        })
        .collect()
}
pub fn active<'a>(events: &'a [Event], receipt_id: &str) -> Option<&'a Event> {
    let reversed: HashSet<_> = events
        .iter()
        .filter_map(|e| e.reverses.as_deref())
        .collect();
    events.iter().rev().find(|e| {
        e.receipt_id == receipt_id
            && e.kind == "purchase_recorded"
            && !reversed.contains(e.id.as_str())
    })
}

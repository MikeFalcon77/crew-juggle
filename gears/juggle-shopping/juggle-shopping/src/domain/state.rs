//! The rules of one crew's shopping state: the list, the catalog, receipts,
//! corrections and the purchase ledger.
use super::error::{Error, Result};
use super::model::*;
use super::rules::*;
use chrono::{Datelike, NaiveDate, Utc};
use uuid::Uuid;

impl State {
    pub(crate) fn reconcile_classification(
        &mut self,
        batch: &[Product],
        classified: &[ExtractedProduct],
    ) {
        for result in classified {
            if text(&result.name, 200).is_err()
                || !category_valid(&result.category, &result.subcategory)
            {
                continue;
            }
            let source = self
                .catalog
                .iter()
                .find(|p| {
                    batch
                        .iter()
                        .any(|b| b.id == p.id && b.revision == p.revision)
                        && (normalized(&result.original_name) == normalized(&p.name)
                            || result.product_id.as_deref() == Some(&p.id)
                            || normalized(&result.name) == normalized(&p.name))
                })
                .cloned();
            let Some(source) = source else {
                continue;
            };
            let target = self
                .catalog
                .iter()
                .find(|p| p.id != source.id && normalized(&p.name) == normalized(&result.name))
                .cloned();
            let referenced = self
                .events
                .iter()
                .filter_map(|e| e.purchase.as_ref())
                .flat_map(|p| &p.items)
                .any(|i| i.product_id.as_deref() == Some(&source.id));
            let id = if let Some(target) = target.filter(|_| !referenced) {
                for p in &mut self.catalog {
                    if p.id == target.id && p.aliases.len() < 30 {
                        p.aliases.push(source.name.clone());
                        p.revision += 1;
                    }
                }
                self.catalog.retain(|p| p.id != source.id);
                for r in &mut self.receipts {
                    for i in &mut r.items {
                        if i.product_id.as_deref() == Some(&source.id) {
                            i.product_id = Some(target.id.clone());
                        }
                    }
                }
                target.id
            } else {
                if let Some(p) = self.catalog.iter_mut().find(|p| p.id == source.id) {
                    if p.aliases.len() < 30 && p.name != result.name {
                        p.aliases.push(p.name.clone());
                    }
                    p.name = result.name.clone();
                    if !p.category_override {
                        p.category = result.category.clone();
                        p.subcategory = result.subcategory.clone();
                    }
                    p.classified = true;
                    p.revision += 1;
                }
                source.id.clone()
            };
            if let Some(product) = self.catalog.iter().find(|p| p.id == id) {
                for item in &mut self.list {
                    if item.product_id.as_deref() == Some(&source.id) {
                        item.product_id = Some(id.clone());
                        item.name = product.name.clone();
                        item.category = product.category.clone();
                        item.revision += 1;
                    }
                }
            }
        }
    }
    fn validate_rule(&self, rule: &CorrectionRule, excluding: Option<&str>) -> Result<()> {
        text(&rule.merchant, 200)?;
        text(&rule.name, 200)?;
        text(&rule.product_name, 200)?;
        if rule.originals.is_empty()
            || rule.originals.len() > 8
            || !category_valid(&rule.category, &rule.subcategory)
            || !self.catalog.iter().any(|p| {
                p.id == rule.product_id && normalized(&p.name) == normalized(&rule.product_name)
            })
        {
            return Err(Error::Invalid(
                "Check the item and category of this rule".into(),
            ));
        }
        for original in &rule.originals {
            text(original, 300)?;
            if self.rules.iter().any(|r| {
                Some(r.id.as_str()) != excluding
                    && normalized(&r.merchant) == normalized(&rule.merchant)
                    && r.originals
                        .iter()
                        .any(|s| normalized(s) == normalized(original))
            }) {
                return Err(Error::Invalid(
                    "There is already a rule for this receipt line".into(),
                ));
            }
        }
        Ok(())
    }
    fn learn_corrections(&mut self, before: &Receipt, after: &Receipt) -> Result<()> {
        for item in &after.items {
            if item.uncertain {
                continue;
            }
            let Some(old) = before.items.iter().find(|i| i.id == item.id) else {
                continue;
            };
            if old.raw_name == item.raw_name
                && old.name == item.name
                && old.product_id == item.product_id
                && old.category == item.category
                && old.subcategory == item.subcategory
            {
                continue;
            }
            let product = self
                .catalog
                .iter_mut()
                .find(|p| Some(&p.id) == item.product_id.as_ref())
                .ok_or(Error::NotFound)?;
            // Search must find the corrected label, even when the generic type stays the same.
            if product.aliases.len() < 30
                && !product
                    .aliases
                    .iter()
                    .any(|a| normalized(a) == normalized(&item.name))
            {
                product.aliases.insert(0, item.name.clone());
                product.revision += 1;
            }
            let product = product.clone();
            if item.raw_name.trim().is_empty() {
                continue;
            }
            let index = self.rules.iter().position(|r| {
                normalized(&r.merchant) == normalized(&after.merchant)
                    && r.originals.iter().any(|s| {
                        normalized(s) == normalized(&old.raw_name)
                            || normalized(s) == normalized(&item.raw_name)
                    })
            });
            let mut originals = vec![item.raw_name.clone()];
            for original in std::iter::once(&old.raw_name).chain(
                index
                    .into_iter()
                    .flat_map(|i| self.rules[i].originals.iter()),
            ) {
                if originals.len() < 8
                    && !originals
                        .iter()
                        .any(|s| normalized(s) == normalized(original))
                {
                    originals.push(original.clone());
                }
            }
            let rule = CorrectionRule {
                id: index
                    .map(|i| self.rules[i].id.clone())
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
                revision: index.map(|i| self.rules[i].revision + 1).unwrap_or(1),
                merchant: after.merchant.clone(),
                originals,
                name: item.name.clone(),
                product_id: product.id.clone(),
                product_name: product.name.clone(),
                category: item.category.clone(),
                subcategory: item.subcategory.clone(),
            };
            self.validate_rule(&rule, index.map(|i| self.rules[i].id.as_str()))?;
            if let Some(index) = index {
                self.rules[index] = rule;
            } else {
                if self.rules.len() >= 500 {
                    return Err(Error::Invalid(
                        "You reached 500 rules. Remove some in the catalog.".into(),
                    ));
                }
                self.rules.push(rule);
            }
        }
        Ok(())
    }
    fn product(
        &mut self,
        name: &str,
        suggested: Option<&str>,
        category: &str,
        subcategory: &str,
        variant: &str,
    ) -> Result<String> {
        text(name, 200)?;
        let normalized_name = normalized(name);
        let found = self.catalog.iter().position(|p| {
            (suggested == Some(p.id.as_str()) && normalized(&p.name) == normalized_name)
                || normalized(&p.name) == normalized_name
                || p.aliases.iter().any(|a| normalized(a) == normalized_name)
        });
        if let Some(index) = found {
            return Ok(self.catalog[index].id.clone());
        }
        let valid = category_valid(category, subcategory);
        let id = Uuid::new_v4().to_string();
        self.catalog.push(Product {
            id: id.clone(),
            name: name.trim().into(),
            category: if valid { category } else { "other" }.into(),
            subcategory: if valid { subcategory } else { "Uncategorized" }.into(),
            aliases: vec![],
            variant: variant.chars().take(300).collect(),
            revision: 1,
            classified: valid && category != "other",
            category_override: false,
        });
        Ok(id)
    }
    pub(crate) fn add_classified(
        &mut self,
        id: String,
        p: &ExtractedProduct,
        actor: &str,
    ) -> Result<()> {
        text(&p.quantity, 60)?;
        if self.list.iter().any(|i| i.id == id) {
            return Ok(());
        }
        let product_id = self.product(
            &p.name,
            p.product_id.as_deref(),
            &p.category,
            &p.subcategory,
            &p.variant,
        )?;
        if !p.original_name.trim().is_empty()
            && normalized(&p.original_name) != normalized(&p.name)
            && let Some(product) = self
                .catalog
                .iter_mut()
                .find(|product| product.id == product_id)
            && product.aliases.len() < 30
            && !product.aliases.contains(&p.original_name)
        {
            product
                .aliases
                .push(p.original_name.chars().take(200).collect());
        }
        let product = self
            .catalog
            .iter()
            .find(|p| p.id == product_id)
            .ok_or(Error::NotFound)?;
        self.list.push(ListItem {
            id,
            product_id: Some(product_id),
            name: product.name.clone(),
            quantity: p.quantity.clone(),
            category: product.category.clone(),
            done: false,
            author: actor.into(),
            revision: 1,
        });
        Ok(())
    }
    pub fn apply(&mut self, action: &Action, actor: &str) -> Result<()> {
        match action {
            Action::Add { items } => {
                if items.is_empty() || items.len() > 100 {
                    return Err(Error::Invalid("Add between 1 and 100 items".into()));
                }
                for i in items {
                    self.add_classified(
                        i.id.to_string(),
                        &ExtractedProduct {
                            name: i.name.clone(),
                            original_name: i.name.clone(),
                            quantity: i.quantity.clone(),
                            product_id: None,
                            category: "other".into(),
                            subcategory: "Uncategorized".into(),
                            variant: String::new(),
                        },
                        actor,
                    )?;
                }
            }
            Action::Mark { id, revision, done } => {
                let item = self
                    .list
                    .iter_mut()
                    .find(|i| &i.id == id)
                    .ok_or(Error::NotFound)?;
                if item.revision != *revision {
                    return Err(Error::Conflict);
                }
                item.done = *done;
                item.revision += 1;
            }
            Action::Remove { id, revision } => {
                let item = self
                    .list
                    .iter()
                    .find(|i| &i.id == id)
                    .ok_or(Error::NotFound)?;
                if item.revision != *revision {
                    return Err(Error::Conflict);
                }
                self.list.retain(|i| &i.id != id);
            }
            Action::Product { product } => {
                text(&product.name, 200)?;
                if !category_valid(&product.category, &product.subcategory) {
                    return Err(Error::Invalid("Invalid category".into()));
                }
                let existing = self
                    .catalog
                    .iter_mut()
                    .find(|p| p.id == product.id)
                    .ok_or(Error::NotFound)?;
                if existing.revision != product.revision {
                    return Err(Error::Conflict);
                }
                existing.name = product.name.trim().into();
                existing.category = product.category.clone();
                existing.subcategory = product.subcategory.clone();
                existing.category_override = true;
                existing.classified = true;
                existing.revision += 1;
                for i in &mut self.list {
                    if i.product_id.as_deref() == Some(&product.id) {
                        i.name = existing.name.clone();
                        i.category = existing.category.clone();
                        i.revision += 1;
                    }
                }
            }
            Action::Rule { rule } => {
                let index = self
                    .rules
                    .iter()
                    .position(|r| r.id == rule.id)
                    .ok_or(Error::NotFound)?;
                if self.rules[index].revision != rule.revision {
                    return Err(Error::Conflict);
                }
                let mut updated = rule.clone();
                updated.product_id = self.product(
                    &rule.product_name,
                    None,
                    &rule.category,
                    &rule.subcategory,
                    "",
                )?;
                self.validate_rule(&updated, Some(&rule.id))?;
                updated.revision += 1;
                self.rules[index] = updated;
            }
            Action::RuleDelete { id, revision } => {
                let index = self
                    .rules
                    .iter()
                    .position(|r| &r.id == id)
                    .ok_or(Error::NotFound)?;
                if self.rules[index].revision != *revision {
                    return Err(Error::Conflict);
                }
                self.rules.remove(index);
            }
            Action::EditReceipt {
                receipt,
                confirm_duplicate,
                allow_incomplete,
            } => {
                let index = self
                    .receipts
                    .iter()
                    .position(|r| r.id == receipt.id)
                    .ok_or(Error::NotFound)?;
                if self.receipts[index].revision != receipt.revision {
                    return Err(Error::Conflict);
                }
                let was_saved = self.receipts[index].status == "saved";
                let draft = *allow_incomplete && !was_saved;
                if draft {
                    validate_receipt_draft(receipt)?;
                } else {
                    validate_receipt(receipt)?;
                }
                if !draft
                    && self.receipts[index].suspected_duplicate.is_some()
                    && !confirm_duplicate
                {
                    return Err(Error::Invalid(
                        "Confirm that this is a separate purchase, or delete the duplicate receipt"
                            .into(),
                    ));
                }
                if !["saved", "review"].contains(&self.receipts[index].status.as_str()) {
                    return Err(Error::Conflict);
                }
                let mut updated = self.receipts[index].clone();
                updated.merchant = receipt.merchant.trim().into();
                updated.date_missing =
                    draft && (receipt.date_missing || receipt.date.trim().is_empty());
                if !receipt.date.trim().is_empty() {
                    updated.date = NaiveDate::parse_from_str(&receipt.date, "%Y-%m-%d")
                        .map_err(|_| Error::Invalid("Check the purchase date".into()))?
                        .to_string();
                }
                updated.time = receipt.time.clone();
                updated.currency = receipt.currency.clone();
                updated.total = receipt.total;
                updated.items = receipt.items.clone();
                for item in &mut updated.items {
                    if let Some(name) = item.product_name.take() {
                        item.product_id = Some(self.product(
                            &name,
                            None,
                            &item.category,
                            &item.subcategory,
                            &item.raw_name,
                        )?);
                    } else if !item
                        .product_id
                        .as_ref()
                        .is_some_and(|id| self.catalog.iter().any(|p| &p.id == id))
                    {
                        item.product_id = Some(self.product(
                            &item.name,
                            None,
                            &item.category,
                            &item.subcategory,
                            &item.raw_name,
                        )?);
                    }
                }
                if *confirm_duplicate {
                    updated.suspected_duplicate = None;
                }
                updated.issues = review_issues(&updated);
                if updated.suspected_duplicate.is_some() {
                    updated
                        .issues
                        .push("Confirm that this is a separate purchase, not a duplicate.".into());
                }
                let ready = updated.issues.is_empty();
                updated.status = if ready { "saved" } else { "review" }.into();
                updated.revision += 1;
                if ready && !was_saved {
                    self.match_list(&mut updated);
                }
                self.learn_corrections(&self.receipts[index].clone(), &updated)?;
                if ready {
                    self.book(&updated, actor)?;
                }
                self.receipts[index] = updated;
            }
            Action::DeleteReceipt { id, revision } => {
                let r = self
                    .receipts
                    .iter_mut()
                    .find(|r| &r.id == id)
                    .ok_or(Error::NotFound)?;
                if r.revision != *revision {
                    return Err(Error::Conflict);
                }
                r.status = "deleted".into();
                r.photo_expires_at = Utc::now();
                r.revision += 1;
                r.lease = None;
                self.reverse(id, actor)?;
            }
            Action::Retry { id, revision } => {
                let r = self
                    .receipts
                    .iter_mut()
                    .find(|r| &r.id == id)
                    .ok_or(Error::NotFound)?;
                if r.revision != *revision || r.status != "failed" {
                    return Err(Error::Conflict);
                }
                if r.photo_expires_at <= Utc::now() {
                    return Err(Error::Invalid(
                        "The photo was already deleted. Upload the receipt again.".into(),
                    ));
                }
                r.status = "queued".into();
                r.attempts = 0;
                r.next_attempt_at = None;
                r.issues.clear();
                r.revision += 1;
            }
            Action::Import { id, text: value } => {
                text(value, 10000)?;
                if self.imports.iter().any(|i| i.id == id.to_string()) {
                    return Ok(());
                }
                self.imports.push(Import {
                    id: id.to_string(),
                    text: value.clone(),
                    status: "queued".into(),
                    error: None,
                    author: actor.into(),
                    lease: None,
                    lease_until: None,
                });
            }
        }
        Ok(())
    }
    /// Applies a model's reading of a receipt. `zone` is the crew's time zone,
    /// used to tell which day the receipt was uploaded.
    pub fn finish_receipt(
        &mut self,
        index: usize,
        extraction: Extraction,
        zone: chrono_tz::Tz,
    ) -> Result<()> {
        let mut receipt = self.receipts[index].clone();
        receipt.merchant = extraction
            .merchant
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Unknown store".into());
        // Without a readable date, keep the upload date (already in the crew's zone).
        let uploaded_on = receipt.date.clone();
        receipt.date = extraction
            .date
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(|d| d.to_string())
            .unwrap_or(uploaded_on);
        receipt.currency = extraction
            .currency
            .clone()
            .filter(|s| valid_currency(s))
            .unwrap_or_else(|| "EUR".into());
        receipt.total = extraction.total_minor.unwrap_or(0);
        receipt.time = extraction
            .time
            .clone()
            .filter(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M").is_ok());
        receipt.issues = extraction
            .issues
            .iter()
            .take(20)
            .map(|s| s.chars().take(300).collect())
            .collect();
        receipt.date_missing = extraction
            .date
            .as_ref()
            .is_none_or(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_err());
        if receipt.date_missing
            && !receipt
                .issues
                .iter()
                .any(|s| s.to_lowercase().contains("date"))
        {
            receipt
                .issues
                .push("The purchase date is not visible. Enter it before saving.".into());
        }
        receipt.items.clear();
        if extraction.items.len() > 300 {
            return Err(Error::Model("This receipt has too many lines".into()));
        }
        for source in &extraction.items {
            let mut item = source.clone();
            let correction = self
                .rules
                .iter()
                .find(|r| {
                    normalized(&r.merchant) == normalized(&receipt.merchant)
                        && r.originals
                            .iter()
                            .any(|s| normalized(s) == normalized(&item.raw_name))
                })
                .cloned();
            if let Some(rule) = &correction {
                item.name = rule.product_name.clone();
                item.product_id = Some(rule.product_id.clone());
                item.translated_name = rule.name.clone();
                item.raw_name = rule.originals[0].clone();
                item.category = rule.category.clone();
                item.subcategory = rule.subcategory.clone();
            }
            if item.amount_minor.unsigned_abs() > 100_000_000
                || !item.quantity.is_finite()
                || item.quantity <= 0.0
                || item.quantity > 1_000_000.0
            {
                return Err(Error::Model(
                    "Could not read an amount or quantity reliably".into(),
                ));
            }
            let (category, subcategory) = if extraction.restaurant {
                ("restaurants", "Eating out")
            } else if category_valid(&item.category, &item.subcategory) {
                (item.category.as_str(), item.subcategory.as_str())
            } else {
                ("other", "Uncategorized")
            };
            let id = self.product(
                &item.name,
                item.product_id.as_deref(),
                category,
                subcategory,
                &item.variant,
            )?;
            let product = self
                .catalog
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or(Error::NotFound)?;
            if !item.uncertain
                && !product.aliases.contains(&item.raw_name)
                && product.aliases.len() < 30
            {
                product
                    .aliases
                    .push(item.raw_name.chars().take(200).collect());
            }
            let (category, subcategory) =
                if product.category_override && !extraction.restaurant && correction.is_none() {
                    (product.category.as_str(), product.subcategory.as_str())
                } else {
                    (category, subcategory)
                };
            receipt.items.push(ReceiptItem {
                id: Uuid::new_v4().to_string(),
                product_id: Some(id),
                product_name: None,
                name: item.translated_name.chars().take(200).collect(),
                raw_name: item.raw_name.chars().take(300).collect(),
                quantity: item.quantity,
                unit: item.unit.chars().take(30).collect(),
                amount: item.amount_minor,
                category: category.into(),
                subcategory: subcategory.into(),
                uncertain: item.uncertain,
            });
        }
        if extraction.adjustments_minor != 0 {
            allocate_adjustment(&mut receipt.items, extraction.adjustments_minor)?;
        }
        if receipt.items.iter().map(|i| i.amount).sum::<i64>() != receipt.total {
            receipt.issues.push(
                "Item amounts do not add up to the total. Check prices and discounts.".into(),
            );
        }
        let uploaded = receipt.created_at.with_timezone(&zone).date_naive();
        let unusual_year = !receipt.date_missing
            && NaiveDate::parse_from_str(&receipt.date, "%Y-%m-%d")
                .is_ok_and(|date| date.year() != uploaded.year());
        if unusual_year {
            receipt.issues.push(format!("Check the year: the receipt says {}, but it was uploaded on {}. Compare with the photo.", receipt.date, uploaded));
        }
        let mut review = extraction.needs_review
            || unusual_year
            || extraction
                .merchant
                .as_ref()
                .is_none_or(|s| s.trim().is_empty())
            || extraction
                .date
                .as_ref()
                .is_none_or(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_err())
            || extraction
                .currency
                .as_ref()
                .is_none_or(|s| !valid_currency(s))
            || extraction.total_minor.is_none()
            || receipt.items.iter().any(|i| i.uncertain)
            || validate_receipt(&receipt).is_err();
        if let Some(duplicate) = self.receipts.iter().find(|r| {
            r.id != receipt.id
                && r.status != "deleted"
                && r.status != "queued"
                && normalized(&r.merchant) == normalized(&receipt.merchant)
                && r.date == receipt.date
                && r.currency == receipt.currency
                && r.total == receipt.total
        }) {
            receipt.suspected_duplicate = Some(duplicate.id.clone());
            receipt.issues.push(
                "A similar receipt is already in history. Check it is not a duplicate.".into(),
            );
            review = true;
        }
        receipt.status = if review { "review" } else { "saved" }.into();
        if review && receipt.issues.is_empty() {
            receipt
                .issues
                .push("Check the date, currency and amounts before saving".into());
        }
        receipt.lease = None;
        receipt.lease_until = None;
        receipt.revision += 1;
        if !review {
            self.match_list(&mut receipt);
            self.book(&receipt, &receipt.registered_by)?;
        }
        self.receipts[index] = receipt;
        Ok(())
    }
    fn match_list(&mut self, receipt: &mut Receipt) {
        for current in &mut self.list {
            if current.done
                || !receipt.eligible.iter().any(|i| {
                    i.id == current.id
                        && i.revision == current.revision
                        && i.name == current.name
                        && i.quantity == current.quantity
                })
            {
                continue;
            }
            if quantity_fulfilled(current, &receipt.items) {
                current.done = true;
                current.revision += 1;
                receipt.matched_ids.push(current.id.clone());
            }
        }
    }
    /// Reverses every purchase still on the books and says whether any was.
    ///
    /// Demo mode calls this before it reseeds a crew: the ledger is append-only
    /// in the database, so old demo purchases are cancelled, never deleted.
    pub(crate) fn reverse_all(&mut self, actor: &str) -> Result<bool> {
        let before = self.events.len();
        let receipts: std::collections::BTreeSet<String> =
            self.events.iter().map(|e| e.receipt_id.clone()).collect();
        for receipt in receipts {
            self.reverse(&receipt, actor)?;
        }
        Ok(self.events.len() > before)
    }
    fn reverse(&mut self, receipt_id: &str, actor: &str) -> Result<()> {
        if let Some(event) = crate::domain::ledger::active(&self.events, receipt_id) {
            let reversal = event.reversed(actor)?;
            self.events.push(reversal);
        }
        Ok(())
    }
    fn book(&mut self, receipt: &Receipt, actor: &str) -> Result<()> {
        self.reverse(&receipt.id, actor)?;
        self.events
            .push(crate::domain::ledger::Event::recorded(receipt, actor)?);
        Ok(())
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

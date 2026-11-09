//! Small rules shared by the shopping use cases: categories, names, dates,
//! quantities and the checks a receipt must pass before it is booked.
use super::error::{Error, Result};
use super::model::{ListItem, Receipt, ReceiptItem};
use chrono::{NaiveDate, Utc};

pub const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "food",
        &[
            "Dairy & eggs",
            "Fruit & vegetables",
            "Meat & fish",
            "Bread & grains",
            "Drinks",
            "Snacks",
            "Other food",
        ],
    ),
    ("restaurants", &["Eating out"]),
    ("home", &["Cleaning", "Hygiene", "Household"]),
    ("pets", &["Pet food", "Pet supplies"]),
    ("hobbies", &["Electronics & parts", "Crafts"]),
    ("clothes", &["Clothes"]),
    ("health", &["Pharmacy", "Personal care"]),
    ("transport", &["Trips", "Fuel"]),
    ("leisure", &["Entertainment", "Books", "Sports"]),
    ("services", &["Services"]),
    ("other", &["Uncategorized"]),
];
pub fn category_valid(category: &str, sub: &str) -> bool {
    CATEGORIES
        .iter()
        .any(|(id, subs)| *id == category && subs.contains(&sub))
}
pub fn normalized(value: &str) -> String {
    value
        .to_lowercase()
        .replace('ё', "е")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
/// Today's date in the crew's zone, as `YYYY-MM-DD`.
pub fn today_in(zone: &str) -> String {
    let zone: chrono_tz::Tz = zone.parse().unwrap_or(chrono_tz::UTC);
    Utc::now().with_timezone(&zone).date_naive().to_string()
}
pub(super) fn text(value: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.chars().count() > max {
        return Err(Error::Invalid(
            "Check that every field is filled in and not too long".into(),
        ));
    }
    Ok(())
}

pub(super) fn quantity_fulfilled(list: &ListItem, items: &[ReceiptItem]) -> bool {
    let matched: Vec<_> = items
        .iter()
        .filter(|i| {
            !i.uncertain
                && i.amount >= 0
                && i.product_id.is_some()
                && i.product_id == list.product_id
        })
        .collect();
    if matched.is_empty() {
        return false;
    }
    let raw = normalized(&list.quantity).replace(' ', "");
    if ["1", "1шт", "1шт.", "1pc", "1 pc"].contains(&raw.as_str()) {
        return true;
    }
    static QUANTITY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*(кг|kg|мл|ml|л|l|г|g|шт\.?|бр\.?|pcs?\.?|packs?|уп\.?|пачк[аи]?|бут\.?)?").expect("constant quantity regex")
    });
    fn unit(value: &str) -> (&'static str, f64) {
        match value.trim_end_matches('.').to_lowercase().as_str() {
            "кг" | "kg" => ("mass", 1000.),
            "г" | "g" => ("mass", 1.),
            "л" | "l" => ("volume", 1000.),
            "мл" | "ml" => ("volume", 1.),
            "шт" | "бр" | "pc" | "pcs" | "" => ("piece", 1.),
            "уп" | "пачка" | "пачки" | "пачк" | "бут" | "pack" | "packs" => {
                ("pack", 1.)
            }
            _ => ("unknown", 1.),
        }
    }
    let Some(need) = QUANTITY.captures(&list.quantity) else {
        return false;
    };
    let wanted = need[1]
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(f64::INFINITY);
    let (kind, factor) = unit(need.get(2).map_or("", |m| m.as_str()));
    let total: f64 = matched
        .iter()
        .map(|i| {
            let (sold_kind, sold_factor) = unit(&i.unit);
            if kind == "pack" && ["piece", "pack"].contains(&sold_kind) {
                return i.quantity;
            }
            let package = QUANTITY
                .captures_iter(&i.raw_name)
                .filter(|c| c.get(2).is_some())
                .last();
            if ["piece", "pack"].contains(&sold_kind)
                && let Some(package) = package
            {
                let (pack_kind, pack_factor) = unit(package.get(2).map_or("", |m| m.as_str()));
                if kind == pack_kind {
                    return i.quantity
                        * package[1].replace(',', ".").parse::<f64>().unwrap_or(0.)
                        * pack_factor;
                }
            }
            if kind == sold_kind {
                i.quantity * sold_factor
            } else {
                0.
            }
        })
        .sum();
    total + 1e-6 >= wanted * factor
}
pub fn valid_currency(value: &str) -> bool {
    value.len() == 3 && value.bytes().all(|b| b.is_ascii_uppercase())
}
pub(super) fn validate_receipt_draft(receipt: &Receipt) -> Result<()> {
    text(&receipt.merchant, 200)?;
    if (!receipt.date.trim().is_empty()
        && NaiveDate::parse_from_str(&receipt.date, "%Y-%m-%d").is_err())
        || receipt
            .time
            .as_ref()
            .is_some_and(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M").is_err())
        || !valid_currency(&receipt.currency)
        || receipt.items.is_empty()
        || receipt.items.len() > 300
        || receipt.total.unsigned_abs() > 100_000_000
    {
        return Err(Error::Invalid("Check the date, currency and total".into()));
    }
    for item in &receipt.items {
        text(&item.name, 200)?;
        if let Some(name) = &item.product_name {
            text(name, 200)?;
        }
        if !item.quantity.is_finite()
            || item.quantity <= 0.0
            || item.quantity > 1_000_000.0
            || item.amount.unsigned_abs() > 100_000_000
            || !category_valid(&item.category, &item.subcategory)
            || item.raw_name.chars().count() > 300
        {
            return Err(Error::Invalid("Check the scanned items".into()));
        }
    }
    Ok(())
}
pub(super) fn review_issues(receipt: &Receipt) -> Vec<String> {
    let mut issues = vec![];
    if receipt.date_missing || receipt.date.trim().is_empty() {
        issues.push("Enter the purchase date; it is not visible in the photo.".into());
    }
    let sum: i64 = receipt.items.iter().map(|i| i.amount).sum();
    if sum != receipt.total {
        issues.push("Item amounts do not add up to the total. Check prices and discounts.".into());
    }
    let uncertain: Vec<_> = receipt
        .items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.uncertain)
        .map(|(n, _)| (n + 1).to_string())
        .collect();
    if !uncertain.is_empty() {
        issues.push(format!("Confirm these items: {}.", uncertain.join(", ")));
    }
    issues
}
pub fn validate_receipt(receipt: &Receipt) -> Result<()> {
    validate_receipt_draft(receipt)?;
    if let Some(issue) = review_issues(receipt).first() {
        return Err(Error::Invalid(issue.clone()));
    }
    Ok(())
}
pub(super) fn allocate_adjustment(items: &mut [ReceiptItem], adjustment: i64) -> Result<()> {
    let sum: i64 = items.iter().map(|i| i.amount.max(0)).sum();
    if sum <= 0 || adjustment.unsigned_abs() > 100_000_000 || sum + adjustment < 0 {
        return Err(Error::Model(
            "Could not spread the receipt discount over the items".into(),
        ));
    }
    let mut remaining = adjustment;
    let mut largest = 0;
    for index in 0..items.len() {
        if items[index].amount > items[largest].amount {
            largest = index;
        }
        let share = (i128::from(adjustment) * i128::from(items[index].amount.max(0))
            / i128::from(sum)) as i64;
        items[index].amount += share;
        remaining -= share;
    }
    if let Some(item) = items.get_mut(largest) {
        item.amount += remaining;
    }
    Ok(())
}

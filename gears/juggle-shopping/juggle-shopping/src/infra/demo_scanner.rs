//! The scanner the demo uses when no OpenAI key is configured.
use crate::domain::{
    CorrectionRule, Error, ExtractedItem, ExtractedProduct, Extraction, Intelligence, Product,
    Result,
};
use async_trait::async_trait;

/// Stand-in scanner for demos and local runs without an OpenAI key.
///
/// It does not look at the photo. It returns a plausible receipt made of
/// things the crew already buys, and says so in the receipt's notes, so the
/// rest of the flow (matching the list, the ledger, notifications) still works.
pub struct DemoScanner;

const DEMO_FALLBACK: [(&str, &str, &str, i64); 3] = [
    ("Cat food", "pets", "Pet food", 449),
    ("Dish soap", "home", "Cleaning", 189),
    ("Instant noodles", "food", "Snacks", 279),
];

#[async_trait]
impl Intelligence for DemoScanner {
    fn configured(&self) -> bool {
        true
    }

    async fn receipt(
        &self,
        photos: &[Vec<u8>],
        catalog: &[Product],
        _rules: &[CorrectionRule],
    ) -> Result<Extraction> {
        if photos.is_empty() || photos.len() > 5 {
            return Err(Error::Invalid("Use 1 to 5 photos of one receipt".into()));
        }
        let picks: Vec<(String, String, String, i64)> = if catalog.is_empty() {
            DEMO_FALLBACK
                .iter()
                .map(|(n, c, s, a)| ((*n).into(), (*c).into(), (*s).into(), *a))
                .collect()
        } else {
            catalog
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, p)| {
                    let price = 149 + (p.name.len() as i64 * 37 + i as i64 * 113) % 850;
                    (
                        p.name.clone(),
                        p.category.clone(),
                        p.subcategory.clone(),
                        price,
                    )
                })
                .collect()
        };
        let items: Vec<ExtractedItem> = picks
            .iter()
            .map(|(name, category, subcategory, amount)| ExtractedItem {
                name: name.clone(),
                raw_name: name.to_uppercase(),
                product_id: catalog
                    .iter()
                    .find(|p| &p.name == name)
                    .map(|p| p.id.clone()),
                variant: String::new(),
                category: category.clone(),
                subcategory: subcategory.clone(),
                quantity: 1.0,
                unit: "pc".into(),
                amount_minor: *amount,
                uncertain: false,
                translated_name: name.clone(),
            })
            .collect();
        Ok(Extraction {
            merchant: Some("Corner Market (demo scan)".into()),
            date: Some(chrono::Utc::now().date_naive().to_string()),
            currency: Some("EUR".into()),
            total_minor: Some(items.iter().map(|i| i.amount_minor).sum()),
            adjustments_minor: 0,
            restaurant: false,
            items,
            needs_review: false,
            issues: vec![
                "Demo scan: these items were not read from the photo. Set OPENAI_API_KEY and OPENAI_MODEL to scan real receipts."
                    .into(),
            ],
            time: None,
        })
    }

    async fn list(&self, text: &str, catalog: &[Product]) -> Result<Vec<ExtractedProduct>> {
        let items: Vec<ExtractedProduct> = text
            .split([',', '\n', ';'])
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .take(100)
            .map(|name| {
                let known = catalog.iter().find(|p| p.name.eq_ignore_ascii_case(name));
                ExtractedProduct {
                    name: known.map_or_else(|| name.to_string(), |p| p.name.clone()),
                    product_id: known.map(|p| p.id.clone()),
                    category: known.map_or_else(|| "other".into(), |p| p.category.clone()),
                    subcategory: known
                        .map_or_else(|| "Uncategorized".into(), |p| p.subcategory.clone()),
                    quantity: "1".into(),
                    variant: String::new(),
                    original_name: name.to_string(),
                }
            })
            .collect();
        if items.is_empty() {
            return Err(Error::Model("Write a few items separated by commas".into()));
        }
        Ok(items)
    }
}

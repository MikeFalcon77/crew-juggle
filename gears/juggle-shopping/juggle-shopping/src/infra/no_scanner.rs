//! No receipt scanning: a production server without an OpenAI model.
use crate::domain::{
    CorrectionRule, Error, ExtractedProduct, Extraction, Intelligence, Product, Result,
};
use async_trait::async_trait;

/// Says scanning is off, so uploads are refused and nothing is made up.
pub struct NoScanner;

fn off() -> Error {
    Error::Model("Receipt scanning is not set up on the server".into())
}

#[async_trait]
impl Intelligence for NoScanner {
    fn configured(&self) -> bool {
        false
    }

    async fn receipt(
        &self,
        _: &[Vec<u8>],
        _: &[Product],
        _: &[CorrectionRule],
    ) -> Result<Extraction> {
        Err(off())
    }

    async fn list(&self, _: &str, _: &[Product]) -> Result<Vec<ExtractedProduct>> {
        Err(off())
    }
}

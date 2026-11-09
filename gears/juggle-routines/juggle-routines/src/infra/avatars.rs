//! Photo uploads: only bounded square PNGs, re-encoded without metadata.
use crate::domain::{Error, Result};
use async_trait::async_trait;
use std::io::Cursor;

/// Validates and decodes bounded PNG images outside async workers.
pub struct PngNormalizer;
#[async_trait]
impl crate::domain::AvatarNormalizer for PngNormalizer {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        tokio::task::spawn_blocking(move || normalize(&bytes))
            .await
            .map_err(|e| Error::Storage(e.into()))?
    }
}

fn failure(e: impl Into<anyhow::Error>) -> Error {
    Error::Storage(e.into())
}
/// Accepts only bounded square PNG images and strips extraneous metadata.
pub fn normalize(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() > 512 * 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(Error::Invalid("Upload a square PNG avatar".into()));
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(256);
    limits.max_image_height = Some(256);
    limits.max_alloc = Some(8 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| Error::Invalid("The photo is damaged or too large".into()))?;
    if image.width() != 256 || image.height() != 256 {
        return Err(Error::Invalid("Avatars must be 256 × 256".into()));
    }
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image.to_rgba8())
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(failure)?;
    Ok(output.into_inner())
}
#[cfg(test)]
#[path = "../tests/avatars_tests.rs"]
mod regression_tests;

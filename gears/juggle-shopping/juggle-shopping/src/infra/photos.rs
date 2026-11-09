//! Receipt photos: JPEG, PNG or WebP in, an upright JPEG out.
use crate::domain::{Error, PhotoNormalizer, Result};
use async_trait::async_trait;
use image::{ImageDecoder, ImageReader};
use std::io::Cursor;

/// Largest stored photo: wide enough to read, tall enough for a long receipt.
const MAX_WIDTH: u32 = 4000;
const MAX_HEIGHT: u32 = 16000;

/// Decodes, bounds and re-encodes one uploaded photo.
pub async fn normalize(bytes: Vec<u8>) -> Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        if bytes.is_empty() || bytes.len() > 10 * 1024 * 1024 {
            return Err(Error::Invalid("Photos must be at most 10 MB".into()));
        }
        let mut reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|_| Error::Invalid("Could not read the photo".into()))?;
        if !matches!(
            reader.format(),
            Some(image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::WebP)
        ) {
            return Err(Error::Invalid("JPEG, PNG and WebP are supported".into()));
        }
        let mut limits = image::Limits::default();
        limits.max_alloc = Some(160 * 1024 * 1024);
        limits.max_image_width = Some(20000);
        limits.max_image_height = Some(20000);
        reader.limits(limits);
        let mut decoder = reader
            .into_decoder()
            .map_err(|_| Error::Invalid("The photo is damaged or too large".into()))?;
        let orientation = decoder
            .orientation()
            .map_err(|_| Error::Invalid("Could not read the photo orientation".into()))?;
        let mut image = image::DynamicImage::from_decoder(decoder)
            .map_err(|_| Error::Invalid("The photo is damaged or too large".into()))?;
        image.apply_orientation(orientation);
        if image.width() < 100 || image.height() < 100 {
            return Err(Error::Invalid("The photo is too small to scan".into()));
        }
        // Shrink very large photos; `thumbnail` would also enlarge small ones.
        let image = if image.width() > MAX_WIDTH || image.height() > MAX_HEIGHT {
            image.thumbnail(MAX_WIDTH, MAX_HEIGHT)
        } else {
            image
        }
        .to_rgb8();
        let mut output = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, 90)
            .encode_image(&image)
            .map_err(|_| Error::Invalid("Could not process the photo".into()))?;
        if output.len() > 10 * 1024 * 1024 {
            return Err(Error::Invalid(
                "The photo is too large. Take the receipt in parts.".into(),
            ));
        }
        Ok(output)
    })
    .await
    .map_err(|_| Error::Storage)?
}

/// The photo normalizer the upload use case calls.
pub struct JpegNormalizer;

#[async_trait]
impl PhotoNormalizer for JpegNormalizer {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        normalize(bytes).await
    }
}

#[cfg(test)]
#[path = "photos_tests.rs"]
mod photos_tests;

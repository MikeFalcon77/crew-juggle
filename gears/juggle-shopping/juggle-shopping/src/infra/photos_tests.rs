use super::*;

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb([250, 250, 245]));
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 80)
        .encode_image(&image)
        .unwrap();
    bytes
}

fn size(bytes: &[u8]) -> (u32, u32) {
    let image = image::load_from_memory(bytes).unwrap();
    (image.width(), image.height())
}

#[tokio::test]
async fn small_photos_keep_their_size() {
    let out = normalize(jpeg(300, 400)).await.unwrap();
    assert_eq!(size(&out), (300, 400));
}

#[tokio::test]
async fn wide_photos_are_shrunk_to_the_limit() {
    let out = normalize(jpeg(4400, 200)).await.unwrap();
    assert_eq!(size(&out), (4000, 182));
}

#[tokio::test]
async fn tiny_photos_are_refused() {
    assert!(matches!(
        normalize(jpeg(50, 400)).await,
        Err(Error::Invalid(_))
    ));
}

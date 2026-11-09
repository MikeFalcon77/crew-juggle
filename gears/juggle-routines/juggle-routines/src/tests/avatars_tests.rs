#[cfg(test)]
mod tests {
    use crate::infra::avatars::*;
    use std::io::Cursor;
    #[test]
    fn only_bounded_square_png_is_kept_without_extra_metadata() {
        let mut buffer = Cursor::new(Vec::new());
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            256,
            256,
            image::Rgba([40, 120, 190, 255]),
        ));
        image
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();
        let mut source = buffer.into_inner();
        source.extend_from_slice(b"PRIVATE_METADATA_SHOULD_NOT_SURVIVE");
        let normalized = normalize(&source).unwrap();
        assert!(
            !normalized
                .windows(b"PRIVATE_METADATA_SHOULD_NOT_SURVIVE".len())
                .any(|s| s == b"PRIVATE_METADATA_SHOULD_NOT_SURVIVE")
        );
        assert_eq!(
            image::load_from_memory(&normalized).unwrap().to_rgba8(),
            image.to_rgba8()
        );
        assert!(normalize(b"not an image").is_err());
        assert!(normalize(&vec![0; 513 * 1024]).is_err());
        let mut small = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(16, 16)
            .write_to(&mut small, image::ImageFormat::Png)
            .unwrap();
        assert!(normalize(&small.into_inner()).is_err());
    }
}

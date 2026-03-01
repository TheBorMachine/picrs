use crate::errors::{ServiceError, ServiceResult};

use std::{
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
};

use image::{DynamicImage, ImageFormat, codecs::jpeg::JpegEncoder};

pub struct ProcessorConfig {
    pub base_dir: PathBuf,
    pub default_quality: u8,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("download"),
            default_quality: 85,
        }
    }
}

pub struct ProcessingResult {
    pub original_path: String,
    pub processed_paths: Vec<String>,
}

pub async fn process_image(
    data: Vec<u8>,
    format: String,
    widths: Vec<u32>,
    heights: Vec<u32>,
) -> ServiceResult<ProcessingResult> {
    let config = ProcessorConfig::default();
    tokio::task::spawn_blocking(move || process_with_config(data, format, widths, heights, config))
        .await
        .map_err(|e| ServiceError::InternalError {
            message: e.to_string(),
        })?
}

fn process_with_config(
    data: Vec<u8>,
    format: String,
    widths: Vec<u32>,
    heights: Vec<u32>,
    config: ProcessorConfig,
) -> ServiceResult<ProcessingResult> {
    let img = image::load_from_memory(&data).map_err(|e| ServiceError::ImageLoadError {
        message: e.to_string(),
    })?;

    let img_dir = config.base_dir.join("images");
    fs::create_dir_all(&img_dir).map_err(|e| ServiceError::InternalError {
        message: e.to_string(),
    })?;

    let original_ext = guess_extension(&data);
    let original_filename = format!("original.{}", original_ext);
    let original_full_path = img_dir.join(&original_filename);

    fs::write(&original_full_path, &data).map_err(|e| ServiceError::ImageSaveError {
        message: e.to_string(),
    })?;

    let mut processed_paths = Vec::new();
    let target_format = parse_format(&format)?;
    let count = widths.len().min(heights.len());

    if widths.len() != heights.len() {
        return Err(ServiceError::InvalidParameters {
            message: "Widths and heights arrays must have same length".to_string(),
        });
    }

    for i in 0..count {
        let w = widths[i];
        let h = heights[i];

        if w == 0 || h == 0 {
            return Err(ServiceError::InvalidParameters {
                message: format!("Invalid dimensions: {}x{}", w, h),
            });
        }

        let resized = img.resize(w, h, image::imageops::FilterType::Lanczos3);

        let filename = format!("{}_{}.{}", w, h, format);
        let file_path = img_dir.join(&filename);

        save(&resized, &file_path, target_format, config.default_quality)?;
        processed_paths.push(file_path.to_string_lossy().to_string());
    }

    Ok(ProcessingResult {
        original_path: original_full_path.to_string_lossy().to_string(),
        processed_paths,
    })
}

#[allow(dead_code)]
fn process(
    data: Vec<u8>,
    format: String,
    widths: Vec<u32>,
    heights: Vec<u32>,
) -> ServiceResult<ProcessingResult> {
    let config = ProcessorConfig::default();
    process_with_config(data, format, widths, heights, config)
}

fn parse_format(fmt: &str) -> ServiceResult<ImageFormat> {
    match fmt.to_lowercase().as_str() {
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "webp" => Ok(ImageFormat::WebP),
        _ => Err(ServiceError::InvalidFormat {
            format: fmt.to_string(),
        }),
    }
}

#[cfg(test)]
fn parse_format_test(fmt: &str) -> ServiceResult<ImageFormat> {
    parse_format(fmt)
}

fn guess_extension(data: &[u8]) -> &str {
    match image::guess_format(data) {
        Ok(ImageFormat::Png) => "png",
        Ok(ImageFormat::Jpeg) => "jpg",
        Ok(ImageFormat::WebP) => "webp",
        _ => "bin",
    }
}

fn save(img: &DynamicImage, path: &Path, fmt: ImageFormat, quality: u8) -> ServiceResult<()> {
    match fmt {
        ImageFormat::Jpeg => {
            let file = fs::File::create(path).map_err(|e| ServiceError::ImageSaveError {
                message: e.to_string(),
            })?;
            let writer = BufWriter::new(file);
            let encoder = JpegEncoder::new_with_quality(writer, quality);
            img.write_with_encoder(encoder)
                .map_err(|e| ServiceError::ImageSaveError {
                    message: e.to_string(),
                })?;
        }
        _ => {
            img.save(path).map_err(|e| ServiceError::ImageSaveError {
                message: e.to_string(),
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn sample_png() -> Vec<u8> {
        let img = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let mut buffer = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
            .expect("Failed to write PNG");
        buffer
    }

    fn sample_jpeg() -> Vec<u8> {
        let img = RgbaImage::from_pixel(10, 10, Rgba([0, 255, 0, 255]));
        let mut buffer = Vec::new();
        let encoder = JpegEncoder::new_with_quality(&mut buffer, 85);
        DynamicImage::ImageRgba8(img)
            .write_with_encoder(encoder)
            .expect("Failed to write JPEG");
        buffer
    }

    #[test]
    fn test_parse_format_valid() {
        assert_eq!(parse_format_test("png").unwrap(), ImageFormat::Png);
        assert_eq!(parse_format_test("PNG").unwrap(), ImageFormat::Png);
        assert_eq!(parse_format_test("jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format_test("jpg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format_test("JPG").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format_test("webp").unwrap(), ImageFormat::WebP);
    }

    #[test]
    fn test_parse_format_invalid() {
        assert!(parse_format_test("gif").is_err());
        assert!(parse_format_test("bmp").is_err());
        assert!(parse_format_test("").is_err());
    }

    #[test]
    fn test_guess_extension_png() {
        let png_data = sample_png();
        assert_eq!(guess_extension(&png_data), "png");
    }

    #[test]
    fn test_guess_extension_jpeg() {
        let jpeg_data = sample_jpeg();
        assert_eq!(guess_extension(&jpeg_data), "jpg");
    }

    #[test]
    fn test_guess_extension_unknown() {
        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(guess_extension(&unknown_data), "bin");
    }

    #[test]
    fn test_process_empty_image_data() {
        let result = process(Vec::new(), "png".to_string(), vec![100], vec![100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_mismatched_dimensions() {
        let png_data = sample_png();
        let result = process(png_data, "png".to_string(), vec![100, 200], vec![100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_zero_dimensions() {
        let png_data = sample_png();
        let result = process(png_data, "png".to_string(), vec![0], vec![100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_processor_config_default() {
        let config = ProcessorConfig::default();
        assert_eq!(config.base_dir, PathBuf::from("download"));
        assert_eq!(config.default_quality, 85);
    }
}

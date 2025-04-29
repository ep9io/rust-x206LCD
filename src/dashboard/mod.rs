use anyhow::Context;
use image::{DynamicImage, Rgba, RgbaImage};

use crate::config::AppConfig;
use crate::models::SystemInfo;
use crate::renderer::{drawing, widgets};

// Create a dashboard image with system information
pub fn create_image(config: &AppConfig, info: &SystemInfo) -> DynamicImage {
    let width = config.lcd.width as u32;
    let height = config.lcd.height as u32;

    // Create a new image
    let mut image = RgbaImage::new(width, height);

    // Fill with black
    for pixel in image.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 255]);
    }

    // Calculate layout dimensions
    let x_middle = width / 2;

    let y_header = 0u32;
    let y_centre = 30u32;
    let y_footer = (height as f32 * (2.0 / 3.0)) as u32;

    // Draw header
    let mut header_ctx = widgets::RenderContext {
        config,
        info,
        image: &mut image,
        x: 0,
        y: y_header,
        width,
    };
    widgets::render_header(&mut header_ctx);

    // Draw separator line below header
    drawing::horizonal_line(&mut image, 0, y_centre, width);

    // Draw a vertical separator line
    drawing::vertical_line(&mut image, x_middle, y_centre, y_footer);

    // TOP SECTION (66% of height)
    // LEFT SIDE (50% of width) - Resource bars
    let y_top_section = y_centre + 10;
    let mut resource_bars_ctx = widgets::RenderContext {
        config,
        info,
        image: &mut image,
        x: 0,
        y: y_top_section,
        width: x_middle,
    };
    widgets::render_resource_bars(&mut resource_bars_ctx);

    // RIGHT SIDE (50% of width) - Process list
    let mut processes_ctx = widgets::RenderContext {
        config,
        info,
        image: &mut image,
        x: x_middle,
        y: y_top_section,
        width: x_middle,
    };
    widgets::render_processes(&mut processes_ctx);

    // BOTTOM SECTION (33% of height)
    drawing::horizonal_line(&mut image, 0, y_footer + 1, width);

    let y_bottom_section = y_footer + 6;

    let mut footer_ctx = widgets::RenderContext {
        config,
        info,
        image: &mut image,
        x: 0,
        y: y_bottom_section,
        width,
    };
    widgets::render_footer(&mut footer_ctx);

    // Convert to RGB for saving as PNG
    let dynamic_image = DynamicImage::ImageRgba8(image);

    // Apply nearest-neighbor scaling
    dynamic_image.resize_exact(width, height, image::imageops::FilterType::Nearest)
}

pub fn save_image(config: &AppConfig, image: &DynamicImage) {
    let source_file = &config.dashboard.file;

    image
        .save(source_file)
        .context(format!("Failed to save dashboard to {}", source_file))
        .expect("Unable to save dashboard image to file");
}

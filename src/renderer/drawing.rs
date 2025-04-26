use crate::renderer::fonts::FontConfig;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{
    draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut,
};
use imageproc::rect::Rect;

pub fn horizonal_line(image: &mut RgbaImage, x: u32, y: u32, width: u32) {
    draw_line_segment_mut(
        image,
        (x as f32, y as f32),
        (width as f32, y as f32),
        Rgba([60, 60, 60, 255]),
    );
}

pub fn vertical_line(image: &mut RgbaImage, x: u32, y1: u32, y2: u32) {
    draw_line_segment_mut(
        image,
        (x as f32, y1 as f32),
        (x as f32, y2 as f32),
        Rgba([60, 60, 60, 255]),
    );
}

pub fn text(
    image: &mut RgbaImage,
    colour: Rgba<u8>,
    x: i32,
    y: i32,
    font_config: &FontConfig,
    header_text: &str,
) {
    draw_text_mut(
        image,
        colour,
        x,
        y,
        font_config.scale,
        &font_config.font,
        &header_text,
    );
}
pub fn progress_bar(
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    percentage: f32,
    color: Rgba<u8>,
) {
    let bg_colour = Rgba([30, 30, 30, 255]);

    // Background
    draw_filled_rect_mut(
        image,
        Rect::at(x, y).of_size(width, height),
        bg_colour,
    );

    // Progress
    let bar_width = (percentage * width as f32) as u32;

    if bar_width > 0 {
        draw_filled_rect_mut(
            image,
            Rect::at(x as i32, y as i32).of_size(bar_width, height),
            color,
        );
    }

    // Border
    draw_hollow_rect_mut(
        image,
        Rect::at(x as i32, y as i32).of_size(width, height),
        Rgba([100, 100, 100, 255]),
    );
}

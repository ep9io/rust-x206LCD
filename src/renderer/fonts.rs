use rusttype::{Font, Scale};

pub(crate) struct FontConfig {
    pub font: Font<'static>,
    pub scale: Scale,
}

pub fn title_font_config() -> FontConfig {
    let config = get_font_config(24.0);
    config
}

pub fn regular_font_config() -> FontConfig {
    let config = get_font_config(24.0);
    config
}

pub fn small_font_config() -> FontConfig {
    let config = get_font_config(20.0);
    config
}

fn get_font_config(scale: f32) -> FontConfig {
    let font = Vec::from(include_bytes!("fonts/DejaVuSansMono.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();
    let scale = Scale::uniform(scale);
    FontConfig { font, scale }
}
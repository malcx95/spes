use std::path::PathBuf;

use color_eyre::Result;
use egui_extras::image::RetainedImage;
use egui_macroquad::egui::{self};

use macroquad::texture::*;

fn load_image_from_path(bytes: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(bytes)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub struct EguiTextures {
    pub cannon: RetainedImage,
}

pub struct Stars {
    pub stars: Vec<Texture2D>,
}

pub struct Assets {
    pub malcolm: Texture2D,
    pub stars: Stars,
    pub egui_textures: EguiTextures,
}

impl Stars {
    pub fn new() -> Stars {
        let stars = vec![
            Texture2D::from_file_with_format(include_bytes!("../resources/star1.png"), None),
            Texture2D::from_file_with_format(include_bytes!("../resources/star2.png"), None),
            Texture2D::from_file_with_format(include_bytes!("../resources/star3.png"), None),
        ];
        Stars { stars }
    }
}

impl Assets {
    pub fn new() -> Result<Assets> {
        let assets = Assets {
            malcolm: Texture2D::from_file_with_format(
                include_bytes!("../resources/malcolm.png"),
                None,
            ),
            stars: Stars::new(),
            egui_textures: EguiTextures {
                cannon: RetainedImage::from_color_image(
                    "egui",
                    load_image_from_path(include_bytes!("../resources/cannon1.png"))?,
                ),
            },
        };
        Ok(assets)
    }
}

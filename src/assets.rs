use color_eyre::Result;
use egui_extras::image::RetainedImage;
use egui_macroquad::egui;

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
    pub node_bg: Texture2D,
    pub root_node: Texture2D,
    pub cannon: Texture2D,
    pub thrusters: Texture2D,
    pub thrust_flame: Texture2D,
    pub shield: Texture2D,
    pub stars: Stars,
    pub egui_textures: EguiTextures,
    pub bullet: Texture2D,
    pub reaction_wheel_bot: Texture2D,
    pub reaction_wheel_mid: Texture2D,
    pub reaction_wheel_top: Texture2D,
}

impl Stars {
    pub fn new() -> Stars {
        let stars = vec![
            Texture2D::from_file_with_format(include_bytes!("../resources/star1.png"), None),
            Texture2D::from_file_with_format(include_bytes!("../resources/star2.png"), None),
            Texture2D::from_file_with_format(include_bytes!("../resources/star3.png"), None),
            //FIXME(frans): Star4
            Texture2D::from_file_with_format(include_bytes!("../resources/star3.png"), None),
        ];
        Stars { stars }
    }
}

macro_rules! load_pixelart {
    ($path:expr) => {{
        let result = Texture2D::from_file_with_format(include_bytes!($path), None);
        result.set_filter(macroquad::texture::FilterMode::Nearest);
        result
    }};
}

impl Assets {
    pub fn new() -> Result<Assets> {
        let assets = Assets {
            malcolm: load_pixelart!("../resources/malcolm.png"),
            node_bg: load_pixelart!("../resources/ship/base.png"),
            root_node: load_pixelart!("../resources/ship/root2.png"),
            shield: load_pixelart!("../resources/ship/shield.png"),
            cannon: load_pixelart!("../resources/cannon1.png"),
            thrust_flame: load_pixelart!("../resources/ship/thruster.png"),
            stars: Stars::new(),
            egui_textures: EguiTextures {
                cannon: RetainedImage::from_color_image(
                    "egui",
                    load_image_from_path(include_bytes!("../resources/cannon1.png"))?,
                ),
            },
            bullet: load_pixelart!("../resources/ship/laser.png"),
            thrusters: load_pixelart!("../resources/ship/thrusters.png"),
            reaction_wheel_bot: load_pixelart!("../resources/ship/reaction_wheel_bottom.png"),
            reaction_wheel_mid: load_pixelart!("../resources/ship/reaction_wheel_wheel.png"),
            reaction_wheel_top: load_pixelart!("../resources/ship/reaction_wheel_top.png"),
        };
        Ok(assets)
    }
}

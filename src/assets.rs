use enum_map::{enum_map, EnumMap};

use macroquad::texture::*;
use libplen::constants;

pub struct Assets {
    pub malcolm: Texture2D,
}

impl Assets {
    pub fn new() -> Assets {
        let mut assets = Assets {
            malcolm: Texture2D::from_file_with_format(include_bytes!("../resources/malcolm.png"), None),
        };
        assets
    }
}

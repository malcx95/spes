use enum_map::{enum_map, EnumMap};

use macroquad::texture::*;
use libplen::constants;


pub struct Stars {
    pub stars: Vec<Texture2D>
}


pub struct Assets {
    pub malcolm: Texture2D,
    pub stars: Stars,
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
    pub fn new() -> Assets {
        let assets = Assets {
            malcolm: Texture2D::from_file_with_format(include_bytes!("../resources/malcolm.png"), None),
            stars: Stars::new(),
        };
        assets
    }
}

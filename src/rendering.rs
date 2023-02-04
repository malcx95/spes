use macroquad::prelude::*;
use macroquad::texture;


pub fn draw_texture(texture: texture::Texture2D, x: f32, y: f32, angle: f32) {
    let params = texture::DrawTextureParams {
        dest_size: None,
        source: None,
        rotation: angle,
        flip_x: false,
        flip_y: false,
        pivot: None,
    };
    texture::draw_texture_ex(texture, x, y, WHITE, params);
}


/*
pub fn draw_texture_pivot(texture: texture::Texture2D, x: f32, y: f32, angle: f32, pivot_x: f32, pivot_y: f32) {
    let params = texture::DrawTextureParams {
        dest_size: None,
        source: None,
        rotation: angle,
        flip_x: false,
        flip_y: false,
        pivot: Some(Vec2::new(pivot_x, pivot_y)),
    };
    texture::draw_texture_ex(texture, x, y, WHITE, params);
}
*/


pub fn draw_texture_pivot_size(texture: texture::Texture2D, x: f32, y: f32, angle: f32, pivot_x: f32, pivot_y: f32, size_x: f32, size_y: f32) {
    let params = texture::DrawTextureParams {
        dest_size: Some(Vec2::new(size_x, size_y)),
        source: None,
        rotation: angle,
        flip_x: false,
        flip_y: false,
        pivot: Some(Vec2::new(pivot_x, pivot_y)),
    };
    texture::draw_texture_ex(texture, x, y, WHITE, params);
}

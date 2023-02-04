use egui_macroquad::egui::emath::exponential_smooth_factor;
use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{self, vec2, Vec2};
use macroquad::prelude::*;
use macroquad::texture;

use crate::assets::Assets;

pub struct ClientState {
    stars_material: macroquad::material::Material,
    // add client side state
}

const STARS_VERT: &str = include_str!("./shaders/stars.vert");
const STARS_FRAG: &str = include_str!("./shaders/stars.frag");

impl ClientState {
    pub fn new() -> ClientState {
        let stars_material = macroquad::material::load_material(
            STARS_VERT,
            STARS_FRAG,
            macroquad::material::MaterialParams {
                pipeline_params: Default::default(),
                uniforms: vec![
                    ("window_dimensions".into(), UniformType::Float2),
                    ("player".into(), UniformType::Float2),
                    ("global_scale".into(), UniformType::Float1),
                    ("global_offset".into(), UniformType::Float1)
                ],
                textures: vec![],
            },
        )
        .unwrap();

        ClientState {
            stars_material, // init client stuff
        }
    }

    pub fn update(&mut self, delta_time: f32, game_state: &mut GameState, my_id: u64) {
        // update client side stuff
    }

    pub fn draw(
        &mut self,
        my_id: u64,
        game_state: &GameState,
        assets: &mut Assets,
    ) -> Result<(), String> {
        clear_background(BLACK);

        let player = game_state.players.iter().find(|p| p.id == my_id);
        if let Some(p) = player {
            Self::draw_background(self, p.position.x, p.position.y, p.speed);
        }

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);

        for player in &game_state.players {
            let params = texture::DrawTextureParams {
                dest_size: None,
                source: None,
                rotation: player.angle,
                flip_x: false,
                flip_y: false,
                pivot: None,
            };

            let px = player.position.x;
            let py = player.position.y;

            texture::draw_texture_ex(assets.malcolm, px, py, BLUE, params);
        }

        Ok(())
    }

    fn draw_background(
        client_state: &mut ClientState,
        player_x: f32,
        player_y: f32,
        player_vel: f32,
    ) {
        let mat = &client_state.stars_material;
        mat.set_uniform("window_dimensions", (screen_width(), screen_height()));
        mat.set_uniform("player", (player_x / 100.0, -player_y / 100.0));
        mat.set_uniform(
            "global_scale",
            1.0 - (exponential_smooth_factor(0.5, 2000.0, player_vel)),
        );
        gl_use_material(*mat);
        draw_cube((0., 0.0, 0.0).into(), (2.0, 2.0, 0.0).into(), None, WHITE);
        gl_use_default_material();
    }
}

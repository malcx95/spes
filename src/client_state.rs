use ::rand::Rng;
use color_eyre::Result;
use libplen::constants;
use libplen::gamestate::GameState;
use macroquad::prelude::*;

use crate::assets::Assets;

use crate::rendering;

pub struct Star {
    x: f32,
    y: f32,
    star_index: i32,
}

pub struct ClientState {
    stars: Vec<Star>,
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState {
            stars: Self::init_stars(),
        }
    }

    fn init_stars() -> Vec<Star> {
        let mut stars = vec![];
        let mut rng = ::rand::thread_rng();
        for _ in 0..constants::NUM_STARS {
            let x = rng.gen_range((-constants::WORLD_SIZE)..(2. * constants::WORLD_SIZE));
            let y = rng.gen_range((-constants::WORLD_SIZE)..(2. * constants::WORLD_SIZE));
            let i = rng.gen_range(0..2);
            stars.push(Star {
                x,
                y,
                star_index: i,
            });
        }
        stars
    }

    pub fn update(&mut self, _delta_time: f32, _game_state: &mut GameState, _my_id: u64) {
        // update client side stuff
    }

    pub fn draw(&mut self, my_id: u64, game_state: &GameState, assets: &Assets) -> Result<()> {
        clear_background(BLACK);

        let player = game_state.players.iter().find(|p| p.id == my_id);
        if let Some(p) = player {
            Self::draw_background2(self, assets, p.position().x, p.position().y, p.angle());
        }

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);

        for player in &game_state.players {
            for component in &player.components {
                rendering::draw_texture(assets.malcolm, component.pos.x, component.pos.y, 0.);
            }
        }

        Ok(())
    }

    fn draw_background2(
        client_state: &mut ClientState,
        assets: &Assets,
        player_x: f32,
        player_y: f32,
        player_angle: f32,
    ) {
        for star in &client_state.stars {
            let star_texture = assets.stars.stars[star.star_index as usize];
            let pivot_x = screen_width() / 2.;
            let pivot_y = screen_height() / 2.;
            rendering::draw_texture_pivot_size(
                star_texture,
                star.x - player_x,
                star.y - player_y,
                -player_angle,
                pivot_x,
                pivot_y,
                20.,
                20.,
            );
        }
    }

    /*
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
    */
}

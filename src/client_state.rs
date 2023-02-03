use libplen::constants;
use libplen::gamestate::GameState;
use libplen::math::{self, vec2, Vec2};

use crate::assets::Assets;

pub struct ClientState {
    // add client side state
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState {
            // init client stuff
        }
    }

    pub fn update(&mut self, delta_time: f32, game_state: &GameState, my_id: u64) {
        // update client side stuff
    }

    pub fn draw(
        &self,
        my_id: u64,
        game_state: &GameState,
        // TODO add some sort of canvas and shit
        assets: &mut Assets,
    ) -> Result<(), String> {
        // let (screen_w, screen_h) = canvas.logical_size();
        // let screen_center = vec2(screen_w as f32 * 0.5, screen_h as f32 * 0.5);

        // draw some stuff
        for player in &game_state.players { }

        Ok(())
    }
}

use serde_derive::{Deserialize, Serialize};

use crate::math::{self, Vec2};
use crate::player::ComponentSpecialization;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum SoundEffect {
    Powerup,
    Explosion,
    Gun,
    LaserCharge,
    LaserFire,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    AssignId(u64),
    GameState(crate::gamestate::GameState),
}

#[derive(Serialize, Deserialize, Default)]
pub struct ClientInput {
    pub x_input: f32,
    pub y_input: f32,

    pub mouse_x: f32,
    pub mouse_y: f32,

    pub mouse_world: Option<Vec2>,

    pub shoot: bool,
    pub aim_angle: f32,

    pub mouse_left: bool,
    pub mouse_right: bool,
    pub shielding: bool,
}

impl ClientInput {
    pub fn new() -> Self {
        ClientInput {
            x_input: 0.,
            y_input: 0.,
            mouse_x: 0.,
            mouse_y: 0.,
            mouse_world: None,
            shoot: false,
            aim_angle: 0.,
            mouse_left: false,
            mouse_right: false,
            shielding: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Input(ClientInput),
    AddComponent {
        world_pos: math::Vec2,
        specialization: ComponentSpecialization,
    },
    JoinGame {
        name: String,
    },
}

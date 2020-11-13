use serde_derive::{Serialize, Deserialize};

use crate::math::{Vec2, vec2};


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub position: Vec2,
    pub angle: f32,
    pub speed: f32
}


impl Player {
    pub fn new(
        id: u64,
        name: String
    ) -> Player {
        Player {
            id,
            name,
            position: vec2(0., 0.),
            angle: 0.,
            speed: 0.
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // player update here
        self.position += Vec2::from_direction(self.angle, self.speed);
    }
}

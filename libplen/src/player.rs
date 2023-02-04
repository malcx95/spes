use serde_derive::{Serialize, Deserialize};

use crate::math::{Vec2, vec2};
use std::f32::consts::PI;

use rapier2d::prelude::*;
use rapier2d::prelude::{RigidBodyHandle, RigidBodySet};

const PLAYER_ANGLE_SPEED: f32 = 0.01;
const PLAYER_ANGLE_OFFSET: f32 = PI / 2.;
const PLAYER_FORWARD_INERTIA: f32 = 5.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct Component {
    pub pos: Vec2,
    pub physics_handle: RigidBodyHandle,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub input_x: f32,
    pub input_y: f32,

    pub position: Vec2,
    pub angle: f32,
    pub speed: f32,

    pub components: Vec<Component>,
}

impl Player {
    pub fn new(id: u64, name: String, components: Vec<Component>) -> Player {
        Player {
            id,
            name,

            input_x: 0.,
            input_y: 0.,

            position: vec2(0., 0.),
            angle: 0.,
            speed: 0.,

            components,
        }
    }

    pub fn set_input(&mut self, input_x: f32, input_y: f32) {
        self.input_x = input_x;
        self.input_y = input_y;
    }

    pub fn update(&mut self, rigid_body_set: &mut RigidBodySet, delta_time: f32) {
        self.angle += self.input_x * PLAYER_ANGLE_SPEED;
        self.speed += self.input_y * PLAYER_FORWARD_INERTIA;

        self.position += Vec2::from_direction(self.angle - PLAYER_ANGLE_OFFSET, self.speed * delta_time);
        // self.position += Vec2::from_direction(self.angle, self.speed * delta_time);

        let root_handle = self
            .components
            .first()
            .expect("Player without a component")
            .physics_handle;

        println!("{}", self.input_y);
        rigid_body_set
            .get_mut(root_handle)
            .expect(&format!("No rigid body for player {}", self.id))
            .apply_impulse_at_point(vector!(self.input_y, 0.)*100., point![0., 0.], true);
    }
}

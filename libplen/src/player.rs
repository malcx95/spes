use serde_derive::{Deserialize, Serialize};

use crate::math::Vec2;

use rapier2d::prelude::*;
use rapier2d::prelude::{RigidBodyHandle, RigidBodySet};

#[derive(Serialize, Deserialize, Clone)]
pub struct Component {
    pub pos: Vec2,
    pub angle: f32,
    pub physics_handle: RigidBodyHandle,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub input_x: f32,
    pub input_y: f32,

    pub components: Vec<Component>,

    pub is_building: bool,
}

impl Player {
    pub fn new(id: u64, name: String, components: Vec<Component>) -> Player {
        Player {
            id,
            name,

            input_x: 0.,
            input_y: 0.,

            components,

            is_building: false,
        }
    }

    pub fn set_input(&mut self, input_x: f32, input_y: f32) {
        self.input_x = input_x;
        self.input_y = input_y;
    }

    pub fn update(&mut self, rigid_body_set: &mut RigidBodySet, _delta_time: f32) {
        let root_handle = self
            .components
            .first()
            .expect("Player without a component")
            .physics_handle;

        println!("{}", self.input_y);
        let rb = rigid_body_set
            .get_mut(root_handle)
            .expect(&format!("No rigid body for player {}", self.id));

        rb.apply_impulse_at_point(
            rb.position().rotation * vector!(0., -self.input_y) * 100_000.,
            rb.position().translation.vector.into(),
            true,
        );

        rb.apply_torque_impulse(self.input_x * 100_000., true)
    }

    pub fn core(&self) -> &Component {
        self.components.first().expect("Player had no components")
    }

    pub fn position(&self) -> Vec2 {
        self.core().pos
    }

    pub fn angle(&self) -> f32 {
        self.core().angle
    }

    pub fn velocity(&self) -> f32 {
        0.0
    }
}

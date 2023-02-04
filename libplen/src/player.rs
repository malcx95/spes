use serde_derive::{Deserialize, Serialize};

use crate::gamestate::Bullet;
use crate::math::{vec2, Vec2};
use crate::messages::ClientInput;

use rapier2d::prelude::*;
use rapier2d::prelude::{RigidBodyHandle, RigidBodySet};

// one degree
const SHIELD_POINT_SPACING: f32 = 1.;

#[derive(Serialize, Deserialize, Clone)]
pub struct Component {
    pub pos: Vec2,
    pub angle: f32,
    pub physics_handle: RigidBodyHandle,
    pub spec: ComponentSpecialization,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ComponentSpecialization {
    Root,
    Shield,
    Cannon { cooldown: f32, aim: bool },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,

    pub input_x: f32,
    pub input_y: f32,

    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_world_pos: Option<Vec2>,

    pub aim_angle: f32,

    pub components: Vec<Component>,

    pub shoot: bool,

    pub is_building: bool,

    pub shield: Shield,
    pub shielding: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Shield {
    pub points: Vec<Vec2>,
    pub angle: f32,
    pub num_points: usize,
    pub radius: f32,
}

impl Shield {
    pub fn new() -> Shield {
        Shield {
            points: vec![],
            angle: 0.,
            num_points: 0,
            radius: 100.,
        }
    }

    fn init_points(&mut self) {
        self.points = vec![];
        for _ in 0..self.num_points {
            self.points.push(Vec2 { x: 0., y: 0. });
        }
    }

    pub fn set_num_points(&mut self, num_points: usize) {
        self.num_points = num_points;
        self.init_points();
        self.update();
    }

    pub fn update_mouse(&mut self, aim_angle: f32) {
        self.angle = aim_angle;

        println!("Angle: {}", self.angle);
        self.update();
    }

    fn update(&mut self) {
        if self.num_points == 0 {
            return;
        }
        let mut i = 0;
        for a in -((self.num_points / 2) as i32)..(self.num_points as i32 / 2) {
            let angle = self.angle + SHIELD_POINT_SPACING * (a as f32) / (self.num_points as f32);
            let x = self.radius * angle.cos();
            let y = self.radius * angle.sin();

            self.points[i].x = x;
            self.points[i].y = y;

            i += 1; // deal with it okay the chinese food is soon here
        }
    }
}

impl Player {
    pub fn new(id: u64, name: String, components: Vec<Component>) -> Player {
        let mut shield = Shield::new();
        // TODO set according to shield modules
        shield.set_num_points(20);

        Player {
            id,
            name,

            input_x: 0.,
            input_y: 0.,

            mouse_x: 0.,
            mouse_y: 0.,

            mouse_world_pos: None,

            aim_angle: 0.,

            components,

            shoot: false,
            shield,

            is_building: false,

            shielding: false,
        }
    }

    pub fn set_input(&mut self, i: &ClientInput) {
        self.input_x = i.x_input;
        self.input_y = i.y_input;
        self.mouse_x = i.mouse_x;
        self.mouse_y = i.mouse_y;
        self.shoot = i.shoot;
        self.aim_angle = i.aim_angle;
        self.shielding = i.shielding;
        self.mouse_world_pos = i.mouse_world;
    }

    pub fn shield_update(&mut self) {
        self.shield.update_mouse(self.aim_angle);
    }

    pub fn update(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        delta: f32,
        bullets: &mut Vec<Bullet>,
    ) {
        let root_handle = self
            .components
            .first()
            .expect("Player without a component")
            .physics_handle;

        let rb = rigid_body_set
            .get_mut(root_handle)
            .expect(&format!("No rigid body for player {}", self.id));

        rb.reset_forces(true);
        rb.reset_torques(true);
        rb.add_force(
            rb.position().rotation * vector!(0., -self.input_y) * 1000_000.,
            true,
        );
        rb.add_torque(self.input_x * 100_0000., true);
        // rb.apply_impulse_at_point(
        //     rb.position().rotation * vector!(0., -self.input_y) * 100_000.,
        //     rb.position().translation.vector.into(),
        //     true,
        // );

        rb.apply_torque_impulse(self.input_x * 100_000., true);

        self.update_components(rigid_body_set, bullets, delta);

        self.shield_update();
    }

    pub fn update_components(
        &mut self,
        rbs: &mut RigidBodySet,
        bullets: &mut Vec<Bullet>,
        delta: f32,
    ) {
        use ComponentSpecialization as CS;
        self.components = self
            .components
            .iter()
            .filter_map(|c| match c.spec {
                CS::Cannon { cooldown, aim } if cooldown <= 0.0 && self.shoot => {
                    let rb = rbs.get(c.physics_handle).unwrap();
                    let rb_vel = rb.linvel();
                    let angle = if aim {
                        self.mouse_world_pos
                            .map(|p| std::f32::consts::PI - (p - c.pos).atan2())
                            .unwrap_or(0.0)
                    } else {
                        rb.rotation().angle()
                    };
                    let rb = RigidBodyBuilder::new(RigidBodyType::KinematicVelocityBased)
                        .translation(rb.translation().clone())
                        .rotation(angle)
                        .linvel(vector!(
                            rb_vel.x + (1000. * (angle - std::f32::consts::PI / 2.).cos()),
                            rb_vel.y + (1000. * (angle - std::f32::consts::PI / 2.).sin())
                        ))
                        .build();

                    let trans = rb.position().translation;

                    let handle = rbs.insert(rb);

                    let bullet = Bullet {
                        handle,
                        lifetime: 0.,
                        pos: vec2(trans.x, trans.y),
                        angle,
                    };

                    bullets.push(bullet);

                    Some(Component {
                        spec: CS::Cannon { cooldown: 0.5, aim },
                        ..*c
                    })
                }
                CS::Cannon { cooldown, aim } => Some(Component {
                    spec: CS::Cannon {
                        cooldown: cooldown - delta,
                        aim,
                    },
                    ..*c
                }),
                _ => Some(c.clone()),
            })
            .collect();
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

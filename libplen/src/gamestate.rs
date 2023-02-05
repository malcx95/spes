use rapier2d::prelude::*;
use serde_derive::{Deserialize, Serialize};
use crate::physics::PhysicsState;
use crate::constants;
use ::rand::{thread_rng, Rng};
use ::rand::seq::SliceRandom;

use crate::{math::Vec2, player::Player};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub asteroids: Vec<Asteroid>,
    pub asteroid_timer: i32,
    // put server side game state stuff here
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bullet {
    pub handle: RigidBodyHandle,
    pub lifetime: f32,
    pub pos: Vec2,
    pub angle: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asteroid {
    pub handle: RigidBodyHandle,
    pub x: f32,
    pub y: f32,
    pub angle: f32
}

const NUM_ASTEROIDS: usize = 40;
const ASTEROID_MASS: f32 = 20000.;

impl Asteroid {
    pub fn new(p: &mut PhysicsState) -> Asteroid {
        let mut rng = ::rand::thread_rng();
        let x = rng.gen_range(0.0..constants::WORLD_SIZE);
        let y = rng.gen_range(0.0..constants::WORLD_SIZE);

        let vx = rng.gen_range(0.0..10.0);
        let vy = rng.gen_range(0.0..10.0);

        let pos = Vec2 {x, y};

        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![x, y])
            .build();

        let collider = ColliderBuilder::ball(constants::ASTEROID_SIZE/2.)
            .restitution(0.2)
            .friction(0.5)
            .mass(ASTEROID_MASS)
            .build();

        let body_handle = p.rigid_body_set.insert(rb);
        p.collider_set
            .insert_with_parent(collider, body_handle, &mut p.rigid_body_set);

        Asteroid {
            handle: body_handle,
            x: x,
            y: y,
            angle: 0.,
        }
    }
}

impl Bullet {

    pub fn collides_with(self, x: f32, y: f32, radius: f32) -> bool {
        ((self.pos.x - x).powi(2) + (self.pos.y - y).powi(2)).sqrt() < radius
    }

}

impl GameState {
    pub fn new(op: Option<&mut PhysicsState>) -> GameState {
        let mut asteroids = vec![];
        match op {
            None => {},
            Some(p) => {
                for _ in 0..NUM_ASTEROIDS {
                    asteroids.push(Asteroid::new(p));
                }
            }
        }
        GameState {
            players: Vec::new(),
            bullets: Vec::new(),
            asteroids: asteroids,
            asteroid_timer: 0,
            // init server side game state stuff here
        }
    }

    /**
     *  Updates the gamestate and returns
     *  (
     *  vec with player ids that got hit with bullets,
     *  vec with positions where powerups where picked up,
     *  vec with positions where lasers are fired
     *  )
     */
    pub fn update(&mut self, delta: f32, p: &mut PhysicsState) {
        if self.asteroid_timer == 0 {
            self.asteroid_timer = 1000;
            self.asteroids.push(Asteroid::new(p));
            println!("NEW ASTEROID!");
        }
        self.asteroid_timer -= 1;

        for player in &mut self.players {
            player.update(delta, &mut self.bullets, p);
        }

        let mut i: usize = 0;
        while i < self.bullets.len() {
            let bullet = &mut self.bullets[i];

            if bullet.lifetime > 3. {
                self.bullets.remove(i);
            } else {
                bullet.lifetime += delta;
                i += 1;
            }
        }
        for asteroid in &mut self.asteroids {
            let rb = p.rigid_body_set.get_mut(asteroid.handle).unwrap();
            let pos = rb.position();
            let angle = pos.rotation.angle();
            asteroid.x = pos.translation.x;
            asteroid.y = pos.translation.y;
            asteroid.angle = angle;
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player.clone());
    }

    pub fn get_player_by_id(&self, id: u64) -> Option<&Player> {
        for player in &self.players {
            if player.id == id {
                return Some(player);
            }
        }
        None
    }
}

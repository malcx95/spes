use rapier2d::prelude::*;
use serde_derive::{Deserialize, Serialize};

use crate::{math::Vec2, player::Player};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    // put server side game state stuff here
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bullet {
    pub handle: RigidBodyHandle,
    pub lifetime: f32,
    pub pos: Vec2,
    pub angle: f32,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: Vec::new(),
            bullets: Vec::new(),
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
    pub fn update(&mut self, rigid_body_set: &mut RigidBodySet, delta: f32) {
        for player in &mut self.players {
            player.update(rigid_body_set, delta, &mut self.bullets);
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

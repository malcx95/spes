
use rapier2d::prelude::RigidBodySet;
use serde_derive::{Serialize, Deserialize};

use crate::player::{Player};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    // put server side game state stuff here
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: Vec::new(),
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
            player.update(rigid_body_set, delta);
            println!("{} {}", player.position.x, player.position.y);
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

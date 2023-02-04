use rapier2d::prelude::{vector, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, RigidBodyType};
use serde_derive::{Deserialize, Serialize};

use crate::{
    math::{vec2, Vec2},
    player::Player,
};

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
            player.update(rigid_body_set, delta);
            println!("{} {}", player.position().x, player.position().y);
            println!("{}", player.angle());
        }

        let mut i: usize = 0;
        println!("{:?}", self.bullets.len());
        while i < self.bullets.len() {
            let bullet = &mut self.bullets[i];

            if bullet.lifetime > 10. {
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

    pub fn shoot(&mut self, id: u64, rbs: &mut RigidBodySet) -> Bullet {
        let player = self.get_player_by_id(id).unwrap();
        let player_rb = rbs.get(player.core().physics_handle).unwrap();
        let rb = RigidBodyBuilder::new(RigidBodyType::Fixed)
            .translation(player_rb.translation().clone())
            .rotation(player_rb.rotation().angle())
            .build();

        let pos = rb.position().translation;
        let pos = vec2(pos.x, pos.y);
        let angle = rb.position().rotation.angle();

        let handle = rbs.insert(rb);

        Bullet {
            handle,
            lifetime: 0.,
            pos,
            angle,
        }
    }
}

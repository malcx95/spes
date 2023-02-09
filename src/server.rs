use std::sync::{Arc, Mutex};
use std::time::Instant;

use postcard::accumulator::{CobsAccumulator, FeedResult};
use quad_net::quad_socket::server::SocketHandle;
use rapier2d::prelude::*;
use unicode_truncate::UnicodeTruncateStr;

use libplen::constants;
use libplen::gamestate;
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientInput, ClientMessage, ServerMessage};
use libplen::physics::PhysicsState;
use libplen::player::ComponentSpecialization;
use libplen::player::Player;

fn send_server_message<'a>(msg: &ServerMessage, handle: &mut SocketHandle<'a>) -> Result<(), ()> {
    let data = postcard::to_stdvec_cobs(msg).expect("Failed to encode message");
    handle.send(&data)
}

struct Client {
    id: Option<u64>,
    input: ClientInput,
    cobs_buf: CobsAccumulator<4096>,
}

impl Default for Client {
    fn default() -> Client {
        Client {
            id: None,
            input: Default::default(),
            cobs_buf: CobsAccumulator::new(),
        }
    }
}

struct Server {
    state: gamestate::GameState,
    next_id: u64,
    last_time: Instant,
    p: PhysicsState,
}

impl Server {
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        /* Create other structures necessary for the simulation. */
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let mut p = PhysicsState {
            rigid_body_set,
            collider_set,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
        };

        Self {
            next_id: 0,
            last_time: Instant::now(),
            state: gamestate::GameState::new(Some(&mut p)),
            p,
        }
    }

    pub fn init_walls(&mut self) {
        for x in -1..2 {
            for y in -1..2 {
                if x == 0 && y == 0 {
                    continue;
                }

                let dx = x as f32;
                let dy = y as f32;

                let rb = RigidBodyBuilder::dynamic().build();

                let collider =
                    ColliderBuilder::cuboid(constants::WORLD_SIZE / 2., constants::WORLD_SIZE / 2.)
                        .mass(0.0)
                        .translation(vector![
                            dx * constants::WORLD_SIZE + constants::WORLD_SIZE / 2.,
                            dy * constants::WORLD_SIZE + constants::WORLD_SIZE / 2.
                        ])
                        .build();

                let body_handle = self.p.rigid_body_set.insert(rb);
                self.p.collider_set.insert_with_parent(
                    collider,
                    body_handle,
                    &mut self.p.rigid_body_set,
                );
            }
        }
    }

    pub fn add_component(&mut self, world_pos: Vec2, specialization: ComponentSpecialization, client_id: u64) {
        for player in self.state.players.iter_mut() {
            if player.id == client_id {
                player.add_component(
                    specialization.clone(),
                    &mut self.p,
                    (world_pos.x, world_pos.y),
                )
            }
        }
    }

    pub fn update(&mut self) {
        self.state.update(constants::DELTA_TIME, &mut self.p);

        self.p.physics_pipeline.step(
            &vector![0., 0.],
            &IntegrationParameters::default(),
            &mut self.p.island_manager,
            &mut self.p.broad_phase,
            &mut self.p.narrow_phase,
            &mut self.p.rigid_body_set,
            &mut self.p.collider_set,
            &mut self.p.impulse_joint_set,
            &mut self.p.multibody_joint_set,
            &mut self.p.ccd_solver,
            None,
            &(),
            &(),
        );

        for player in &mut self.state.players {
            for component in &mut player.components {
                let rb = self
                    .p
                    .rigid_body_set
                    .get(component.physics_handle)
                    .expect("Missing physics rigid body for player {player}");

                let pos = rb.position();

                let trans = pos.translation;
                let rot = pos.rotation;

                component.pos = vec2(trans.x, trans.y);
                component.angle = rot.angle();
            }
        }

        for bullet in &mut self.state.bullets {
            let rb = self.p.rigid_body_set.get(bullet.handle).unwrap();
            let pos = rb.position();
            let trans = pos.translation;

            bullet.pos = vec2(trans.x, trans.y);
            bullet.angle = pos.rotation.angle();
        }
    }
}

fn main() {
    let tcp_addr = "0.0.0.0:4444";
    let ws_addr = "0.0.0.0:4445";

    println!("TCP Listening on {}", tcp_addr);
    println!("WebSocket Listening on {}", ws_addr);

    let dt_duration = std::time::Duration::from_secs_f32(constants::DELTA_TIME);

    let server = Arc::new(Mutex::new(Server::new()));
    //server.init_walls();
    quad_net::quad_socket::server::listen(
        tcp_addr, ws_addr,
        quad_net::quad_socket::server::Settings {
            on_message: {
                let server = server.clone();
                move |out, mut client: &mut Client, msg| {
                    let mut server = server.lock().unwrap();

                    if client.id.is_none() {
                        println!("Got new connection {}", server.next_id);
                        if let Err(_) =
                            send_server_message(&ServerMessage::AssignId(server.next_id), out)
                        {
                            println!("Could not send assign id message");
                            out.disconnect();
                            return;
                        }
                        println!("Sent id {}", server.next_id);
                        client.id = Some(server.next_id);
                        server.next_id += 1;
                    }

                    let client_id = client.id.unwrap();
                    match client.cobs_buf.feed(&msg) {
                        FeedResult::Success { data, remaining: _ } => {
                            match data {
                                ClientMessage::Input(input) => {
                                    client.input = input;
                                }
                                ClientMessage::JoinGame { mut name } => {
                                    if name.trim().len() != 0 {
                                        name = name.trim().unicode_truncate(20).0.to_string()
                                    } else {
                                        name = "Mr Whitespace".into();
                                    }

                                    let mut player = Player::new(client_id, name, &mut server.p);
                                    player.set_num_shield_points(20, &mut server.p);
                                    server.state.add_player(player);
                                }
                                ClientMessage::AddComponent {
                                    world_pos,
                                    specialization,
                                } => {
                                    server.add_component(world_pos, specialization, client_id);
                                }
                            }
                        }
                        FeedResult::DeserError(_) => {
                            println!("Could not decode message from {}", client_id);
                        }
                        FeedResult::OverFull(_) => {
                            print!("Buffer overflow!");
                        }
                        FeedResult::Consumed => {},
                    }
                }
            },
            on_timer: {
                let server = server.clone();
                move |out, client| {
                    let mut server = server.lock().unwrap();
                    if server.last_time.elapsed() > dt_duration {
                        server.update();
                        server.last_time = Instant::now();
                    }

                    if let Err(_) = send_server_message(&ServerMessage::GameState(server.state.clone()), out) {
                        println!("Could not send state to {:?}", client.id);
                        out.disconnect();
                    }
                }
            },
            on_disconnect: {
                let server = server.clone();
                move |client| {
                    if let Some(id) = client.id {
                        println!("Player {} disconnected", id);
                       let mut server = server.lock().unwrap();
                        server.state
                            .players
                            .retain(|player| player.id != id);
                    }
                }
            },
            timer: Some(dt_duration),
            _marker: std::marker::PhantomData,
        }
    );
}

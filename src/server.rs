use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Instant;
use std::vec;

use libplen::math::vec2;
use libplen::player::Component;
use rapier2d::prelude::*;
use unicode_truncate::UnicodeTruncateStr;

use libplen::constants;
use libplen::gamestate;
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage};
use libplen::player::Player;

fn send_bytes(bytes: &[u8], stream: &mut TcpStream) -> io::Result<()> {
    let mut start = 0;
    loop {
        match stream.write(&bytes[start..bytes.len()]) {
            Ok(n) => {
                if n < bytes.len() - start {
                    start = start + n;
                } else {
                    break Ok(());
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => continue,
                io::ErrorKind::Interrupted => continue,
                _ => return Err(e),
            },
        }
    }
}

fn send_server_message(msg: &ServerMessage, stream: &mut TcpStream) -> io::Result<()> {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    send_bytes(&length.to_be_bytes(), stream)?;
    send_bytes(&data, stream)
}

struct Client {
    id: u64,
    message_reader: MessageReader,
    input: ClientInput,
}

struct PhysicsState {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
}

struct Server {
    listener: TcpListener,
    connections: Vec<Client>,
    state: gamestate::GameState,
    next_id: u64,
    last_time: Instant,
    p: PhysicsState,
}

impl Server {
    pub fn new() -> Self {
        let listener = TcpListener::bind("0.0.0.0:4444").unwrap();

        listener.set_nonblocking(true).unwrap();

        println!("Listening on 0.0.0.0:4444");

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

        Self {
            listener,
            connections: vec![],
            next_id: 0,
            last_time: Instant::now(),
            state: gamestate::GameState::new(),
            p: PhysicsState {
                rigid_body_set,
                collider_set,
                physics_pipeline,
                island_manager,
                broad_phase,
                narrow_phase,
                impulse_joint_set,
                multibody_joint_set,
                ccd_solver,
            },
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.last_time.elapsed();
        let delta_time = constants::DELTA_TIME;
        let dt_duration = std::time::Duration::from_millis(constants::SERVER_SLEEP_DURATION);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }
        self.last_time = Instant::now();

        self.state.update(&mut self.p.rigid_body_set, delta_time);

        self.accept_new_connections();
        self.update_clients(delta_time);

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
    }

    fn accept_new_connections(&mut self) {
        // Read data from clients
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    stream.set_nonblocking(true).unwrap();
                    println!("Got new connection {}", self.next_id);
                    if let Err(_) =
                        send_server_message(&ServerMessage::AssignId(self.next_id), &mut stream)
                    {
                        println!("Could not send assign id message");
                        continue;
                    }
                    println!("Sent id {}", self.next_id);
                    self.connections.push(Client {
                        id: self.next_id,
                        message_reader: MessageReader::new(stream),
                        input: ClientInput::new(),
                    });
                    self.next_id += 1;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // wait until network socket is ready, typically implemented
                    // via platform-specific APIs such as epoll or IOCP
                    break;
                }
                e => {
                    e.expect("Socket listener error");
                }
            }
        }
    }

    fn update_clients(&mut self, _delta_time: f32) {
        // Send data to clients
        let mut clients_to_delete = vec![];
        // let mut sounds_to_play = vec![];

        macro_rules! remove_player_on_disconnect {
            ($op:expr, $id:expr) => {
                match $op {
                    Ok(_) => {}
                    Err(e) => match e.kind() {
                        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                            println!("Player {} disconnected", $id);
                            clients_to_delete.push($id);
                            break;
                        }
                        e => panic!("Unhandled network issue: {:?}", e),
                    },
                };
            };
        }

        for client in self.connections.iter_mut() {
            remove_player_on_disconnect!(client.message_reader.fetch_bytes(), client.id);

            for message in client.message_reader.iter() {
                match bincode::deserialize(&message) {
                    Ok(ClientMessage::Input(input)) => {
                        client.input = input;
                    }
                    Ok(ClientMessage::JoinGame { mut name }) => {
                        if name.trim().len() != 0 {
                            name = name.trim().unicode_truncate(20).0.to_string()
                        } else {
                            name = "Mr Whitespace".into();
                        }

                        let p = &mut self.p;
                        let components = [(0., 0.), (0., 200.)]
                            .into_iter()
                            .map(|(x, y)| {
                                let rb = RigidBodyBuilder::dynamic()
                                    .translation(vector![x, y])
                                    .build();

                                let collider = ColliderBuilder::ball(32.).restitution(1.0).build();

                                let body_handle = p.rigid_body_set.insert(rb);
                                p.collider_set.insert_with_parent(
                                    collider,
                                    body_handle,
                                    &mut p.rigid_body_set,
                                );

                                Component {
                                    pos: vec2(x, y),
                                    physics_handle: body_handle,
                                    angle: 0.,
                                }
                            })
                            .collect::<Vec<_>>();

                        let connections = vec![(0, 1)];

                        for (l, r) in connections {
                            let joint = FixedJointBuilder::new()
                                .local_anchor1(point![0.0, 0.0])
                                .local_anchor2(point![0.0, 64.0]);

                            p.impulse_joint_set.insert(
                                components[l].physics_handle,
                                components[r].physics_handle,
                                joint,
                                true,
                            );
                        }

                        let player = Player::new(client.id, name, components);
                        self.state.add_player(player);
                    }
                    Err(_) => {
                        println!("Could not decode message from {}, deleting", client.id);
                        clients_to_delete.push(client.id);
                    }
                }
            }

            let result = send_server_message(
                &ServerMessage::GameState(self.state.clone()),
                &mut client.message_reader.stream,
            );
            remove_player_on_disconnect!(result, client.id);

            for player in &mut self.state.players {
                if player.id == client.id {
                    player.set_input(client.input.x_input, client.input.y_input);
                }
            }
        }

        // for (sound, pos) in &sounds_to_play {
        //     for client in self.connections.iter_mut() {
        //         let result = send_server_message(
        //             &ServerMessage::PlaySound(*sound, *pos),
        //             &mut client.message_reader.stream,
        //         );
        //         remove_player_on_disconnect!(result, client.id);
        //     }
        // }

        self.state
            .players
            .retain(|player| !clients_to_delete.contains(&player.id));
        self.connections
            .retain(|client| !clients_to_delete.contains(&client.id));
    }
}

fn main() {
    let mut server = Server::new();
    loop {
        server.update();
    }
}

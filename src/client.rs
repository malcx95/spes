mod assets;
mod client_state;
mod rendering;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use color_eyre::Result;
use egui::{Align, Layout, Sense};
use egui_macroquad::egui::{self, Color32, Rounding, Stroke};

use assets::Assets;
use libplen::constants::WORLD_SIZE;
use libplen::gamestate;
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage};

use macroquad::prelude::*;

fn send_client_message(msg: &ClientMessage, stream: &mut TcpStream) {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    let length = data.len() as u16;
    stream
        .write(&length.to_be_bytes())
        .expect("Failed to send message length to server");
    stream
        .write(&data)
        .expect("Failed to send message to server");
}

#[allow(unused)]
#[derive(PartialEq)]
enum StateResult {
    Continue,
    GotoNext,
}

struct MainState {
    my_id: u64,
    game_state: gamestate::GameState,
    client_state: client_state::ClientState,
    last_time: Instant,
}

impl MainState {
    fn new(my_id: u64) -> MainState {
        MainState {
            my_id,
            game_state: gamestate::GameState::new(),
            client_state: client_state::ClientState::new(my_id),
            last_time: Instant::now(),
        }
    }

    fn read_input() -> ClientInput {
        let mut x_input = 0.0;
        let mut y_input = 0.0;

        if is_key_down(KeyCode::W) {
            y_input += 1.0;
        }
        if is_key_down(KeyCode::S) {
            y_input -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            x_input -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            x_input += 1.0;
        }

        let (mouse_x, mouse_y) = mouse_position();

        ClientInput {
            x_input,
            y_input,
            mouse_x,
            mouse_y,
        }
    }

    fn update(
        &mut self,
        server_reader: &mut MessageReader,
        extra_messages: &mut Vec<ClientMessage>,
    ) -> StateResult {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();
        let dt_duration = std::time::Duration::from_millis(1000 / 60);
        if elapsed < dt_duration {
            std::thread::sleep(dt_duration - elapsed);
        }

        server_reader.fetch_bytes().unwrap();

        for message in server_reader.iter() {
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::AssignId(_) => panic!("Got new ID after intialisation"),
                ServerMessage::GameState(state) => self.game_state = state,
            }
        }

        let input = Self::read_input();

        self.client_state.update(
            elapsed.as_secs_f32(),
            &mut self.game_state,
            self.my_id,
            extra_messages,
        );

        let input_message = ClientMessage::Input(input);
        send_client_message(&input_message, &mut server_reader.stream);

        if is_key_down(KeyCode::Space) {
            let msg = ClientMessage::Shoot;
            send_client_message(&msg, &mut server_reader.stream);
        }

        StateResult::Continue
    }

    fn draw(&mut self, assets: &mut Assets) -> Result<()> {
        self.client_state
            .draw(self.my_id, &self.game_state, assets)?;

        Ok(())
    }
}

#[macroquad::main("BasicShapes")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let host = std::env::var("SERVER").unwrap_or(String::from("localhost:4444"));
    let stream = TcpStream::connect(host).expect("Could not connect to server");
    println!("Connected to server");

    stream
        .set_nonblocking(true)
        .expect("Could not set socket as nonblocking");
    let mut reader = MessageReader::new(stream);

    let msg = loop {
        reader.fetch_bytes().unwrap();
        if let Some(msg) = reader.iter().next() {
            break bincode::deserialize(&msg).unwrap();
        }
    };

    let mut assets = assets::Assets::new()?;

    let my_id = if let ServerMessage::AssignId(id) = msg {
        println!("Received the id {}", id);
        id
    } else {
        panic!("Expected to get an id from server")
    };

    let mut main_state = MainState::new(my_id);

    let name = whoami::username();

    loop {
        send_client_message(&ClientMessage::JoinGame { name }, &mut reader.stream);

        // let main_state = &mut MainState::new(my_id);
        loop {
            let mut client_messages = vec![];
            main_state.update(&mut reader, &mut client_messages);

            main_state.draw(&mut assets)?;

            // Process keys, mouse etc.

            egui_macroquad::ui(|ctx| {
                egui::TopBottomPanel::bottom("signal select left panel").show(ctx, |ui| {
                    ui.with_layout(
                        Layout::top_down(Align::LEFT).with_cross_justify(true),
                        |ui| {
                            let total_space = ui.available_height();

                            egui::Frame::none().show(ui, |ui| {
                                ui.set_max_height(total_space / 2.);
                                ui.set_min_height(total_space / 2.);

                                ui.heading("Modules");
                                ui.add_space(3.0);

                                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                                    ui.image(
                                        assets.egui_textures.cannon.texture_id(ctx),
                                        egui::Vec2 { x: 64., y: 64. },
                                    )
                                    .interact(egui::Sense {
                                        click: true,
                                        drag: true,
                                        focusable: true,
                                    })
                                    .clicked()
                                    .then(|| println!("Clicked"));
                                });
                            });
                        },
                    )
                });
                egui::Window::new("minimap").show(ctx, |ui| {
                    let (response, painter) =
                        ui.allocate_painter(ui.available_size_before_wrap(), Sense::hover());
                    let Some(player_pos) = main_state
                        .client_state
                        .my_player(main_state.my_id, &main_state.game_state)
                        .map(|p| p.position()) else { return; };
                    let inner = response.rect.shrink(10.);
                    let px = inner.min.x + (inner.width() * (player_pos.x / WORLD_SIZE));
                    let py = inner.min.y + (inner.height() * (player_pos.y / WORLD_SIZE));
                    painter.rect(
                        inner,
                        Rounding::none(),
                        Color32::BLACK,
                        Stroke::new(5., Color32::WHITE),
                    );
                    painter.rect_filled(
                        egui::Rect::from_center_size((px, py).into(), (6., 6.).into()),
                        Rounding::none(),
                        Color32::RED,
                    );
                });
                egui::Window::new("debug").show(ctx, |ui| {
                    let Some(player) = main_state
                        .client_state
                        .my_player(main_state.my_id, &main_state.game_state) else { return; };
                    ui.style_mut().wrap = Some(false);
                    ui.monospace(format!(
                        "player position: x: {:4.0}, y: {:4.0}",
                        player.position().x,
                        player.position().y
                    ));
                    ui.monospace(format!("player velocity: {}", player.velocity()));
                    ui.monospace(format!(
                        "player angle: {:1.3}",
                        player.angle() + std::f32::consts::PI
                    ));
                });
            });

            egui_macroquad::draw();

            next_frame().await;

            while let Some(msg) = client_messages.pop() {
                send_client_message(&msg, &mut reader.stream);
            }
        }
    }
}

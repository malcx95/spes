mod assets;
mod client_state;
mod rendering;

use std::time::Instant;

use anyhow::Result;
use client_state::ClientState;
use egui::{Align, Layout, Sense};
use egui_macroquad::egui::{self, Color32, Painter, Rounding, Stroke, Ui};
use macroquad::prelude::*;
use quad_net::quad_socket::client::QuadSocket;

use assets::Assets;
use libplen::constants::WORLD_SIZE;
use libplen::gamestate;
use libplen::messages::{ClientInput, ClientMessage, ServerMessage};

fn send_client_message<'a>(msg: &ClientMessage, socket: &mut QuadSocket) {
    let data = bincode::serialize(msg).expect("Failed to encode message");
    socket.send(&data);
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
            game_state: gamestate::GameState::new(None),
            client_state: client_state::ClientState::new(my_id),
            last_time: Instant::now(),
        }
    }

    fn read_input(&self) -> ClientInput {
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

        let mouse_left = is_mouse_button_down(MouseButton::Left);
        let mouse_right = is_mouse_button_down(MouseButton::Right);

        let (mouse_x, mouse_y) = mouse_position();

        let shoot = is_key_down(KeyCode::Space);
        let (nmx, nmy) = (
            mouse_x - screen_width() / 2.,
            mouse_y - screen_height() / 2.,
        );
        let aim_angle = nmy.atan2(nmx);

        let shielding = mouse_right && !self.client_state.is_building;

        let mouse_world = self
            .client_state
            .my_player(self.client_state.my_id, &self.game_state)
            .map(|p| ClientState::mouse_world_pos(p));

        ClientInput {
            x_input,
            y_input,
            mouse_x,
            mouse_y,
            mouse_world,
            shoot,
            mouse_left,
            mouse_right,
            shielding,
            aim_angle,
        }
    }

    fn update(
        &mut self,
        socket: &mut QuadSocket,
        extra_messages: &mut Vec<ClientMessage>,
    ) -> StateResult {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();

        while let Some(message) = socket.try_recv() {
            if message.len() == 0 {
                break;
            }
            dbg!(&message);
            match bincode::deserialize(&message).unwrap() {
                ServerMessage::AssignId(_) => panic!("Got new ID after intialisation"),
                ServerMessage::GameState(state) => self.game_state = state,
            }
        }

        let input = self.read_input();

        self.client_state.update(
            elapsed.as_secs_f32(),
            &mut self.game_state,
            self.my_id,
            extra_messages,
        );

        let input_message = ClientMessage::Input(input);
        send_client_message(&input_message, socket);

        StateResult::Continue
    }

    fn draw(&mut self, assets: &mut Assets) -> Result<()> {
        self.client_state
            .draw(self.my_id, &self.game_state, assets)?;

        Ok(())
    }
}

/// Minimap
impl MainState {
    fn draw_minimap_player(
        &self,
        painter: &mut Painter,
        inner: &egui_macroquad::egui::Rect,
        x: f32,
        y: f32,
        color: Color32,
    ) {
        let px = inner.min.x + (inner.width() * (x / WORLD_SIZE));
        let py = inner.min.y + (inner.height() * (y / WORLD_SIZE));
        painter.rect_filled(
            egui_macroquad::egui::Rect::from_center_size((px, py).into(), (3., 3.).into()),
            Rounding::none(),
            color,
        );
    }

    fn draw_minimap_me(&self, painter: &mut Painter, inner: &egui_macroquad::egui::Rect) {
        let Some(player) = self
            .client_state
            .my_player(self.my_id, &self.game_state) else { return; };
        for component in &player.components {
            self.draw_minimap_player(
                painter,
                inner,
                component.pos.x,
                component.pos.y,
                Color32::RED,
            );
        }
    }

    fn draw_minimap_asteroids(&self, painter: &mut Painter, inner: &egui_macroquad::egui::Rect) {
        for asteroid in &self.game_state.asteroids {
            self.draw_minimap_player(painter, inner, asteroid.x, asteroid.y, Color32::GREEN);
        }
    }

    fn draw_minimap_others(&self, painter: &mut Painter, inner: &egui_macroquad::egui::Rect) {
        for player in &self.game_state.players {
            if player.id == self.my_id {
                continue;
            }
            for component in &player.components {
                self.draw_minimap_player(
                    painter,
                    inner,
                    component.pos.x,
                    component.pos.y,
                    Color32::WHITE,
                );
            }
        }
    }

    pub fn draw_minimap(&self, ui: &mut Ui) {
        let (response, mut painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::hover());
        let inner = response.rect.shrink(10.);

        // Background
        painter.rect(
            inner,
            Rounding::none(),
            Color32::BLACK,
            Stroke::new(5., Color32::WHITE),
        );

        self.draw_minimap_me(&mut painter, &inner);
        self.draw_minimap_others(&mut painter, &inner);
        self.draw_minimap_asteroids(&mut painter, &inner);
    }
}

#[macroquad::main("BasicShapes")]
async fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    let mut socket = QuadSocket::connect(&std::env::var("SERVER").unwrap_or(String::from("localhost:4444"))).expect("Could not connect to server");
    #[cfg(target_arch = "wasm32")]
    let mut socket = QuadSocket::connect("ws://localhost:4445").unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        while !socket.is_wasm_websocket_connected() {
            next_frame().await;
        }
    }
    println!("Connected to server");

    // TODO: replace server loop to allow immediate message
    let input_message = ClientMessage::Input(Default::default());
    send_client_message(&input_message, &mut socket);

    let msg = loop {
        if let Some(msg) = socket.try_recv() {
            if msg.len() > 0 {
                break bincode::deserialize(&msg).unwrap();
            }
            next_frame().await;
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

    let name = String::new();

    loop {
        send_client_message(&ClientMessage::JoinGame { name }, &mut socket);

        // let main_state = &mut MainState::new(my_id);
        loop {
            let mut client_messages = vec![];
            main_state.update(&mut socket, &mut client_messages);

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
                    main_state.draw_minimap(ui);
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
                    ui.monospace(format!(
                        "player mouse_x: {}, mouse_y: {}",
                        player.mouse_x, player.mouse_y
                    ));
                });
            });

            egui_macroquad::draw();

            next_frame().await;

            while let Some(msg) = client_messages.pop() {
                send_client_message(&msg, &mut socket);
            }

            next_frame().await
        }
    }
}

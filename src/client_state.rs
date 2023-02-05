use ::rand::seq::SliceRandom;
use ::rand::{thread_rng, Rng};
use color_eyre::Result;
use egui_macroquad::egui::emath::exponential_smooth_factor;
use egui_macroquad::egui::{Color32, Rounding, Sense, Ui};
use libplen::constants::{ASTEROID_SIZE, WORLD_SIZE};
use libplen::gamestate::GameState;
use libplen::messages::ClientMessage;
use libplen::player::{ComponentSpecialization, Player};
use libplen::{constants, math};
use macroquad::prelude::*;

use crate::assets::Assets;

use crate::rendering;

pub struct Star {
    x: f32,
    y: f32,
    star_index: i32,
}

pub struct ClientState {
    pub my_id: u64,
    stars: Vec<Star>,
    stars_material: Material,
    pub is_building: bool,
}

const STARS_VERT: &str = include_str!("./shaders/stars.vert");
const STARS_FRAG: &str = include_str!("./shaders/stars.frag");

impl ClientState {
    pub fn new(my_id: u64) -> ClientState {
        let stars_material = macroquad::material::load_material(
            STARS_VERT,
            STARS_FRAG,
            macroquad::material::MaterialParams {
                pipeline_params: Default::default(),
                uniforms: vec![
                    ("window_dimensions".into(), UniformType::Float2),
                    ("player".into(), UniformType::Float2),
                    ("global_scale".into(), UniformType::Float1),
                ],
                textures: vec![],
            },
        )
        .unwrap();

        ClientState {
            my_id,
            stars: Self::init_stars(),
            stars_material,
            is_building: false,
        }
    }

    fn init_stars() -> Vec<Star> {
        let mut stars = vec![];
        let mut rng = ::rand::thread_rng();
        for _ in 0..constants::NUM_STARS {
            let x = rng.gen_range((-constants::WORLD_SIZE)..(2. * constants::WORLD_SIZE));
            let y = rng.gen_range((-constants::WORLD_SIZE)..(2. * constants::WORLD_SIZE));
            let i = rng.gen_range(0..4);
            stars.push(Star {
                x,
                y,
                star_index: i,
            });
        }
        stars
    }

    pub fn update(
        &mut self,
        _delta_time: f32,
        game_state: &mut GameState,
        my_id: u64,
        client_messages: &mut Vec<ClientMessage>,
    ) {
        let player = self.my_player(my_id, game_state);
        if let Some(p) = player {
            if is_key_pressed(KeyCode::B) {
                self.is_building = !self.is_building;
            }

            let mouse_world_pos = Self::mouse_world_pos(p);
            if self.is_building
                && self.is_valid_component_pos(my_id, game_state, mouse_world_pos)
                && is_mouse_button_pressed(MouseButton::Left)
            {
                println!("Building component");

                client_messages.push(ClientMessage::AddComponent {
                    world_pos: mouse_world_pos,
                    specialization: ComponentSpecialization::addable()
                        .choose(&mut thread_rng())
                        .unwrap()
                        .clone(),
                });
            }
        }
    }

    pub fn is_valid_component_pos(
        &self,
        my_id: u64,
        game_state: &GameState,
        math::Vec2 { x, y }: math::Vec2,
    ) -> bool {
        if let Some(p) = self.my_player(my_id, game_state) {
            let in_range = p
                .components
                .iter()
                .any(|c| (c.pos - math::vec2(x, y)).norm().abs() < constants::MODULE_RADIUS * 4.);
            let overlap = p
                .components
                .iter()
                .any(|c| (c.pos - math::vec2(x, y)).norm().abs() < constants::MODULE_RADIUS * 2.);

            in_range && !overlap
        } else {
            false
        }
    }

    pub fn draw(&mut self, my_id: u64, game_state: &GameState, assets: &Assets) -> Result<()> {
        clear_background(BLACK);

        let player = self.my_player(my_id, game_state);
        if let Some(p) = player {
            if p.requesting_death {
                clear_background(RED);
                draw_text(
                    "u ded, spes no inifity",
                    screen_width() as f32 / 2.0,
                    screen_height() as f32 / 2.0,
                    100.,
                    BLACK,
                );
                draw_text(
                    "pres D for again",
                    screen_width() as f32 / 2.0,
                    screen_height() as f32 / 2.0 + 100.,
                    70.,
                    BLACK,
                );
            } else {
                if whoami::hostname() == "ares" || whoami::hostname() == "spirit" {
                    Self::draw_background(self, p.position().x, p.position().y, p.velocity());
                } else {
                    Self::draw_background2(self, assets, p.position().x, p.position().y);
                }

                let self_pos = p.position();
                let _self_angle = p.angle();

                let center = Vec2::new(
                    screen_width() as f32 / 2.0 - self_pos.x,
                    screen_height() as f32 / 2.0 - self_pos.y,
                );

                Self::draw_bounds(self_pos.x, self_pos.y);

                for asteroid in &game_state.asteroids {
                    let (x, y) = (
                        screen_width() / 2. - self_pos.x + asteroid.x,
                        screen_height() / 2. - self_pos.y + asteroid.y,
                    );
                    rendering::draw_texture_centered_size(
                        assets.malcolm,
                        x,
                        y,
                        asteroid.angle,
                        Vec2::new(ASTEROID_SIZE, ASTEROID_SIZE),
                    );
                }

                for player in &game_state.players {
                    Self::draw_shield(player, self_pos.x, self_pos.y);

                    for component in &player.components {
                        let (x, y) = (center.x + component.pos.x, center.y + component.pos.y);

                        use ComponentSpecialization as CS;
                        let spec = &component.spec;

                        let bg_sprite = match spec {
                            CS::Cannon { .. } => Some(assets.node_bg),
                            CS::Reactionwheel { .. } => Some(assets.reaction_wheel_bot),
                            _ => None,
                        };

                        if let Some(s) = bg_sprite {
                            rendering::draw_texture_centered_size(
                                s,
                                x,
                                y,
                                component.angle,
                                Vec2 { x: 64., y: 64. },
                            );
                        }

                        if let CS::Thrusters = spec {
                            let angle = if player.input_y < -0.5 {
                                0.
                            } else {
                                std::f32::consts::PI
                            };

                            if player.input_y.abs() > 0.5 {
                                rendering::draw_texture_centered_size(
                                    assets.thrust_flame,
                                    x,
                                    y,
                                    component.angle - angle,
                                    Vec2 { x: 128., y: 128. },
                                );
                            }
                        }

                        match spec {
                            CS::Root
                            | CS::Shield
                            | CS::Cannon { aim: false, .. }
                            | CS::Thrusters => {
                                let fg_sprite = match spec {
                                    CS::Root => assets.root_node,
                                    CS::Shield => assets.shield,
                                    CS::Thrusters => assets.thrusters,
                                    CS::Cannon { .. } => assets.cannon,
                                    _ => unreachable!(),
                                };

                                rendering::draw_texture_centered_size(
                                    fg_sprite,
                                    x,
                                    y,
                                    component.angle,
                                    Vec2 { x: 64., y: 64. },
                                );
                            }
                            CS::Reactionwheel { angle } => {
                                rendering::draw_texture_centered_size(
                                    assets.reaction_wheel_mid,
                                    x,
                                    y,
                                    std::f32::consts::PI - angle,
                                    Vec2 { x: 64., y: 64. },
                                );
                                rendering::draw_texture_centered_size(
                                    assets.reaction_wheel_top,
                                    x,
                                    y,
                                    std::f32::consts::PI,
                                    Vec2 { x: 64., y: 64. },
                                );
                            }
                            CS::Cannon { aim: true, .. } => {
                                let angle = player
                                    .mouse_world_pos
                                    .map(|p| (p - component.pos).atan2())
                                    .unwrap_or(0.0);
                                rendering::draw_texture_centered_size(
                                    assets.cannon,
                                    x,
                                    y,
                                    std::f32::consts::PI - angle,
                                    Vec2 { x: 64., y: 64. },
                                );
                            }
                        };

                        let debug = false;
                        if debug {
                            draw_circle_lines(x, y, 64., 1., GREEN);
                            draw_circle_lines(x, y, 32., 1., RED);
                        }
                    }
                }
                for bullet in &game_state.bullets {
                    rendering::draw_texture_centered(
                        assets.bullet,
                        center.x + bullet.pos.x,
                        center.y + bullet.pos.y,
                        bullet.angle,
                    );
                }
                if self.is_building {
                    let (x, y) = mouse_position();

                    if self.is_valid_component_pos(my_id, game_state, Self::mouse_world_pos(p)) {
                        draw_circle_lines(x, y, constants::MODULE_RADIUS, 1., BLUE);
                        draw_circle_lines(x, y, constants::MODULE_RADIUS * 2., 1., PURPLE)
                    } else {
                        draw_circle_lines(x, y, constants::MODULE_RADIUS, 1., ORANGE);
                        draw_circle_lines(x, y, constants::MODULE_RADIUS * 2., 1., ORANGE)
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_shield(player: &Player, self_x: f32, self_y: f32) {
        let pos = player.position();

        for v in &player.shield.points {
            let (x, y) = (
                screen_width() / 2. - self_x + v.x,
                screen_height() / 2. - self_y + v.y,
            );

            let alpha = if player.shielding { 1. } else { 0.1 };
            let color = Color {
                r: 1.,
                g: 1.,
                b: 0.,
                a: alpha,
            };

            draw_circle(x, y, constants::SHIELD_SEGMENT_RADIUS, color);
            println!("Draw {} {}", x, y);
        }
    }

    pub fn mouse_world_pos(p: &Player) -> math::Vec2 {
        let (x, y) = mouse_position();

        p.position() + math::vec2(x, y) - math::vec2(screen_width(), screen_height()) / 2.
    }

    pub fn my_player<'gs>(&self, my_id: u64, game_state: &'gs GameState) -> Option<&'gs Player> {
        game_state.players.iter().find(|p| p.id == my_id)
    }

    fn draw_bounds(player_x: f32, player_y: f32) {
        let sx = screen_width() / 2.;
        let sy = screen_height() / 2.;

        let lines = vec![
            ((0., 0.), (0., constants::WORLD_SIZE)),
            (
                (0., constants::WORLD_SIZE),
                (constants::WORLD_SIZE, constants::WORLD_SIZE),
            ),
            (
                (constants::WORLD_SIZE, constants::WORLD_SIZE),
                (constants::WORLD_SIZE, 0.),
            ),
            ((constants::WORLD_SIZE, 0.), (0., 0.)),
        ];

        for ((x1, y1), (x2, y2)) in lines {
            draw_line(
                sx + x1 - player_x,
                sy + y1 - player_y,
                sx + x2 - player_x,
                sy + y2 - player_y,
                5.,
                GREEN,
            );
        }
    }

    fn draw_background2(
        client_state: &mut ClientState,
        assets: &Assets,
        player_x: f32,
        player_y: f32,
    ) {
        for star in &client_state.stars {
            let star_texture = assets.stars.stars[star.star_index as usize];
            let pivot_x = screen_width() / 2.;
            let pivot_y = screen_height() / 2.;
            rendering::draw_texture_pivot_size(
                star_texture,
                star.x - player_x,
                star.y - player_y,
                0., // -player_angle,
                pivot_x,
                pivot_y,
                40.,
                40.,
            );
        }
    }

    fn draw_background(
        client_state: &mut ClientState,
        player_x: f32,
        player_y: f32,
        player_vel: f32,
    ) {
        let mat = &client_state.stars_material;
        mat.set_uniform("window_dimensions", (screen_width(), screen_height()));
        mat.set_uniform("player", (player_x / 100.0, -player_y / 100.0));
        mat.set_uniform(
            "global_scale",
            1.0 - (exponential_smooth_factor(0.5, 2000.0, player_vel)),
        );
        gl_use_material(*mat);
        draw_cube((0., 0.0, 0.0).into(), (2.0, 2.0, 0.0).into(), None, WHITE);
        gl_use_default_material();
    }
}

use std::collections::VecDeque;
use std::io::{self, prelude::*};
use std::iter::Iterator;
use std::net::TcpStream;

use serde_derive::{Deserialize, Serialize};

use crate::math;

pub struct MessageReader {
    pub stream: TcpStream,
    byte_queue: VecDeque<u8>,
}

pub struct MessageIterator<'a> {
    message_reader: &'a mut MessageReader,
}

impl MessageReader {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            byte_queue: VecDeque::new(),
        }
    }

    pub fn fetch_bytes(&mut self) -> io::Result<()> {
        let mut buffer = [1; 64];
        loop {
            let amount = match self.stream.read(&mut buffer) {
                Ok(amount) => amount,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                e => e?,
            };
            if amount == 0 {
                break Ok(());
            }
            self.byte_queue.extend(buffer.iter().take(amount));
        }
    }

    pub fn iter<'a>(&'a mut self) -> MessageIterator<'a> {
        MessageIterator {
            message_reader: self,
        }
    }
}

impl Iterator for MessageIterator<'_> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        // We need two bytes for the length
        if self.message_reader.byte_queue.len() < 2 {
            return None;
        }

        let length = u16::from_be_bytes([
            self.message_reader.byte_queue[0],
            self.message_reader.byte_queue[1],
        ]) as usize;

        // We will not read a message until a complete message has been
        // received
        if self.message_reader.byte_queue.len() < 2 + length {
            return None;
        }

        self.message_reader.byte_queue.pop_front().unwrap();
        self.message_reader.byte_queue.pop_front().unwrap();

        Some(self.message_reader.byte_queue.drain(0..length).collect())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum SoundEffect {
    Powerup,
    Explosion,
    Gun,
    LaserCharge,
    LaserFire,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    AssignId(u64),
    GameState(crate::gamestate::GameState),
}

#[derive(Serialize, Deserialize)]
pub struct ClientInput {
    pub x_input: f32,
    pub y_input: f32,

    pub mouse_x: f32,
    pub mouse_y: f32,

    pub aim_angle: f32,

    pub mouse_left: bool,
    pub mouse_right: bool,
    pub shielding: bool,
}

impl ClientInput {
    pub fn new() -> Self {
        ClientInput {
            x_input: 0.,
            y_input: 0.,
            mouse_x: 0.,
            mouse_y: 0.,
            aim_angle: 0.,
            mouse_left: false,
            mouse_right: false,
            shielding: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Input(ClientInput),
    AddComponent{world_pos: math::Vec2},
    JoinGame { name: String },
    Shoot,
}

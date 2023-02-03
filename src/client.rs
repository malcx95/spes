mod assets;
mod client_state;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

use libplen::constants;
use libplen::gamestate;
use libplen::math::{vec2, Vec2};
use libplen::messages::{ClientInput, ClientMessage, MessageReader, ServerMessage, SoundEffect};
use assets::Assets;

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
            client_state: client_state::ClientState::new(),
            last_time: Instant::now(),
        }
    }

    fn update(
        &mut self,
        server_reader: &mut MessageReader,
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

        let mut input = ClientInput::new();

        self.client_state
            .update(elapsed.as_secs_f32(), &self.game_state, self.my_id);

        let input_message = ClientMessage::Input(input);
        send_client_message(&input_message, &mut server_reader.stream);

        StateResult::Continue
    }

    fn draw(&mut self, assets: &mut Assets) -> Result<(), String> {
        self.client_state.draw(self.my_id, &self.game_state, assets)?;

        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let host = std::env::var("SERVER").unwrap_or(String::from("localhost:4444"));
    let stream = TcpStream::connect(host).expect("Could not connect to server");
    println!("Connected to server");

    // stream
    //     .set_nonblocking(true)
    //     .expect("Could not set socket as nonblocking");
    let mut reader = MessageReader::new(stream);

    let msg = loop {
        reader.fetch_bytes().unwrap();
        if let Some(msg) = reader.iter().next() {
            break bincode::deserialize(&msg).unwrap();
        }
    };

    let my_id = if let ServerMessage::AssignId(id) = msg {
        println!("Received the id {}", id);
        id
    } else {
        panic!("Expected to get an id from server")
    };

    let mut name = whoami::username();

    loop {
        send_client_message(
            &ClientMessage::JoinGame {
                name: "hej".to_string()
            },
            &mut reader.stream,
        );

        let main_state = &mut MainState::new(my_id);
        'gameloop: loop {

            // game loop here please
            if true {
                break 'gameloop;
            }
        }
    }

}

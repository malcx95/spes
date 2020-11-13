use serde_derive::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
}


impl Player {
    pub fn new(
        id: u64,
        name: String
    ) -> Player {
        Player {
            id: id,
            name: name,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // player update here
    }
}

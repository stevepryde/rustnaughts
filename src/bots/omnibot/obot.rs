use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, PlayerData};
use serde_json;

#[derive(Default)]
pub struct OmniBot {
    player_data: PlayerData,
}

impl OmniBot {
    pub fn new(_game_info: &GameInfo) -> Self {
        let mut data = PlayerData::default();
        data.name = String::from("OmniBot");
        data.is_magic = true;
        OmniBot { player_data: data }
    }
}

impl GameObject for OmniBot {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    fn from_json(&mut self, _data: &serde_json::Value) {}
}

impl GamePlayer for OmniBot {
    fn get_data(&self) -> &PlayerData {
        &self.player_data
    }

    fn get_data_mut(&mut self) -> &mut PlayerData {
        &mut self.player_data
    }

    fn process_magic(&mut self, _inputs: Vec<f32>, available_moves: &[u32]) -> Vec<u32> {
        available_moves.to_vec()
    }
}

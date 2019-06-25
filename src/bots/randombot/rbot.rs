use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, PlayerData};
use rand::Rng;
use serde_json;

#[derive(Default)]
pub struct RandomBot {
  player_data: PlayerData,
}

impl RandomBot {
  pub fn new(_game_info: &GameInfo) -> Self {
    let mut data = PlayerData::default();
    data.name = String::from("RandomBot");
    RandomBot { player_data: data }
  }
}

impl GameObject for RandomBot {
  fn to_json(&self) -> serde_json::Value {
    serde_json::json!({})
  }
  fn from_json(&mut self, _data: &serde_json::Value) {}
}

impl GamePlayer for RandomBot {
  fn get_data(&self) -> &PlayerData {
    &self.player_data
  }

  fn get_data_mut(&mut self) -> &mut PlayerData {
    &mut self.player_data
  }

  fn process(&mut self, _inputs: Vec<f32>, available_moves: &[u32]) -> u32 {
    let idx = rand::thread_rng().gen_range(0, available_moves.len());
    available_moves[idx]
  }
}
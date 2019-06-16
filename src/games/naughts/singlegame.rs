use crate::games::naughts::board::{Board, BoardData};
use crate::engine::gameobject::GameObject;
use crate::engine::gamebase::GameInfo;
use crate::engine::gameplayer::GamePlayer;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize, Serialize)]
struct NaughtsData {
  bot_data: [Value; 2],
  num_turns: [u32; 2],
  current_bot_index: u8,
  board_data: BoardData
}

impl Default for NaughtsData {
  fn default() -> Self {
    NaughtsData {
      bot_data: [json!(null), json!(null)],
      num_turns: [0, 0],
      current_bot_index: 0,
      board_data: BoardData::default()
    }
  }
}

pub struct NaughtsGame {
  identities: [String; 2],
  game_info: GameInfo,
  data: NaughtsData,
  bots: Vec<Box<dyn GamePlayer>>
}

impl Default for NaughtsGame {
  fn default() -> Self {
    NaughtsGame {
      identities: [String::from("X"), String::from("O")],
      game_info: GameInfo {
        input_count: 18,
        output_count: 9
      },
      data: NaughtsData::default(),
      bots: Vec::new()
    }
  }
}


impl NaughtsGame {
  pub fn new() -> Self {
    NaughtsGame {
      identities: [String::from("X"), String::from("O")],
      game_info: GameInfo {
        input_count: 18,
        output_count: 9
      },
      data: NaughtsData::default(),
      bots: Vec::new()
    }
  }
}


impl GameObject<NaughtsData> for NaughtsGame {
  fn get_data(&self) -> &NaughtsData {
    &self.data
  }

  fn set_data(&mut self, data: &NaughtsData) {
    self.data.bot_data[..2].clone_from_slice(&data.bot_data[..2]);
    self.data.num_turns[..2].clone_from_slice(&data.num_turns[..2]);
    self.data.current_bot_index = data.current_bot_index;
    self.data.board_data = data.board_data.clone();
  }
}
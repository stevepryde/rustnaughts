
use crate::engine::botfactory::BotList;
use crate::engine::gamebase::{GameInfo, GameTrait};
use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::{GameResult, GameScore};
use crate::games::naughts::board::Board;

use serde_json;

pub struct NaughtsGame {
  board: Board,
}

impl Default for NaughtsGame {
  fn default() -> Self {
    NaughtsGame {
      board: Board::default(),
    }
  }
}


impl NaughtsGame {
  pub fn new() -> Self {
    NaughtsGame::default()
  }

  fn calculate_score(&self, num_turns: u32, outcome: i8) -> GameScore {
    let score: f32 = 10.0 - num_turns as f32;
    let multiplier: f32 = match outcome {
      x if x > 0 => 1.0,
      x if x < 0 => -10.0,
      _ => 0.0,
    };

    score * multiplier
  }
}

impl GameObject for NaughtsGame {
  fn to_json(&self) -> serde_json::Value {
    self.board.to_json()
  }

  fn from_json(&mut self, value: &serde_json::Value) {
    self.board.from_json(value);
  }
}


impl GameTrait for NaughtsGame {
  fn get_identities(&self) -> [char; 2] {
    ['X', 'O']
  }

  fn get_game_info(&self) -> GameInfo {
    GameInfo {
      input_count: 18,
      output_count: 9,
    }
  }

  fn get_inputs(&self, identity: char) -> (Vec<f32>, Vec<u32>) {
    let mut inputs = Vec::new();
    for pos in 0..9 {
      let c = self.board.getat(pos);
      inputs.push(if c == identity { 1.0 } else { 0.0 });
    }

    for pos in 0..9 {
      let c = self.board.getat(pos);
      inputs.push(if c == identity || c == ' ' { 0.0 } else { 1.0 });
    }

    (inputs, self.board.get_possible_moves())
  }

  fn update(&mut self, identity: char, output: u32) {
    let moves = self.board.get_possible_moves();
    assert!(!moves.is_empty(), "No valid move available: {:?}", moves);

    let target_move = if moves.len() == 1 {
      moves[0]
    } else {
      let mut target = moves[0];
      let mut lowest_diff: Option<u32> = None;
      for m in moves.iter() {
        let diff = (output as i32 - *m as i32).abs() as u32;
        if lowest_diff == None || diff < lowest_diff.unwrap() {
          lowest_diff = Some(diff);
          target = *m;
        }
      }
      target
    };

    self.board.setat(target_move as usize, identity);
    // self.board.show(4);
  }

  fn is_ended(&self) -> bool {
    self.board.is_ended()
  }

  fn get_result(&self, bots: &BotList, num_turns: [u32; 2]) -> GameResult {
    let mut show = false;
    for bot in bots.iter() {
      if bot.should_show_result() {
        show = true;
      }
    }
    if show {
      self.board.show(0);
    }

    let mut result = GameResult::new();
    let outcome = self.board.get_game_state();
    let mut outcomes: [i8; 2] = [0, 0];
    match outcome {
      1 => {
        result.set_win();
        outcomes = [1, -1];
      }
      2 => {
        result.set_win();
        outcomes = [-1, 1];
      }
      3 => {
        result.set_tie();
      }
      _ => {
        panic!("BUG: Invalid game outcome returned: {}", outcome);
      }
    }

    for (i, x) in self.get_identities().iter().enumerate() {
      result.set_score(*x, self.calculate_score(num_turns[i], outcomes[i]))
    }

    result
  }

  fn show(&self, indent: u8) {
    self.board.show(indent);
  }
}
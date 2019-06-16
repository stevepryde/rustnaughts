use crate::games::naughts::board::{Board, BoardData};
use crate::engine::gameobject::GameObject;
use crate::engine::gamebase::{GameBase, GameInfo};
use crate::engine::gameplayer::{GamePlayer, NullPlayer};
use crate::engine::gameresult::{GameScore, GameResult};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize, Serialize)]
struct NaughtsData {
  bot_data: [Value; 2],
  num_turns: [u32; 2],
  current_bot_index: usize,
  board_data: BoardData,
  disqualified: Option<char>
}

impl Default for NaughtsData {
  fn default() -> Self {
    NaughtsData {
      bot_data: [json!(null), json!(null)],
      num_turns: [0, 0],
      current_bot_index: 0,
      board_data: BoardData::default(),
      disqualified: None
    }
  }
}

pub struct NaughtsGame {
  game_info: GameInfo,
  data: NaughtsData,
  board: Board,
  bots: [Box<dyn GamePlayer>; 2]
}

impl Default for NaughtsGame {
  fn default() -> Self {
    NaughtsGame {
      board: Board::default(),
      game_info: GameInfo {
        input_count: 18,
        output_count: 9
      },
      data: NaughtsData::default(),
      bots: [Box::new(NullPlayer::new()), Box::new(NullPlayer::new())]
    }
  }
}


impl NaughtsGame {
  pub fn new() -> Self {
    let game = NaughtsGame::default();
    game.get_data();
    game
  }

  fn calculate_score(&self, num_turns: u32, outcome: i8) -> GameScore {
    let score: f32 = 10.0 - num_turns as f32;
    let multiplier: f32 = match outcome {
      x if x > 0 => 1.0,
      x if x < 0 => -10.0,
      _ => 0.0
    };

    score * multiplier
  }
}

impl GameBase<NaughtsData> for NaughtsGame {
  fn get_identities(&self) -> [char; 2] {
    ['X', 'O']
  }

  fn get_game_info(&self) -> &GameInfo {
    &self.game_info
  }

  fn get_bots(&self) -> &[Box<dyn GamePlayer>; 2] {
    &self.bots
  }

  fn set_bots(&mut self, bots: &[Box<dyn GamePlayer>; 2]) {
    // TODO: rethink how bots work. They should be owned in one place and referenced from there.
    // Also maybe look at GameFactory and do something similar for BotFactory.
    // Use an enum for the bot type, and also perhaps implement a botmanager with the ability to
    // clone bots with state as needed.
    self.bots = bots.clone();
  }

  fn get_current_bot_index(&self) -> usize {
    self.data.current_bot_index
  }

  fn set_current_bot_index(&mut self, index: usize) {
    self.data.current_bot_index = index;
  }

  fn get_num_turns_for_index(&self, index: usize) -> u32 {
    self.data.num_turns[index]
  }

  fn set_num_turns_for_index(&mut self, index: usize, num_turns: u32) {
    self.data.num_turns[index] = num_turns;
  }

  fn set_disqualified(&mut self, identity: char) {
    self.data.disqualified = Some(identity);
  }

  fn get_disqualified(&self) -> Option<char> {
    self.data.disqualified
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

  fn update(&self, identity: char, output: u32) {
    let moves = self.board.get_possible_moves();
    assert!(!moves.is_empty(), "No valid move available!");

    let target_move = if moves.len() == 1 {
      moves[0]
    } else {
      let mut target = moves[0];
      let lowest_diff: Option<u32> = None;
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
  }

  fn is_ended(&self) -> bool {
    self.board.is_ended()
  }

  fn get_result(&self) -> GameResult {
    let show = false;
    for bot in self.bots.iter() {
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
      },
      2 => {
        result.set_win();
        outcomes = [-1, 1];
      },
      3 => {
        result.set_tie();
      },
      _ => {
        panic!("BUG: Invalid game outcome returned: {}", outcome);
      }
    }

    for (i, x) in self.get_identities().iter().enumerate() {
      result.set_score(*x, self.calculate_score(self.data.num_turns[i], outcomes[i]))
    }

    result
  }
}


impl GameObject<NaughtsData> for NaughtsGame {
  fn get_data(&self) -> &NaughtsData {
    self.data.board_data = self.board.get_data().clone();
    &self.data
  }

  fn set_data(&mut self, data: &NaughtsData) {
    self.data.bot_data[..2].clone_from_slice(&data.bot_data[..2]);
    self.data.num_turns[..2].clone_from_slice(&data.num_turns[..2]);
    self.data.current_bot_index = data.current_bot_index;
    self.data.board_data = data.board_data.clone();
    self.board.set_data(&self.data.board_data);
  }
}
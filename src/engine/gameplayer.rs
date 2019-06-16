use crate::engine::gamebase::{GameDisplay, GameInfo};
use crate::engine::gameresult::{GameScore, NULL_SCORE};

pub trait GamePlayerInit {
  fn create(game_info: GameInfo) -> Self;
}

pub trait GamePlayer {
  fn setup(&self) {}
  fn process(&self, inputs: Vec<f32>, available_moves: &Vec<u32>) -> u32 { 0 }
  fn process_magic(&self, inputs: Vec<f32>, available_moves: &Vec<u32>) -> Vec<u32> {
    available_moves.clone()
  }

  fn name(&self) -> &str;
  fn identity(&self) -> char;

  fn score(&self) -> GameScore;
  fn set_score(&mut self, score: GameScore);
  fn mutate(&self) {}
  fn should_show_result(&self) -> bool { false }

  fn clear_score(&mut self) {
    self.set_score(NULL_SCORE);
  }

  fn is_magic(&self) -> bool {
    false
  }

  fn label(&self) -> String {
    format!("{} {}", self.name(), self.identity())
  }
}

pub trait GamePlayerResult {
  fn show_result(&self, game: impl GameDisplay) {}
}

pub struct NullPlayer {
  name: String
}

impl NullPlayer {
  pub fn new() -> Self {
    NullPlayer {
      name: String::from("NullBot")
    }
  }
}
impl GamePlayer for NullPlayer {
  fn name(&self) -> &str {
    ""
  }

  fn identity(&self) -> char {
    'X'
  }

  fn score(&self) -> GameScore {
    -999.0
  }

  fn set_score(&mut self, score: GameScore) {}
}
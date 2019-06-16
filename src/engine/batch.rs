use std::collections::HashMap;
use crate::engine::gameconfig::BatchConfig;
use crate::engine::gameresult::GameResult;

pub struct BatchInput {
  generation: u32,
  sample: u32,
  index: u32,
  genetic_score: f32
}

impl Default for BatchInput {
  fn default() -> Self {
    BatchInput {
      generation: 0,
      sample: 0,
      index: 0,
      genetic_score: 0.0
    }
  }
}

pub struct Batch {
  batch_config: &BatchConfig,
  total_score: HashMap<String, f32>,
  wins: HashMap<String, u32>,
  num_draws: u32,
  identities: [String; 2],
  num_games_played: u32,
  info: BatchInput
}

impl Batch {
  fn new(batch_config: &BatchConfig) -> Self {
    Batch {
      batch_config: batch_config,
      total_score: HashMap<String, f32>::new(),
      wins: HashMap<String, u32>::new(),
      num_draws: 0,
      identities: [String::new(), String::new()],
      num_games_played: 0,
      info: BatchInput::default()
    }
  }

  fn start_batch(&self) {
    let game_obj = GameFactory::get_game_obj(self.batch_config.game);
  }

  pub fn run_batch(&self) -> GameResult {
    self.start_batch();
    if (self.)
  }


}

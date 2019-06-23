use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::{GameScore, NULL_SCORE};

#[derive(Default)]
pub struct PlayerData {
  pub name: String,
  pub identity: char,
  pub score: GameScore,
  pub should_show_result: bool,
  pub is_magic: bool,
  pub is_genetic: bool,
}


pub trait GamePlayer: GameObject {
  fn get_data(&self) -> &PlayerData;
  fn get_data_mut(&mut self) -> &mut PlayerData;

  fn setup(&self, _identity: char) {}
  fn process(&self, _inputs: Vec<f32>, _available_moves: &[u32]) -> u32 {
    0
  }
  fn process_magic(&self, _inputs: Vec<f32>, available_moves: &[u32]) -> Vec<u32> {
    available_moves.to_vec()
  }

  fn get_name(&self) -> &str {
    self.get_data().name.as_str()
  }

  fn get_identity(&self) -> char {
    self.get_data().identity
  }

  fn get_score(&self) -> GameScore {
    self.get_data().score
  }

  fn set_score(&mut self, score: GameScore) {
    self.get_data_mut().score = score;
  }

  fn mutate(&self) {}
  fn should_show_result(&self) -> bool {
    self.get_data().should_show_result
  }

  fn clear_score(&mut self) {
    self.set_score(NULL_SCORE);
  }

  fn is_magic(&self) -> bool {
    self.get_data().is_magic
  }

  fn is_genetic(&self) -> bool {
    self.get_data().is_genetic
  }

  fn label(&self) -> String {
    format!("{} {}", self.get_name(), self.get_identity())
  }
}

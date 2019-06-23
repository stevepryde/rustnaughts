use serde_json;

pub trait GameObject {
  fn to_json(&self) -> serde_json::Value;
  fn from_json(&mut self, data: &serde_json::Value);
}
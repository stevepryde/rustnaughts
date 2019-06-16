use serde_json;

pub trait GameObject<T: Default> {
  fn get_data(&self) -> &T;
  fn set_data(&mut self, data: &T);

  fn to_json(&self) -> serde_json::Value
  where
    T: serde::Serialize,
  {
    serde_json::to_value(self.get_data()).unwrap_or(serde_json::Value::Null)
  }

  fn from_json(&mut self, value: serde_json::Value)
  where
    T: serde::de::DeserializeOwned,
  {
    let data: T = serde_json::from_value(value).unwrap_or(T::default());
    self.set_data(&data);
  }

  fn clone_from(&mut self, other: impl GameObject<T>)
  where
    T: serde::de::DeserializeOwned,
    T: serde::Serialize,
  {
    self.from_json(other.to_json());
  }
}
pub struct GameFactory {}

impl GameFactory {
  pub fn get_game_obj(game: &str) -> impl GameBase {
    match game {
      "naughts" => NaughtsGame::new(),
      _ => {
        panic!("Unknown game: {}", game);
      }
    }
  }
}
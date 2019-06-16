use crate::games::naughts::singlegame::NaughtsGame;

pub enum GameEnum {
  Naughts(NaughtsGame)
}

pub struct GameFactory {}

impl GameFactory {
  pub fn get_game_obj(game: &str) -> GameEnum {
    match game {
      "naughts" => GameEnum::Naughts(NaughtsGame::new()),
      _ => {
        panic!("Unknown game: {}", game);
      }
    }
  }
}
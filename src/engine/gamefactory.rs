use crate::engine::gamebase::GameTrait;
use crate::games::connect4::singlegame::Connect4Game;
use crate::games::naughts::singlegame::NaughtsGame;

#[derive(Debug, Clone)]
pub enum GameType {
    Connect4,
    Naughts,
    Unknown,
}

impl<S> From<S> for GameType
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        let svalue: String = value.into();
        match svalue.to_ascii_lowercase().as_str() {
            "naughts" => GameType::Naughts,
            "connect4" => GameType::Connect4,
            _ => GameType::Unknown,
        }
    }
}

pub type GameFactory = fn() -> Box<dyn GameTrait>;

pub fn create_game_factory(game: &GameType) -> GameFactory {
    match game {
        GameType::Connect4 => || Box::new(Connect4Game::new()),
        GameType::Naughts => || Box::new(NaughtsGame::new()),
        GameType::Unknown => panic!("Unknown game type"),
    }
}

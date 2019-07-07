use crate::engine::gamebase::GameTrait;
use crate::games::connect4::singlegame::Connect4Game;
use crate::games::naughts::singlegame::NaughtsGame;

pub fn create_game(game: &str) -> Box<dyn GameTrait> {
    match game {
        "connect4" => Box::new(Connect4Game::new()),
        "naughts" => Box::new(NaughtsGame::new()),
        _ => {
            panic!("Unknown game: {}", game);
        }
    }
}

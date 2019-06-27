use crate::engine::gamebase::GameTrait;
use crate::games::naughts::singlegame::NaughtsGame;

pub fn create_game(game: &str) -> Box<dyn GameTrait> {
    match game {
        "naughts" => Box::new(NaughtsGame::new()),
        _ => {
            panic!("Unknown game: {}", game);
        }
    }
}

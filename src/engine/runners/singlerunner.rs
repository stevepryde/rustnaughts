use log::info;
use std::error::Error;

use crate::engine::botfactory::BotFactory;
use crate::engine::gamebase::run_one_game;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game_factory;

pub fn single_runner(config: GameConfig) -> Result<(), Box<dyn Error>> {
    let game_factory = create_game_factory(&config.game);
    let game = game_factory();
    let bot_factory = BotFactory::new(game.get_game_info(), config.get_bot_config());
    let result = run_one_game(false, game_factory, &bot_factory);
    info!("{}", result.to_string());
    Ok(())
}

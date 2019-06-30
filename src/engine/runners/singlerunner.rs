use log::*;
use std::error::Error;

use crate::engine::botfactory::create_bots;
use crate::engine::gamebase::run_one_game;
use crate::engine::gameconfig::GameConfig;

pub fn single_runner(config: GameConfig) -> Result<(), Box<Error>> {
    let mut bots = create_bots(&config.get_bot_config());
    let result = run_one_game(config.game.as_str(), true, &mut bots);
    info!("{}", result.to_string());
    Ok(())
}

use crate::engine::botfactory::BotFactory;
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game_factory;
use std::error::Error;

pub fn batch_runner(config: GameConfig) -> Result<(), Box<dyn Error>> {
    let batch_config = config.get_batch_config();
    let game_factory = create_game_factory(&config.game);
    let game = game_factory();
    let bot_factory = BotFactory::new(game.get_game_info(), config.get_bot_config());
    run_batch(&batch_config, true, game_factory, &bot_factory);
    Ok(())
}

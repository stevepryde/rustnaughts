
use log::info;
use std::error::Error;
use crate::engine::botfactory::create_bots;
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::GameConfig;

pub fn batch_runner(config: GameConfig) -> Result<(), Box<Error>> {
    let batch_config = config.get_batch_config();
    let mut bots = create_bots(&batch_config.bot_config);
    if !config.botrecipe.is_null() {
        for bot in &mut bots {
            if bot.is_genetic() {
                info!("Loaded recipe from BotDB");
                bot.from_json(&config.botrecipe)
            }
        }
    }
    run_batch(&batch_config, true, &mut bots);
    Ok(())
}

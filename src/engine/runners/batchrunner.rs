use std::error::Error;

use crate::engine::botfactory::create_bots;
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::GameConfig;

pub fn batch_runner(config: GameConfig) -> Result<(), Box<Error>> {
    let mut bots = create_bots(config.get_bot_config());
    run_batch(&config, &mut bots);
    Ok(())
}

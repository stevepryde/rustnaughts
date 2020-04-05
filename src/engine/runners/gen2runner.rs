// Continuous improvement runner
/*
Basically the idea is to kind of turn the genetic runner from breadth-first to depth-first.
You have a stack containing the current best recipes and their score, plus a counter that starts at 0.

Each thread will:
- Get next bot from the top of the stack as follows:
  - Increment count of top bot
  - if count > X (whatever the number of tries we want is), remove it from the stack and try the next one
- Create a mutant derivative of the bot
- Run the batch.
- If the score is higher, add new mutant to the top of the stack.
- If it's an overall high score - add it to the keep list (do we want to track the top 10?)


Later expansion:
- When things are removed from the stack, add them to the bottom of a second stack.
- The second stack will end up being the new stack to run once all stack items have reached X count.
- order should also be the same if my thinking is correct.
- Now we can continue on with the new stack and reset all counts back to 0.


-- This should find the best bots sooner, while falling back to searching further down the tree
   to hopefully prevent getting stuck on local maxima.
-- If we get stuck close to the bottom (maybe the initial bots need some identifier?) then we
   could generate more random seeds...


-- With some tweaks this bot could run indefinitely which would be great for the online
   one.

-- TODO: add rust web server to run the rust version on the website.
*/

use crate::engine::botfactory::BotFactory;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game_factory;
use crate::engine::runners::genetic::processor::MTBatchProcessor;
use log::*;
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;

#[derive(Debug, Clone)]
struct BotSample {
    recipe: serde_json::Value,
    generation: u32,
    children: u32,
}

impl BotSample {
    pub fn new(recipe: serde_json::Value) -> Self {
        Self {
            recipe,
            generation: 0,
            children: 0,
        }
    }
}

pub fn gen2_runner(config: GameConfig) -> Result<(), Box<dyn Error>> {
    let botdb = config.botdb;
    let botrecipe = &config.botrecipe;
    if !botrecipe.is_null() {
        info!("Loaded recipe from BotDB");
    }
    let genetic_config = config.get_genetic_config();
    let batch_config = genetic_config.batch_config;
    let game_factory = create_game_factory(&genetic_config.game);
    let game = game_factory();
    let game_info = game.get_game_info();
    let bot_factory = BotFactory::new(game_info.clone(), config.get_bot_config());
    let (bot1, bot2) = bot_factory.create_bots();
    let bots = vec![bot1, bot2];
    let identities = game.get_identities();
    let num_samples = genetic_config.num_samples;

    let genetic_index = if bots[0].is_genetic() {
        if bots[1].is_genetic() {
            warn!(
                "GENETICRUNNER: Both bots are genetic. Only first bot ({}) will use the \
                 genetic algorithm",
                bots[0].get_name()
            );
        }

        0
    } else {
        assert!(bots[1].is_genetic(), "Neither bot is a genetic bot!");
        1
    };

    let genetic_name = batch_config.bot_config.bot_names[genetic_index].as_str();
    let genetic_identity = identities[genetic_index];
    let other_index = if genetic_index == 1 { 0 } else { 1 };
    let other_name = batch_config.bot_config.bot_names[other_index].clone();
    let other_bot_data = bots[other_index].to_json();

    let mut best_botid = String::new();

    let mut scores_file = {
        let scores_path = match env::current_exe() {
            Ok(x) => {
                let mut p = x.parent().expect("Error getting parent dir").to_path_buf();
                p.push("scores.csv");
                p
            }
            _ => panic!("Error getting current exe path"),
        };
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(scores_path)
                .expect("Error opening scores.csv"),
        )
    };

    // let processor = MTBatchProcessor::new(
    //     6,
    //     batch_config.clone(),
    //     game_info.clone(),
    //     other_name.clone(),
    //     other_bot_data.clone(),
    //     genetic_index,
    //     genetic_identity,
    // );

    let mut bot_stack: Vec<BotSample> = Vec::new();
    // bot_stack.push(BotSample::new())

    Ok(())
}

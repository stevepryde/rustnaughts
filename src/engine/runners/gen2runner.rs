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
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game_factory;
use crate::engine::gameresult::GameScore;

use log::*;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;

use std::sync::{Arc, Mutex};

use std::{env, thread};

#[derive(Debug, Clone)]
struct BotSample {
    recipe: serde_json::Value,
    generation: u32,
    children: u32,
    score: Option<GameScore>,
}

impl BotSample {
    pub fn new(recipe: serde_json::Value) -> Self {
        Self {
            recipe,
            generation: 0,
            children: 0,
            score: None,
        }
    }

    pub fn new_from(
        old_generation: u32,
        old_score: Option<GameScore>,
        recipe: serde_json::Value,
        new_score: GameScore,
    ) -> Self {
        Self {
            recipe,
            generation: old_generation + 1,
            children: 0,
            // Average the scores to diminish lucky flukes.
            score: match old_score {
                Some(x) => Some((new_score + x) / 2.0),
                None => Some(new_score),
            },
        }
    }

    pub fn null() -> Self {
        Self::new(serde_json::Value::Null)
    }
}

#[derive(Debug)]
struct BotStack {
    stack: Vec<BotSample>,
    child_limit: u32,
    best_score: Option<GameScore>,
    best_recipe: Option<serde_json::Value>,
    scores_csv: Option<File>,
}

impl BotStack {
    pub fn new(child_limit: u32, scores_csv: Option<File>) -> Self {
        Self {
            stack: Vec::new(),
            child_limit,
            best_score: None,
            best_recipe: None,
            scores_csv,
        }
    }

    pub fn push(&mut self, sample: BotSample) {
        self.stack.push(sample);
    }

    pub fn next(&mut self, last_result: BotSample) -> Option<BotSample> {
        let mut sample = match self.stack.pop() {
            Some(x) => x,
            None => {
                return None;
            }
        };

        if last_result.generation <= sample.generation + 1 {
            let keep = match sample.score {
                Some(x) => match last_result.score {
                    Some(y) => y > x, // New score is higher? Keep it.
                    None => false,    // New one doesn't have a score.
                },
                None => last_result.score.is_some(), // Old one doesn't have a score so keep the new one.
            };
            if keep {
                if let Some(x) = last_result.score {
                    let is_high_score = match self.best_score {
                        Some(y) => x > y,
                        None => true,
                    };
                    if is_high_score {
                        info!(
                            "Found new high score: {:.3} (Generation: {})",
                            x, last_result.generation
                        );
                        self.best_score = last_result.score;
                        self.best_recipe = Some(last_result.recipe.clone());
                        if let Some(x) = &mut self.scores_csv {
                            writeln!(x, "{}", last_result.recipe.to_string())
                                .expect("Error writing scores.csv");
                        }
                    }
                }

                // Put the current item back on the stack and use the new one.
                self.stack.push(sample);
                sample = last_result;
            }
        }

        sample.children += 1;
        if sample.children > self.child_limit {
            info!("Dropping generation {}", sample.generation);
            // Just return this one - it has one last chance but is now removed from the stack.
            Some(sample)
        } else {
            // Put it back on the stack and return a clone.
            self.stack.push(sample.clone());
            Some(sample)
        }
    }
}

pub fn gen2_runner(config: GameConfig) -> Result<(), Box<dyn Error>> {
    let botrecipe = &config.botrecipe;
    if !botrecipe.is_null() {
        info!("Loaded recipe from BotDB");
    }
    let genetic_config = config.get_genetic_config();
    let batch_config = genetic_config.batch_config;
    let game_factory = create_game_factory(&genetic_config.game);
    let game = game_factory();
    let game_info = game.get_game_info();
    let mut bot_factory = BotFactory::new(game_info, config.get_bot_config());
    let (bot1, bot2) = bot_factory.create_bots();
    let bots = vec![bot1, bot2];

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

    bot_factory.set_genetic_index(genetic_index);
    let scores_file = {
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

    let bot_stack: Arc<Mutex<BotStack>> = Arc::new(Mutex::new(BotStack::new(
        genetic_config.num_samples,
        scores_file,
    )));
    bot_stack
        .lock()
        .unwrap()
        .push(BotSample::new(bots[genetic_index].to_json()));

    let mut pool = Vec::new();

    for _ in 0..6 {
        let mut factory = bot_factory.clone();
        let thread_batch_config = batch_config.clone();
        let thread_bot_stack = bot_stack.clone();

        let handle = thread::spawn(move || {
            let mut last_result = BotSample::null();
            loop {
                let lr = std::mem::replace(&mut last_result, BotSample::null());
                let recipe = {
                    match thread_bot_stack
                        .lock()
                        .ok()
                        .map(|mut x| x.next(lr))
                        .flatten()
                    {
                        Some(x) => x,
                        None => break,
                    }
                };

                if recipe.recipe.is_null() {
                    break;
                }

                // Mutate this bot.
                let mut bot = factory.create_genetic_bot_with_custom_recipe(&recipe.recipe);
                bot.mutate();
                let new_recipe = bot.to_json();

                // Run batch for this recipe.
                let last_gen = recipe.generation;
                let last_score = recipe.score;
                factory.set_genetic_recipe(new_recipe.clone());
                let batch_result = run_batch(&thread_batch_config, false, game_factory, &factory);
                let genetic_score = if genetic_index == 0 {
                    batch_result.get_score1()
                } else {
                    batch_result.get_score2()
                };
                last_result = BotSample::new_from(last_gen, last_score, new_recipe, genetic_score);
            }
        });

        pool.push(handle);
    }

    for handle in pool {
        if let Err(e) = handle.join() {
            error!("Error joining thread: {:?}", e);
        }
    }

    Ok(())
}

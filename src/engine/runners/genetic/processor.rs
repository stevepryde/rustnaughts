use log::debug;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

use crate::engine::botfactory::BotFactory;
use crate::engine::gamebase::run_batch;
use crate::engine::gameconfig::BatchConfig;
use crate::engine::gamefactory::GameFactory;
use crate::engine::gameresult::GameScore;

#[derive(Default, Debug)]
pub struct GeneticRecipe {
    pub recipe: serde_json::Value,
    pub genetic_score: GameScore,
    pub index: u32,
}

pub fn process_batch(
    batch_config: &BatchConfig,
    game_factory: GameFactory,
    bot_factory: &mut BotFactory,
    sample_recipe: serde_json::Value,
    index: u32,
    genetic_index: usize,
) -> GeneticRecipe {
    bot_factory.set_recipe(genetic_index, sample_recipe.clone());
    let batch_result = run_batch(batch_config, false, game_factory, &bot_factory);
    let genetic_score = if genetic_index == 0 {
        batch_result.get_score1()
    } else {
        batch_result.get_score2()
    };

    GeneticRecipe {
        recipe: sample_recipe,
        genetic_score,
        index,
    }
}

pub trait BatchProcessor {
    fn process_batches(
        &self,
        game_factory: GameFactory,
        bot_factory: &mut BotFactory,
        samples: Vec<serde_json::Value>,
        selected_recipes: &mut Vec<GeneticRecipe>,
        score_threshold: GameScore,
    );
}

pub struct MTBatchProcessor {
    pool: ThreadPool,
    batch_config: BatchConfig,
    genetic_index: usize,
}

impl MTBatchProcessor {
    pub fn new(num_threads: usize, batch_config: BatchConfig, genetic_index: usize) -> Self {
        MTBatchProcessor {
            pool: ThreadPool::new(num_threads),
            batch_config,
            genetic_index,
        }
    }
}

impl BatchProcessor for MTBatchProcessor {
    fn process_batches(
        &self,
        game_factory: GameFactory,
        bot_factory: &mut BotFactory,
        samples: Vec<serde_json::Value>,
        selected_recipes: &mut Vec<GeneticRecipe>,
        score_threshold: GameScore,
    ) {
        let (tx, rx) = channel();
        let mut samples = samples;
        let mut index = 0;
        let sample_count = samples.len();
        while !samples.is_empty() {
            // Each thread wants exclusive access to everything. Easiest way is to clone.
            let sample = samples.pop().expect("Error popping sample");
            let thread_batch_config = self.batch_config.clone();
            let genetic_index = self.genetic_index;
            let mut bot_factory_clone = bot_factory.clone();

            let tx = tx.clone();
            self.pool.execute(move || {
                let item = process_batch(
                    &thread_batch_config,
                    game_factory,
                    &mut bot_factory_clone,
                    sample,
                    index,
                    genetic_index,
                );
                tx.send(item).expect("Error sending batch result");
            });
            index += 1;
        }

        for item in rx.iter().take(sample_count) {
            let mut win = String::new();
            let index = item.index;
            let score = item.genetic_score;
            if score > score_threshold {
                selected_recipes.push(item);
                win.push('*');
            }
            debug!(
                "Completed batch for sample {} :: score = {:.3} {}",
                index, score, win
            );
        }
    }
}

pub struct STBatchProcessor {
    batch_config: BatchConfig,
    genetic_index: usize,
}

impl STBatchProcessor {
    pub fn new(batch_config: BatchConfig, genetic_index: usize) -> Self {
        STBatchProcessor {
            batch_config,
            genetic_index,
        }
    }
}

impl BatchProcessor for STBatchProcessor {
    fn process_batches(
        &self,
        game_factory: GameFactory,
        bot_factory: &mut BotFactory,
        samples: Vec<serde_json::Value>,
        selected_recipes: &mut Vec<GeneticRecipe>,
        score_threshold: GameScore,
    ) {
        let mut samples = samples;
        let mut index = 0;
        while !samples.is_empty() {
            // Each thread wants exclusive access to everything. Easiest way is to clone.
            let sample = samples.pop().expect("Error popping sample");
            let batch_config = self.batch_config.clone();

            let item = process_batch(
                &batch_config,
                game_factory,
                bot_factory,
                sample,
                index,
                self.genetic_index,
            );

            let mut win = String::new();
            let score = item.genetic_score;
            if score > score_threshold {
                selected_recipes.push(item);
                win.push('*');
            }
            debug!(
                "Completed batch for sample {} :: score = {:.3} {}",
                index, score, win
            );
            index += 1;
        }
    }
}

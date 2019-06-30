use log::debug;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

use crate::engine::botfactory::clone_bot;
use crate::engine::gamebase::{run_batch, GameInfo};
use crate::engine::gameconfig::BatchConfig;
use crate::engine::gameplayer::GamePlayer;
use crate::engine::gameresult::GameScore;

#[derive(Default, Debug)]
pub struct GeneticRecipe {
    pub recipe: serde_json::Value,
    pub genetic_score: GameScore,
    pub index: u32,
}

pub fn process_batch(
    batch_config: BatchConfig,
    sample: Box<dyn GamePlayer>,
    index: u32,
    genetic_index: usize,
    genetic_identity: char,
    other_bot: Box<dyn GamePlayer>,
) -> GeneticRecipe {
    let r = sample.to_json();
    let mut bots = if genetic_index == 0 {
        [sample, other_bot]
    } else {
        [other_bot, sample]
    };

    let batch_result = run_batch(&batch_config, false, &mut bots);
    let genetic_score = *batch_result.get_score(genetic_identity).unwrap();

    GeneticRecipe {
        recipe: r,
        genetic_score,
        index,
    }
}

pub trait BatchProcessor {
    fn process_batches(
        &self,
        samples: Vec<Box<dyn GamePlayer>>,
        selected_recipes: &mut Vec<GeneticRecipe>,
        score_threshold: GameScore,
    );
}

pub struct MTBatchProcessor {
    pool: ThreadPool,
    batch_config: BatchConfig,
    game_info: GameInfo,
    other_bot_name: String,
    other_bot_data: serde_json::Value,
    genetic_index: usize,
    genetic_identity: char,
}

impl MTBatchProcessor {
    pub fn new(
        num_threads: usize,
        batch_config: BatchConfig,
        game_info: GameInfo,
        other_bot_name: String,
        other_bot_data: serde_json::Value,
        genetic_index: usize,
        genetic_identity: char,
    ) -> Self {

        MTBatchProcessor {
            pool: ThreadPool::new(num_threads),
            batch_config,
            game_info,
            other_bot_name,
            other_bot_data,
            genetic_index,
            genetic_identity,
        }
    }

}

impl BatchProcessor for MTBatchProcessor {
    fn process_batches(
        &self,
        samples: Vec<Box<dyn GamePlayer>>,
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
            let thread_other_bot = clone_bot(
                self.other_bot_name.as_str(),
                &self.game_info,
                &self.other_bot_data,
            );
            let genetic_index = self.genetic_index;
            let genetic_identity = self.genetic_identity;

            let tx = tx.clone();
            self.pool.execute(move || {
                let item = process_batch(
                    thread_batch_config,
                    sample,
                    index,
                    genetic_index,
                    genetic_identity,
                    thread_other_bot,
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
    game_info: GameInfo,
    other_bot_name: String,
    other_bot_data: serde_json::Value,
    genetic_index: usize,
    genetic_identity: char,
}

impl STBatchProcessor {
    pub fn new(
        batch_config: BatchConfig,
        game_info: GameInfo,
        other_bot_name: String,
        other_bot_data: serde_json::Value,
        genetic_index: usize,
        genetic_identity: char,
    ) -> Self {

        STBatchProcessor {
            batch_config,
            game_info,
            other_bot_name,
            other_bot_data,
            genetic_index,
            genetic_identity,
        }
    }

}

impl BatchProcessor for STBatchProcessor {
    fn process_batches(
        &self,
        samples: Vec<Box<dyn GamePlayer>>,
        selected_recipes: &mut Vec<GeneticRecipe>,
        score_threshold: GameScore,
    ) {
        let mut samples = samples;
        let mut index = 0;
        while !samples.is_empty() {
            // Each thread wants exclusive access to everything. Easiest way is to clone.
            let sample = samples.pop().expect("Error popping sample");
            let other_bot = clone_bot(
                self.other_bot_name.as_str(),
                &self.game_info,
                &self.other_bot_data,
            );
            let batch_config = self.batch_config.clone();

            let item = process_batch(
                batch_config,
                sample,
                index,
                self.genetic_index,
                self.genetic_identity,
                other_bot,
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

use log::{error, info, warn};
use std::cmp;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

use crate::engine::botdb::BotDB;
use crate::engine::botfactory::{create_bot, create_bots, BotListMut};
use crate::engine::gamebase::GameInfo;
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game;
use crate::engine::runners::genetic::processor::{
    BatchProcessor, GeneticRecipe, MTBatchProcessor, STBatchProcessor,
};

fn generate_original_samples(
    bot_name: &str,
    game_info: &GameInfo,
    count: u32,
    recipe: &serde_json::Value,
) -> BotListMut {
    let mut samples_out = Vec::new();
    for _ in 0..count {
        let mut bot = create_bot(bot_name, game_info);
        if !recipe.is_null() {
            bot.from_json(recipe);
        }
        samples_out.push(bot);
    }

    samples_out
}

fn generate_samples(
    bot_name: &str,
    game_info: &GameInfo,
    input_recipes: &[GeneticRecipe],
    num_samples: u32,
) -> BotListMut {
    let mut samples_out = Vec::new();
    for recipe in input_recipes {
        for _ in 0..num_samples {
            let mut bot = create_bot(bot_name, game_info);
            bot.from_json(&recipe.recipe);
            assert_eq!(
                bot.to_json(),
                recipe.recipe,
                "New sample not identical to old sample!"
            );
            bot.mutate();
            if bot.to_json() == recipe.recipe {
                warn!("Sample did not mutate");
            }
            samples_out.push(bot);
        }
    }
    samples_out
}


pub fn filter_samples(selected_recipes: &mut Vec<GeneticRecipe>, keep_samples: usize) {
    // Sort by score.
    selected_recipes.sort_by(|a, b| {
        b.genetic_score
            .partial_cmp(&a.genetic_score)
            .unwrap_or(cmp::Ordering::Equal)
    });
    let keep = cmp::min(keep_samples, selected_recipes.len());
    selected_recipes.drain(keep..);
}

pub fn genetic_runner(config: GameConfig) -> Result<(), Box<Error>> {
    let botdb = config.botdb;
    let botrecipe = &config.botrecipe;
    if !botrecipe.is_null() {
        info!("Loaded recipe from BotDB");
    }
    let genetic_config = config.get_genetic_config();
    let batch_config = genetic_config.batch_config;
    let bots = create_bots(&batch_config.bot_config);
    let game = create_game(genetic_config.game.as_str());
    let game_info = game.get_game_info();
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

    let mut selected_recipes = Vec::new();
    let mut score_threshold = -999.0;

    let mut best_botid = String::new();

    let mut scores_file = if botdb {
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
    } else {
        None
    };

    let processor = MTBatchProcessor::new(
        6,
        batch_config.clone(),
        game_info.clone(),
        other_name.clone(),
        other_bot_data.clone(),
        genetic_index,
        genetic_identity,
    );

    // let processor = STBatchProcessor::new(
    //     batch_config.clone(),
    //     game_info.clone(),
    //     other_name.clone(),
    //     other_bot_data.clone(),
    //     genetic_index,
    //     genetic_identity,
    // );

    for gen in 0..genetic_config.num_generations {
        info!("--------------------------");
        info!("Generation {}:", gen);

        let mut new_samples = if selected_recipes.is_empty() {
            generate_original_samples(genetic_name, &game_info, num_samples, botrecipe)
        } else {
            generate_samples(genetic_name, &game_info, &selected_recipes, num_samples)
        };

        if genetic_config.wild_samples > 0 {
            new_samples.append(&mut generate_original_samples(
                genetic_name,
                &game_info,
                num_samples,
                botrecipe,
            ));
        }

        let recipe_count = selected_recipes.len();
        processor.process_batches(new_samples, &mut selected_recipes, score_threshold);
        if selected_recipes.len() == recipe_count {
            info!(
                "Generation {} :: No improvement - will generate more samples",
                gen
            );
            info!("Current best score: {:.3}", score_threshold);
            if botdb && !best_botid.is_empty() {
                info!("Current best botid: {}", best_botid);
            }
            continue;
        }

        filter_samples(&mut selected_recipes, genetic_config.keep_samples as usize);
        let mut selected_scores = Vec::new();
        for recipe in &selected_recipes {
            if recipe.genetic_score > score_threshold {
                // Lifting the score more slowly avoids getting stuck due to a random fluke
                // increasing it out of reach in one jump.
                score_threshold += (recipe.genetic_score - score_threshold) * 0.2;

                // Write scores somewhere.
                if botdb {
                    match BotDB::new().save_bot(&genetic_name, &recipe.recipe, recipe.genetic_score)
                    {
                        Ok(x) => {
                            info!("BotID {}", x);
                            best_botid = x;
                        }
                        Err(x) => error!("Error saving bot: {}", x),
                    };
                } else if let Some(x) = &mut scores_file {
                    writeln!(x, "{}", recipe.recipe.to_string()).expect("Error writing scores.csv");
                }
            }

            selected_scores.push(recipe.genetic_score);
        }

        info!(
            "Generation {} highest scores: [{}]",
            gen,
            selected_scores
                .iter()
                .map(|x| format!("{:.3}", x))
                .collect::<Vec<String>>()
                .join(",")
        );
    }
    Ok(())
}

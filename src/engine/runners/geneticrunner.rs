use log::*;

use crate::engine::botfactory::{clone_bot, create_bot, create_bots, BotListMut};
use crate::engine::gamebase::{run_batch, GameInfo};
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game;
use crate::engine::gameplayer::GamePlayer;
use crate::engine::gameresult::GameScore;
use std::cmp;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

fn generate_original_samples(bot_name: &str, game_info: &GameInfo, count: u32) -> BotListMut {
  let mut samples_out = Vec::new();
  for _ in 0..count {
    let bot = create_bot(bot_name, game_info);
    samples_out.push(bot);
  }

  samples_out
}

fn generate_samples(
  bot_name: &str,
  game_info: &GameInfo,
  input_recipes: &[(serde_json::Value, GameScore)],
  num_samples: u32,
) -> BotListMut {
  let mut samples_out = Vec::new();
  for recipe in input_recipes {
    let mut sample_bot = create_bot(bot_name, game_info);;
    sample_bot.from_json(&recipe.0);
    samples_out.push(sample_bot);

    for _ in 0..num_samples {
      let mut bot = create_bot(bot_name, game_info);
      bot.from_json(&recipe.0);
      assert_eq!(
        bot.to_json(),
        recipe.0,
        "New sample not identical to old sample!"
      );
      bot.mutate();
      if bot.to_json() == recipe.0 {
        warn!("Sample did not mutate");
      }
      samples_out.push(bot);
    }
  }
  samples_out
}

pub fn process_batches<F>(
  config: &GameConfig,
  new_samples: Vec<Box<dyn GamePlayer>>,
  genetic_index: usize,
  genetic_identity: char,
  score_threshold: GameScore,
  bot_maker: F,
) -> Vec<(serde_json::Value, GameScore)>
where
  F: Fn() -> Box<dyn GamePlayer>,
{
  let mut new_samples = new_samples;
  let mut pool = Vec::new();
  let mut index = 0;
  while !new_samples.is_empty() {
    let sample = match new_samples.pop() {
      Some(x) => x,
      _ => continue,
    };
    let r = sample.to_json();

    let other_bot_cloned = bot_maker();
    let mut bots = if genetic_index == 0 {
      [sample, other_bot_cloned]
    } else {
      [other_bot_cloned, sample]
    };

    let batch_result = run_batch(config, &mut bots);
    let genetic_score = *batch_result.get_score(genetic_identity).unwrap();
    let mut win = String::new();
    if genetic_score > score_threshold {
      pool.push((r, genetic_score));
      win.push('*');
    }
    debug!(
      "Completed batch for sample {} :: score = {:.3} {}",
      index, genetic_score, win
    );
    index += 1;
  }

  pool
}

pub fn genetic_runner(config: GameConfig) -> Result<(), Box<Error>> {
  let bots = create_bots(config.get_bot_config());
  let game = create_game(config.game.as_str());
  let game_info = game.get_game_info();
  let identities = game.get_identities();
  let genetic_config = config.get_genetic_config();
  let bot_config = config.get_bot_config();
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

  let genetic_name = bot_config.bot_names[genetic_index].as_str();
  let genetic_identity = identities[genetic_index];
  let other_index = if genetic_index == 1 { 0 } else { 1 };
  let other_name = bot_config.bot_names[other_index].as_str();

  let mut selected_recipes = Vec::new();
  let mut score_threshold = -999.0;

  let scores_path = match env::current_exe() {
    Ok(x) => {
      let mut p = x.parent().expect("Error getting parent dir").to_path_buf();
      p.push("scores.csv");
      p
    }
    _ => panic!("Error getting current exe path"),
  };
  let mut scores_file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(scores_path)
    .expect("Error opening scores.csv");

  for gen in 0..genetic_config.num_generations {
    info!("--------------------------");
    info!("Generation {}:", gen);

    let mut new_samples = if selected_recipes.is_empty() {
      generate_original_samples(genetic_name, &game_info, num_samples)
    } else {
      generate_samples(genetic_name, &game_info, &selected_recipes, num_samples)
    };

    if genetic_config.wild_samples > 0 {
      new_samples.append(&mut generate_original_samples(
        genetic_name,
        &game_info,
        num_samples,
      ));
    }

    let mut pool = process_batches(
      &config,
      new_samples,
      genetic_index,
      genetic_identity,
      score_threshold,
      || clone_bot(other_name, &game_info, &*bots[other_index]),
    );

    if pool.is_empty() {
      info!(
        "Generation {} :: No improvement - will generate more samples",
        gen
      );
      info!("Current best score: {:.3}", score_threshold);
      continue;
    }

    let diff: i32 = selected_recipes.len() as i32 - pool.len() as i32;
    if diff > 0 {
      let udiff = diff as usize;
      for _ in 0..udiff {
        if let Some(x) = selected_recipes.pop() {
          pool.push(x);
        }
      }
    }
    // Sort by score.
    pool.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(cmp::Ordering::Equal));
    let keep = cmp::min(genetic_config.keep_samples as usize, pool.len());
    selected_recipes.clear();
    for _ in 0..keep {
      if let Some(x) = pool.pop() {
        selected_recipes.push(x);
      }
    }

    let mut selected_scores = Vec::new();
    for recipe in &selected_recipes {
      if recipe.1 > score_threshold {
        // Lifting the score more slowly avoids getting stuck due to a random fluke
        // increasing it out of reach in one jump.
        score_threshold += (recipe.1 - score_threshold) * 0.2;
        writeln!(scores_file, "{}", recipe.0.to_string()).expect("Error writing scores.csv");
      }

      selected_scores.push(recipe.1);
    }

    info!(
      "Generation {} highest scores: {:?}",
      gen,
      selected_scores.iter().map(|x| format!("{:.3}", x))
    );
  }

  Ok(())
}

use log::*;


use crate::engine::botfactory::{clone_bot, create_bot, create_bots, BotList, BotListMut};
use crate::engine::gamebase::{run_batch, GameInfo};
use crate::engine::gameconfig::{BotConfig, GameConfig};
use crate::engine::gamefactory::create_game;
use crate::engine::gameplayer::GamePlayer;
use std::cmp;
use std::error::Error;

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
  input_samples: &BotList,
  num_samples: u32,
) -> BotListMut {
  let mut samples_out = Vec::new();
  for sample in input_samples {
    let mut sample_bot = create_bot(bot_name, game_info);
    let sample_data = sample.to_json();
    sample_bot.from_json(&sample_data);
    samples_out.push(sample_bot);

    for _ in 0..num_samples {
      let mut bot = create_bot(bot_name, game_info);

      bot.from_json(&sample_data);
      assert_eq!(
        bot.to_json(),
        sample_data,
        "New sample not identical to old sample!"
      );
      bot.mutate();
      if bot.to_json() == sample_data {
        warn!("Sample did not mutate");
      }
      samples_out.push(bot);
    }
  }

  samples_out
}

// fn select_samples(keep_samples: u32, sorted_pool: &BotList)  {
//   let keep = cmp::min(keep_samples as usize, sorted_pool.len());
//   sorted_pool[0..keep]
// }

pub fn genetic_runner(config: GameConfig) -> Result<(), Box<Error>> {
  let mut bots = create_bots(config.get_bot_config());
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

  let genetic_label = bots[genetic_index].get_name().to_string();
  let genetic_name = bot_config.bot_names[genetic_index].as_str();
  let genetic_identity = identities[genetic_index];
  let genetic_bot = &bots[genetic_index];
  let other_index = if genetic_index == 1 { 0 } else { 1 };
  let other_bot = &bots[other_index];
  let other_name = bot_config.bot_names[other_index].as_str();

  // TODO: figure out how to use selected_samples properly.
  let mut selected_samples = Vec::new();
  let mut score_threshold = -999.0;

  for gen in 0..genetic_config.num_generations {
    info!("--------------------------");
    info!("Generation {}:", gen);

    let mut new_samples = if selected_samples.is_empty() {
      generate_original_samples(genetic_name, &game_info, num_samples)
    } else {
      generate_samples(genetic_name, &game_info, &selected_samples, num_samples)
    };

    if genetic_config.wild_samples > 0 {
      new_samples.append(&mut generate_original_samples(
        genetic_name,
        &game_info,
        num_samples,
      ));
    }

    let mut pool = Vec::new();
    // let other_bot = clone_bot(genetic_name, &game_info, other_bot);
    //for (index, sample) in new_samples.iter().enumerate() {
    let mut index = 0;
    for sample in new_samples {
      // let mut batch_bots = create_bots(bot_config);

      // let mut sample = &mut new_samples[index];
      let other_bot_cloned = clone_bot(other_name, &game_info, other_bot);
      let mut bots = if genetic_index == 0 {
        [sample, other_bot_cloned]
      } else {
        [other_bot_cloned, sample]
      };

      let batch_result = run_batch(&config, &mut bots);

      let mut sample = create_bot(genetic_name, &game_info);
      // sample.from_json(sample.to_json());
      let genetic_score = *batch_result.get_score(genetic_identity).unwrap();
      sample.set_score(genetic_score);
      pool.push(sample);

      let win = if genetic_score > score_threshold {
        String::from("*")
      } else {
        String::new()
      };
      debug!(
        "Completed batch for sample {} :: score = {:3} {}",
        index, genetic_score, win
      );

      index += 1;
    }

    let mut filtered_pool: Vec<&Box<dyn GamePlayer>> = pool
      .iter()
      .filter(|&bot| bot.get_score() > score_threshold)
      .collect();
    if filtered_pool.is_empty() {
      info!(
        "Generation {} :: No improvement - will generate more samples",
        gen
      );
      info!("Current best score: {:.3}", score_threshold);
      continue;
    }

    let diff: i32 = selected_samples.len() as i32 - filtered_pool.len() as i32;
    if diff > 0 {
      let udiff = diff as usize;
      for sample in &selected_samples[0..udiff] {
        filtered_pool.push(sample);
      }
    }

    filtered_pool.sort_by(|a, b| a.get_score().partial_cmp(&b.get_score()).unwrap());
    let keep = cmp::min(genetic_config.keep_samples as usize, filtered_pool.len());
    selected_samples.clear();
    for i in 0..keep {
      selected_samples.push(*filtered_pool.pop().unwrap());
    }
    // selected_samples = filtered_pool[0..keep].to_vec();

    let mut selected_scores = Vec::new();
    for sample in selected_samples {
      let score = sample.get_score();
      if score > score_threshold {
        score_threshold = score;
        // TODO: Write recipe to file.
      }

      selected_scores.push(score);
    }

    info!("Generation {} highest scores: {:?}", gen, selected_scores);

    // TODO: write scores.csv.
  }

  Ok(())
}
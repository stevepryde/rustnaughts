
use log::*;
use std::collections::HashMap;

use crate::engine::botfactory::{create_bots, BotList};
use crate::engine::gameconfig::GameConfig;
use crate::engine::gamefactory::create_game;
use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::GameResult;

#[derive(Clone)]
pub struct GameInfo {
  pub input_count: u32,
  pub output_count: u32,
}

pub trait GameTrait: GameObject {
  fn get_identities(&self) -> [char; 2];
  fn get_game_info(&self) -> GameInfo;
  fn get_inputs(&self, identity: char) -> (Vec<f32>, Vec<u32>);
  fn update(&mut self, identity: char, output: u32);
  fn is_ended(&self) -> bool;
  fn get_result(&self, bots: &BotList, num_turns: [u32; 2]) -> GameResult;
  fn show(&self, _indent: u8) {}
}

pub fn run_one_game(config: &GameConfig, bots: &mut BotList) -> GameResult {
  let mut game = create_game(config.game.as_str());
  let game_info = game.get_game_info();
  let identities = game.get_identities();
  let mut num_turns: [u32; 2] = [0, 0];

  for (index, bot) in bots.iter().enumerate() {
    num_turns[index] = 0;
    bot.setup(identities[index]);
  }

  let mut bot_index = 0;
  while !game.is_ended() {
    num_turns[bot_index] += 1;
    let (inputs, available_moves) = game.get_inputs(identities[bot_index]);
    assert_eq!(
      inputs.len(),
      game_info.input_count as usize,
      "Game returned the wrong number of inputs!"
    );

    let output = bots[bot_index].process(inputs, &available_moves);
    game.update(identities[bot_index], output);
    if config.run_mode == "SINGLE" {
      game.show(4);
    }

    bot_index = if bot_index == 0 { 1 } else { 0 };
  }

  let result = game.get_result(bots, num_turns);
  for (index, identity) in identities.iter().enumerate() {
    bots[index].set_score(*result.get_score(*identity).unwrap());
  }

  result
}

pub fn run_batch(config: &GameConfig, bots: &mut BotList) -> GameResult {
  let game = create_game(config.game.as_str());
  let identities = game.get_identities();

  let batch_config = config.get_batch_config();
  let mut bot_states = Vec::new();
  for bot in bots.iter() {
    bot_states.push(bot.to_json());
  }

  let mut wins = HashMap::new();
  let mut total_score = [0.0 as f32, 0.0 as f32];
  let mut num_draws: u32 = 0;
  for _ in 0..batch_config.batch_size {
    let mut bots_this_game = create_bots(config.get_bot_config());
    for index in 0..2 {
      bots_this_game[index].from_json(&bot_states[index]);
    }

    let result = run_one_game(config, &mut bots_this_game);
    for index in 0..2 {
      total_score[index] += result.get_score(identities[index]).unwrap();
    }

    if result.is_tie() {
      num_draws += 1;
    } else {
      let winner = result.get_winner();
      let current = *wins.entry(winner).or_insert(0);
      wins.insert(winner, current + 1);
    }
  }

  for index in 0..2 {
    info!(
      "{} WINS: {}",
      bots[index].get_name(),
      wins[&identities[index]]
    );
  }
  info!("DRAW/TIE: {}\n", num_draws);

  let mut final_result = GameResult::new();
  final_result.set_batch();
  info!("Average Scores:");
  for index in 0..2 {
    let avg = total_score[index] as f32 / batch_config.batch_size as f32;
    final_result.set_score(identities[index], avg);
    info!("{}: {:.3}", bots[index].get_name(), avg);
  }

  final_result
}

pub fn run_magic_batch(_config: &GameConfig, _bots: &mut BotList) -> GameResult {
  let mut final_result = GameResult::new();
  final_result.set_batch();
  // info!("Average Scores:");
  // for index in [0..2] {
  //   let avg = total_score[index] / batch_config.batch_size;
  //   final_result.set_score(identities[index], avg);
  //   info!("{}: {:.3}", bots[index].get_name(), avg);
  // }

  final_result
}
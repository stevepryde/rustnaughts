use log::*;
use std::collections::HashMap;

use crate::engine::botfactory::{create_bots, BotList};
use crate::engine::gameconfig::BatchConfig;
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

pub fn run_one_game(game_name: &str, log_output: bool, bots: &mut BotList) -> GameResult {
    let mut game = create_game(game_name);
    let game_info = game.get_game_info();
    let identities = game.get_identities();
    let mut num_turns: [u32; 2] = [0, 0];

    for (index, bot) in bots.iter_mut().enumerate() {
        num_turns[index] = 0;
        let other_index = if index == 0 { 1 } else { 0 };
        bot.setup(identities[index], identities[other_index]);
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
        if log_output {
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

pub fn run_batch(batch_config: &BatchConfig, log_output: bool, bots: &mut BotList) -> GameResult {
    if batch_config.magic {
        return run_magic_batch(batch_config, log_output, bots);
    }

    let game = create_game(batch_config.game.as_str());
    let identities = game.get_identities();

    let mut bot_states = Vec::new();
    for bot in bots.iter() {
        bot_states.push(bot.to_json());
    }

    let mut wins = HashMap::new();
    let mut total_score = [0.0 as f32, 0.0 as f32];
    let mut num_draws: u32 = 0;
    for _ in 0..batch_config.batch_size {
        let mut bots_this_game = create_bots(&batch_config.bot_config);
        for index in 0..2 {
            bots_this_game[index].from_json(&bot_states[index]);
        }

        let result = run_one_game(batch_config.game.as_str(), false, &mut bots_this_game);
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

    if log_output {
        for index in 0..2 {
            info!(
                "{} WINS: {}",
                bots[index].get_name(),
                wins[&identities[index]]
            );
        }
        info!("DRAW/TIE: {}\n", num_draws);
    }

    let mut final_result = GameResult::new();
    final_result.set_batch();

    if log_output {
        info!("Average Scores:");
    }
    for index in 0..2 {
        let avg = total_score[index] as f32 / batch_config.batch_size as f32;
        final_result.set_score(identities[index], avg);

        if log_output {
            info!("{}: {:.3}", bots[index].get_name(), avg);
        }
    }

    final_result
}

struct GameState {
    game_state: serde_json::Value,
    bot_state: serde_json::Value,
    num_turns: [u32; 2],
    bot_index: usize,
}

impl GameState {
    pub fn new(
        game_state: serde_json::Value,
        bot_state: serde_json::Value,
        num_turns: [u32; 2],
        bot_index: usize,
    ) -> Self {
        GameState {
            game_state,
            bot_state,
            num_turns,
            bot_index,
        }
    }
}

pub fn run_magic_batch(
    batch_config: &BatchConfig,
    log_output: bool,
    bots: &mut BotList,
) -> GameResult {
    // Ensure 1 and only 1 bot is magic.
    let magic_index = if bots[0].is_magic() {
        assert!(!bots[1].is_magic(), "Both bots cannot be magic!");
        0
    } else {
        assert!(bots[1].is_magic(), "At least 1 bot must be magic!");
        1
    };
    let other_index = if magic_index == 0 { 1 } else { 0 };

    let mut game = create_game(batch_config.game.as_str());
    let identities = game.get_identities();
    let game_info = game.get_game_info();
    for (index, bot) in bots.iter_mut().enumerate() {
        let other_index = if index == 0 { 1 } else { 0 };
        bot.setup(identities[index], identities[other_index]);
    }

    // Each game state is a tuple containing game state and bot state.
    let initial_state = GameState::new(game.to_json(), bots[other_index].to_json(), [0, 0], 0);
    let mut game_stack = vec![initial_state];

    let mut wins = HashMap::new();
    let mut total_score = [0.0 as f32, 0.0 as f32];
    let mut num_draws: u32 = 0;

    let mut count = 0;
    while !game_stack.is_empty() {
        let state = game_stack.pop().expect("No game state on stack!");
        game.from_json(&state.game_state);
        // TODO: Possibly don't need to do this. Bot state does not change outside of mutate().
        // This magic runner is about 4x faster without this.
        bots[other_index].from_json(&state.bot_state);

        if game.is_ended() {
            count += 1;

            let result = game.get_result(&bots, state.num_turns);
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
        } else {
            let mut new_num_turns = state.num_turns;
            new_num_turns[state.bot_index] += 1;
            let (inputs, available_moves) = game.get_inputs(identities[state.bot_index]);
            assert_eq!(
                inputs.len(),
                game_info.input_count as usize,
                "Game returned the wrong number of inputs!"
            );

            let outputs = if state.bot_index == magic_index {
                bots[state.bot_index].process_magic(inputs, &available_moves)
            } else {
                vec![bots[state.bot_index].process(inputs, &available_moves)]
            };
            let new_bot_index = if state.bot_index == 0 { 1 } else { 0 };
            let bot_state = bots[other_index].to_json();
            for output in outputs {
                game.update(identities[state.bot_index], output);
                // Append new game state to the stack.
                let new_state = GameState::new(
                    game.to_json(),
                    bot_state.clone(),
                    new_num_turns,
                    new_bot_index,
                );
                game_stack.push(new_state);
                game.from_json(&state.game_state);
            }
        }
    }

    if log_output {
        info!("RESULTS:");
        info!("Games Played: {}\n", count);

        for index in 0..2 {
            info!(
                "{} WINS: {}",
                bots[index].get_name(),
                wins.get(&identities[index]).unwrap_or(&0)
            );
        }
        info!("DRAW/TIE: {}\n", num_draws);
    }

    let mut final_result = GameResult::new();
    final_result.set_batch();

    if log_output {
        info!("Average Scores:");
    }
    for index in 0..2 {
        let avg = total_score[index] as f32 / count as f32;
        final_result.set_score(identities[index], avg);

        if log_output {
            info!("{}: {:.3}", bots[index].get_name(), avg);
        }
    }

    final_result
}

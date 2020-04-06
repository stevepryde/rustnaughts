use crate::engine::botfactory::BotFactory;
use crate::engine::gameconfig::BatchConfig;
use crate::engine::gamefactory::GameFactory;
use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::GameResult;
use log::*;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub input_count: u32,
    pub output_count: u32,
}

pub trait GameTrait: GameObject {
    fn get_identities(&self) -> [char; 2];
    fn get_game_info(&self) -> GameInfo;
    fn get_inputs(&self, index: usize) -> (Vec<f32>, Vec<u32>);
    fn update(&mut self, index: usize, output: u32);
    fn is_ended(&self) -> bool;
    fn get_result(&self) -> GameResult;
    fn show(&self, _indent: u8) {}

    fn get_identity(&self, index: usize) -> char {
        assert!(index < 2, "Index out of bounds on get_identity()!");
        self.get_identities()[index]
    }
}

pub fn run_one_game(
    log_output: bool,
    game_factory: GameFactory,
    bot_factory: &BotFactory,
) -> GameResult {
    let mut game = game_factory();
    let identities = game.get_identities();
    let (mut bot1, mut bot2) = bot_factory.create_bots();

    bot1.setup(identities[0], identities[1]);
    bot2.setup(identities[1], identities[0]);

    let mut bot_index = 0;
    while !game.is_ended() {
        let (inputs, available_moves) = game.get_inputs(bot_index);

        let output = match bot_index {
            0 => bot1.process(inputs, &available_moves),
            1 => bot2.process(inputs, &available_moves),
            _ => panic!("Invalid bot index: {}", bot_index),
        };
        game.update(bot_index, output);
        if log_output {
            game.show(4);
        }

        bot_index = if bot_index == 0 { 1 } else { 0 };
    }

    if bot1.should_show_result() || bot2.should_show_result() {
        game.show(4);
    }

    game.get_result()
}

pub fn run_batch(
    batch_config: &BatchConfig,
    log_output: bool,
    game_factory: GameFactory,
    bot_factory: &BotFactory,
) -> GameResult {
    if batch_config.magic {
        return run_magic_batch(log_output, game_factory, bot_factory);
    }

    let game = &game_factory();
    let identities = game.get_identities();

    let mut wins1 = 0;
    let mut wins2 = 0;
    let mut total_score1 = 0.0;
    let mut total_score2 = 0.0;
    let mut num_draws: u32 = 0;
    for _ in 0..batch_config.batch_size {
        let result = run_one_game(false, game_factory, bot_factory);
        total_score1 += result.get_score1();
        total_score2 += result.get_score2();

        match result.get_winner() {
            Some(0) => wins1 += 1,
            Some(1) => wins2 += 1,
            Some(_) => panic!("Invalid winner: {:?}", result.get_winner()),
            None => num_draws += 1,
        }
    }

    let mut final_result = GameResult::new(identities);
    final_result.set_batch();
    final_result.set_score1(total_score1 / batch_config.batch_size as f32);
    final_result.set_score2(total_score2 / batch_config.batch_size as f32);

    if log_output {
        let (bot1_name, bot2_name) = bot_factory.bot_names();
        info!("{} WINS: {}", bot1_name, wins1);
        info!("{} WINS: {}", bot2_name, wins2);
        info!("DRAW/TIE: {}\n", num_draws);

        info!("Average Scores:");
        info!("{}: {:.3}", bot1_name, final_result.get_score1());
        info!("{}: {:.3}", bot2_name, final_result.get_score2());
    }

    final_result
}

struct GameState {
    game_state: serde_json::Value,
    bot_state: serde_json::Value,
    bot_index: usize,
}

impl GameState {
    pub fn new(
        game_state: serde_json::Value,
        bot_state: serde_json::Value,
        bot_index: usize,
    ) -> Self {
        GameState {
            game_state,
            bot_state,
            bot_index,
        }
    }
}

pub fn run_magic_batch(
    log_output: bool,
    game_factory: GameFactory,
    bot_factory: &BotFactory,
) -> GameResult {
    let game = &mut game_factory();
    let game_info = game.get_game_info();
    let identities = game.get_identities();
    let (mut bot1, mut bot2) = bot_factory.create_bots();

    // Ensure 1 and only 1 bot is magic.
    let magic_index = if bot1.is_magic() {
        assert!(!bot1.is_magic(), "Both bots cannot be magic!");
        0
    } else {
        assert!(bot2.is_magic(), "At least 1 bot must be magic!");
        1
    };

    bot1.setup(identities[1], identities[0]);
    bot2.setup(identities[0], identities[1]);

    // Each game state is a tuple containing game state and bot state.
    let magic_json = match magic_index {
        0 => bot1.to_json(),
        1 => bot2.to_json(),
        _ => panic!("Invalid bot index: {}", magic_index),
    };
    let initial_state = GameState::new(game.to_json(), magic_json, 0);
    let mut game_stack = Vec::with_capacity(game_info.output_count as usize * 2);
    game_stack.push(initial_state);

    let mut wins1 = 0;
    let mut wins2 = 0;
    let mut num_draws: u32 = 0;
    let mut total_score1 = 0.0;
    let mut total_score2 = 0.0;

    let mut count = 0;
    while !game_stack.is_empty() {
        let state = game_stack.pop().expect("No game state on stack!");
        game.from_json(&state.game_state);
        // TODO: do we need this? bot state generally doesn't change outside mutate().
        match magic_index {
            0 => bot2.from_json(&state.bot_state),
            1 => bot1.from_json(&state.bot_state),
            _ => panic!("Invalid bot index: {}", magic_index),
        }

        if game.is_ended() {
            count += 1;

            let result = game.get_result();
            total_score1 += result.get_score1();
            total_score2 += result.get_score2();

            match result.get_winner() {
                Some(0) => wins1 += 1,
                Some(1) => wins2 += 1,
                Some(_) => panic!("Unknown winner index: {:?}", result.get_winner()),
                None => num_draws += 1,
            }
        } else {
            let (inputs, available_moves) = game.get_inputs(state.bot_index);
            assert_eq!(
                inputs.len(),
                game_info.input_count as usize,
                "Game returned the wrong number of inputs!"
            );

            let (outputs, magic_json) = match state.bot_index {
                0 => {
                    if state.bot_index == magic_index {
                        (bot1.process_magic(inputs, &available_moves), bot1.to_json())
                    } else {
                        (vec![bot1.process(inputs, &available_moves)], bot2.to_json())
                    }
                }
                1 => {
                    if state.bot_index == magic_index {
                        (bot2.process_magic(inputs, &available_moves), bot2.to_json())
                    } else {
                        (vec![bot2.process(inputs, &available_moves)], bot1.to_json())
                    }
                }
                _ => panic!("Invalid bot index: {}", state.bot_index),
            };
            let new_bot_index = if state.bot_index == 0 { 1 } else { 0 };

            for output in outputs {
                game.update(state.bot_index, output);
                // Append new game state to the stack.
                let new_state = GameState::new(game.to_json(), magic_json.clone(), new_bot_index);
                game_stack.push(new_state);
                game.from_json(&state.game_state);
            }
        }
    }

    let mut final_result = GameResult::new(identities);
    final_result.set_batch();
    final_result.set_score1(total_score1 / count as f32);
    final_result.set_score2(total_score2 / count as f32);

    if log_output {
        let (bot1_name, bot2_name) = bot_factory.bot_names();
        info!("{} WINS: {}", bot1_name, wins1);
        info!("{} WINS: {}", bot2_name, wins2);
        info!("DRAW/TIE: {}\n", num_draws);

        info!("Average Scores:");
        info!("{}: {:.3}", bot1_name, final_result.get_score1());
        info!("{}: {:.3}", bot2_name, final_result.get_score2());
    }

    final_result
}

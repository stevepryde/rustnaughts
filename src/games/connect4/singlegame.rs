use crate::engine::gamebase::{GameInfo, GameTrait};
use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::{GameResult, GameScore};
use crate::games::connect4::world::{World, WorldState};

use serde_json;

pub struct Connect4Game {
    world: World,
    num_turns: [u32; 2],
}

impl Default for Connect4Game {
    fn default() -> Self {
        Connect4Game {
            world: World::default(),
            num_turns: [0, 0],
        }
    }
}

impl Connect4Game {
    pub fn new() -> Self {
        Connect4Game::default()
    }

    fn calculate_score(&self, num_turns: u32, outcome: i8) -> GameScore {
        let score: f32 = 25.0 - num_turns as f32;
        let multiplier: f32 = match outcome {
            x if x > 0 => 1.0,
            x if x < 0 => -10.0,
            _ => 0.0,
        };

        score * multiplier
    }
}

impl GameObject for Connect4Game {
    fn to_json(&self) -> serde_json::Value {
        self.world.to_json()
    }

    fn from_json(&mut self, value: &serde_json::Value) {
        self.world.from_json(value);
    }
}

impl GameTrait for Connect4Game {
    fn get_identities(&self) -> [char; 2] {
        ['X', 'O']
    }

    fn get_game_info(&self) -> GameInfo {
        GameInfo {
            input_count: 98, // 49 for each player.
            output_count: 7,
        }
    }

    fn get_inputs(&self, index: usize) -> (Vec<f32>, Vec<u32>) {
        let mut inputs = Vec::with_capacity(98);
        let identity = self.get_identity(index);
        for row in 0..7 {
            for col in 0..7 {
                let c = self.world.getat(col, row);
                inputs.push(if c == identity { 1.0 } else { 0.0 });
            }
        }

        for row in 0..7 {
            for col in 0..7 {
                let c = self.world.getat(col, row);
                inputs.push(if c == identity || c == ' ' { 0.0 } else { 1.0 });
            }
        }

        (inputs, self.world.get_possible_moves())
    }

    fn update(&mut self, index: usize, output: u32) {
        let moves = self.world.get_possible_moves();
        assert!(!moves.is_empty(), "No valid move available: {:?}", moves);
        let identity = self.get_identity(index);
        self.num_turns[index] += 1;

        let target_move = if moves.len() == 1 {
            moves[0]
        } else {
            let mut target = moves[0];
            let mut lowest_diff: Option<u32> = None;
            for m in moves.iter() {
                let diff = (output as i32 - *m as i32).abs() as u32;
                if lowest_diff == None || diff < lowest_diff.unwrap() {
                    lowest_diff = Some(diff);
                    target = *m;
                }
            }
            target
        };

        self.world.setat(target_move as usize, identity);
    }

    fn is_ended(&self) -> bool {
        self.world.is_ended()
    }

    fn get_result(&self) -> GameResult {
        let mut result = GameResult::new(self.get_identities());
        let outcome = self.world.get_game_state();
        let mut outcomes: [i8; 2] = [0, 0];
        match outcome {
            WorldState::XWin => {
                result.set_win();
                outcomes = [1, -1];
            }
            WorldState::OWin => {
                result.set_win();
                outcomes = [-1, 1];
            }
            WorldState::Draw => {
                result.set_tie();
            }
            _ => {
                panic!("BUG: Invalid game outcome returned: {:?}", outcome);
            }
        }

        result.set_score1(self.calculate_score(self.num_turns[0], outcomes[0]));
        result.set_score2(self.calculate_score(self.num_turns[1], outcomes[1]));
        result
    }

    fn show(&self, indent: u8) {
        self.world.show(indent);
    }
}

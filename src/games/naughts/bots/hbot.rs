use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, PlayerData};
use crate::games::naughts::board::Board;

use log::info;
use serde_json;
use std::io;

#[derive(Default)]
pub struct HumanBot {
    player_data: PlayerData,
    board: Board,
}

impl HumanBot {
    pub fn new(_game_info: &GameInfo) -> Self {
        let mut data = PlayerData::default();
        data.name = String::from("HumanBot");
        data.should_show_result = true;
        HumanBot {
            player_data: data,
            board: Board::default(),
        }
    }
}

impl GameObject for HumanBot {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    fn from_json(&mut self, _data: &serde_json::Value) {}
}

impl GamePlayer for HumanBot {
    fn get_data(&self) -> &PlayerData {
        &self.player_data
    }

    fn get_data_mut(&mut self) -> &mut PlayerData {
        &mut self.player_data
    }

    fn process(&mut self, inputs: Vec<f32>, _available_moves: &[u32]) -> u32 {
        self.board.clear();
        for pos in 0..9 {
            if inputs[pos] > 0.0 {
                self.board.setat(pos, self.get_identity());
            } else if inputs[pos + 9] > 0.0 {
                self.board.setat(pos, self.get_other_identity());
            }
        }

        let moves = self.board.get_possible_moves();
        self.board.show(4);

        let prompt = format!(
            "Possible moves are [{}]",
            moves
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );

        if moves.len() == 1 {
            info!("{} (Automatically choose {})", prompt, moves[0].to_string());
            return moves[0];
        }

        loop {
            println!();
            let mut answer = String::new();
            println!("{}: ", prompt);
            io::stdin()
                .read_line(&mut answer)
                .expect("Error reading user input");
            match answer.trim().parse::<u32>() {
                Ok(x) => {
                    if moves.contains(&x) {
                        return x;
                    }
                }
                _ => continue,
            }
        }
    }
}

use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, PlayerData};
use crate::games::connect4::world::World;

use log::info;
use serde_json;
use std::io;

#[derive(Default)]
pub struct HumanConnect4Bot {
    player_data: PlayerData,
    world: World,
}

impl HumanConnect4Bot {
    pub fn new(_game_info: &GameInfo) -> Self {
        let mut data = PlayerData::default();
        data.name = String::from("HumanConnect4Bot");
        data.should_show_result = true;
        HumanConnect4Bot {
            player_data: data,
            world: World::default(),
        }
    }
}

impl GameObject for HumanConnect4Bot {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    fn from_json(&mut self, _data: &serde_json::Value) {}
}

impl GamePlayer for HumanConnect4Bot {
    fn get_data(&self) -> &PlayerData {
        &self.player_data
    }

    fn get_data_mut(&mut self) -> &mut PlayerData {
        &mut self.player_data
    }

    fn process(&mut self, inputs: Vec<f32>, _available_moves: &[u32]) -> u32 {
        self.world.clear();
        let mut index = 0;
        for row in 0..7 {
            for col in 0..7 {
                if inputs[index] > 0.0 {
                    self.world.setat_raw(col, row, self.get_identity());
                }

                index += 1;
            }
        }

        for row in 0..7 {
            for col in 0..7 {
                if inputs[index] > 0.0 {
                    self.world.setat_raw(col, row, self.get_other_identity());
                }

                index += 1;
            }
        }

        let moves = self.world.get_possible_moves();
        self.world.show(4);

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

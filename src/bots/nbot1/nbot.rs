use crate::bots::nbot1::neurons::NeuronLayer;

use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, MutationIntensity, PlayerData};

use rand::Rng;

pub struct NBot1Config {
    pub nodes_per_layer: usize,
    pub num_hidden_layers: usize,
}

#[derive(Default)]
pub struct NBot1 {
    player_data: PlayerData,
    layers: Vec<NeuronLayer>,
    nodes_per_layer: usize,
    num_layers: usize,
}

impl NBot1 {
    pub fn new(game_info: &GameInfo) -> Self {
        Self::new_with_config(
            game_info,
            NBot1Config {
                nodes_per_layer: game_info.input_count as usize,
                num_hidden_layers: 2,
            },
        )
    }

    pub fn new_with_config(game_info: &GameInfo, config: NBot1Config) -> Self {
        let mut data = PlayerData::default();
        data.name = String::from("NBot1");
        let mut obj = NBot1 {
            player_data: data,
            layers: Vec::new(),
            nodes_per_layer: config.nodes_per_layer,
            num_layers: config.num_hidden_layers + 1,
        };
        obj.create(game_info);
        obj
    }

    fn create(&mut self, game_info: &GameInfo) {
        let mut prev_layer_nodes = game_info.input_count as usize;
        for _ in 0..(self.num_layers - 1) {
            let layer = NeuronLayer::generate(self.nodes_per_layer, prev_layer_nodes);
            prev_layer_nodes = layer.nodes.len();
            self.layers.push(layer);
        }

        let output_layer = NeuronLayer::generate(game_info.output_count as usize, prev_layer_nodes);
        self.layers.push(output_layer);
    }

    fn mutate_single(&mut self, step_scale: f32) {
        let mut rng = rand::thread_rng();
        let layer_index = rng.gen_range(0, self.layers.len());
        let node_index = rng.gen_range(0, self.layers[layer_index].nodes.len());
        let range = 0.5 * step_scale;
        if rng.gen::<bool>() {
            let i = rng.gen_range(
                0,
                self.layers[layer_index].nodes[node_index]
                    .input_weights
                    .len(),
            );
            let weight = self.layers[layer_index].nodes[node_index].input_weights[i];
            self.layers[layer_index].nodes[node_index].input_weights[i] =
                weight + rng.gen_range(-range, range);
        } else {
            let bias = self.layers[layer_index].nodes[node_index].bias;
            self.layers[layer_index].nodes[node_index].bias =
                bias + rng.gen_range(-range, range);
        }
    }
}

impl GameObject for NBot1 {
    fn to_json(&self) -> serde_json::Value {
        let values: Vec<serde_json::Value> = self.layers.iter().map(|x| x.to_json()).collect();
        serde_json::json!({ "layers": values })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        if let Some(serde_json::Value::Array(ref v)) = data.get("layers") {
            if !v.is_empty() {
                self.layers.clear();
                for s in v {
                    let mut layer = NeuronLayer::default();
                    layer.from_json(s);
                    self.layers.push(layer);
                }

                self.num_layers = self.layers.len();
                self.nodes_per_layer = 0;
                if !self.layers.is_empty() {
                    self.nodes_per_layer = self.layers[0].nodes.len();
                }
            }
        }
    }
}

impl GamePlayer for NBot1 {
    fn is_genetic(&self) -> bool {
        true
    }

    fn get_data(&self) -> &PlayerData {
        &self.player_data
    }

    fn get_data_mut(&mut self) -> &mut PlayerData {
        &mut self.player_data
    }

    fn process(&mut self, inputs: Vec<f32>, available_moves: &[u32]) -> u32 {
        let mut outputs = inputs;
        for layer in &self.layers {
            outputs = layer.process(outputs.as_slice());
        }

        // Now sort moves according to the outputs.
        let mut best_move = available_moves[0];
        let mut best_output = f32::NEG_INFINITY;
        for m in available_moves {
            if outputs[*m as usize] > best_output {
                best_move = *m;
                best_output = outputs[*m as usize];
            }
        }

        best_move
    }

    fn mutate(&mut self) {
        self.mutate_single(1.0);
    }

    fn mutate_with_intensity(&mut self, intensity: MutationIntensity) {
        let mut rng = rand::thread_rng();
        if intensity.correlated {
            // Correlated mode: pick one neuron and perturb all its weights + bias together.
            let layer_index = rng.gen_range(0, self.layers.len());
            let node_index = rng.gen_range(0, self.layers[layer_index].nodes.len());
            let range = 0.5 * intensity.step_scale;
            for w in &mut self.layers[layer_index].nodes[node_index].input_weights {
                *w += rng.gen_range(-range, range);
            }
            self.layers[layer_index].nodes[node_index].bias += rng.gen_range(-range, range);
        } else {
            for _ in 0..intensity.count {
                self.mutate_single(intensity.step_scale);
            }
        }
    }

    fn crossover(&mut self, other: &serde_json::Value) {
        if let Some(serde_json::Value::Array(ref other_layers)) = other.get("layers") {
            let mut rng = rand::thread_rng();
            for (i, other_layer_json) in other_layers.iter().enumerate() {
                if i >= self.layers.len() {
                    break;
                }
                if !rng.gen::<bool>() {
                    continue;
                }
                let mut other_layer = NeuronLayer::default();
                other_layer.from_json(other_layer_json);
                if other_layer.nodes.len() == self.layers[i].nodes.len() {
                    self.layers[i] = other_layer;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_roundtrip() {
        let game_info = GameInfo {
            input_count: 18,
            output_count: 9,
        };
        let b = NBot1::new(&game_info);
        let state1 = b.to_json();
        println!("State1 = {}", state1);
        let mut b2 = NBot1::new(&game_info);
        b2.from_json(&state1);

        assert_eq!(b.to_json(), state1, "State was exported the same twice");
        assert_eq!(b2.to_json(), state1, "State was imported correctly");

        assert_eq!(
            b.nodes_per_layer, b2.nodes_per_layer,
            "Nodes per layer is same"
        );
        assert_eq!(b.num_layers, b2.num_layers, "Num layers is same");
    }
}

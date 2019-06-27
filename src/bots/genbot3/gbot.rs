use crate::bots::genbot3::nodes::{get_node_instance, get_random_node_instance, Node, NodeOutput};
use crate::engine::gamebase::GameInfo;
use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::{GamePlayer, PlayerData};

use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Default)]
pub struct GenBot3 {
    player_data: PlayerData,
    nodes: Vec<Node>,
    output_nodes: Vec<NodeOutput>,
}

impl GenBot3 {
    pub fn new(game_info: &GameInfo) -> Self {
        let mut data = PlayerData::default();
        data.name = String::from("GenBot3");
        let mut obj = GenBot3 {
            player_data: data,
            nodes: Vec::new(),
            output_nodes: Vec::new(),
        };
        obj.create(game_info);
        obj
    }

    fn get_recipe(&self) -> String {
        let mut recipe_blocks = Vec::new();
        for node in &self.nodes {
            let mut ingredient_blocks = vec![node.name()];
            for input_index in &node.input_indexes {
                ingredient_blocks.push(input_index.to_string());
            }

            recipe_blocks.push(ingredient_blocks.join(":"));
        }

        for node in &self.output_nodes {
            let mut ingredient_blocks = vec![node.name()];
            for input_index in &node.input_indexes {
                ingredient_blocks.push(input_index.to_string());
            }

            recipe_blocks.push(ingredient_blocks.join(":"));
        }

        recipe_blocks.join(",")
    }

    fn create_from_recipe(&mut self, recipe: &str) {
        self.nodes = Vec::new();
        self.output_nodes = Vec::new();

        let mut node_index = 0;
        let recipe_blocks = recipe.split(',');
        for recipe_block in recipe_blocks {
            let ingredient_blocks: Vec<&str> = recipe_block.split(':').collect();
            let class_name = ingredient_blocks[0];
            if class_name == "NODE_OUTPUT" {
                let mut instance = NodeOutput::default();
                for input_number in &ingredient_blocks[1..] {
                    let index: usize = input_number.parse().expect("Couldn't parse ingredient");
                    instance.add_input_node(index);
                }
                self.output_nodes.push(instance);
            } else {
                let mut instance = get_node_instance(class_name);
                if class_name != "NODE_INPUT" {
                    for input_number in &ingredient_blocks[1..] {
                        let index: usize = input_number.parse().expect("Couldn't parse ingredient");
                        instance.add_input_node(index);
                    }
                }
                instance.index = node_index;
                self.nodes.push(instance);
                node_index += 1;
            }
        }
    }

    fn create(&mut self, game_info: &GameInfo) {
        self.nodes = Vec::new();
        self.output_nodes = Vec::new();

        for n in 0..game_info.input_count {
            let mut instance = get_node_instance("NODE_INPUT");
            instance.index = n as usize;
            self.nodes.push(instance);
        }

        let num_nodes = 100;
        let mut next_index = self.nodes.len();
        for _ in 0..num_nodes {
            let mut instance = get_random_node_instance();
            instance.index = next_index;
            let indexes: Vec<usize> = (0..self.nodes.len()).collect();
            for index in indexes.choose_multiple(&mut rand::thread_rng(), instance.num_inputs) {
                instance.add_input_node(*index);
            }
            self.nodes.push(instance);
            next_index += 1;
        }

        let num_inputs = 20;
        for _ in 0..game_info.output_count {
            let mut instance = NodeOutput::default();
            let indexes: Vec<usize> = (0..self.nodes.len()).collect();
            for index in indexes.choose_multiple(&mut rand::thread_rng(), num_inputs) {
                instance.add_input_node(*index);
            }
            self.output_nodes.push(instance);
        }
    }

    fn mutate_output_node(&mut self) {
        let output_indexes: Vec<usize> = (0..self.output_nodes.len()).collect();
        let mut node_indexes = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if node.input_indexes.is_empty() {
                continue;
            }

            node_indexes.push(index);
        }

        let node_index = *output_indexes
            .choose(&mut rand::thread_rng())
            .expect("No nodes!");

        let index: usize =
            rand::thread_rng().gen_range(0, self.output_nodes[node_index].num_inputs);
        let input_num = *node_indexes
            .choose(&mut rand::thread_rng())
            .expect("No indexes");
        self.output_nodes[node_index].input_indexes[index] = input_num;
    }
}

impl GameObject for GenBot3 {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
          "recipe": self.get_recipe()
        })
    }

    fn from_json(&mut self, data: &serde_json::Value) {
        if let Some(serde_json::Value::String(ref v)) = data.get("recipe") {
            if !v.is_empty() {
                self.create_from_recipe(v);
            }
        }
    }
}

impl GamePlayer for GenBot3 {
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
        let mut outputs = Vec::new();
        let start = inputs.len();
        for input_value in inputs {
            outputs.push(input_value > 0.0);
        }

        // ENGAGE BRAIN.
        let end = self.nodes.len();
        for index in start..end {
            self.nodes[index].update(&outputs.as_slice());
            outputs.push(self.nodes[index].output);
        }

        let mut outputs_final = Vec::new();
        for node in &mut self.output_nodes {
            node.update(&outputs.as_slice());
            outputs_final.push(node.output);
        }

        // Now sort moves according to the outputs.
        let mut best_move = available_moves[0];
        let mut best_output = 0;
        for m in available_moves {
            if outputs_final[*m as usize] > best_output {
                best_move = *m;
                best_output = outputs_final[*m as usize];
            }
        }

        best_move
    }

    fn mutate(&mut self) {
        // Optionally mutate an output node instead.
        if rand::thread_rng().gen::<bool>() {
            self.mutate_output_node();
            return;
        }

        let mut mutable_node_indexes = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if node.input_indexes.is_empty() {
                continue;
            }

            mutable_node_indexes.push(index);
        }

        let node_index = *mutable_node_indexes
            .choose(&mut rand::thread_rng())
            .expect("No nodes!");
        let choice: bool = rand::thread_rng().gen();
        if choice {
            let mut instance = get_random_node_instance();
            instance.index = node_index;
            self.nodes[node_index] = instance;
        }

        let num_inputs = self.nodes[node_index].num_inputs;
        let indexes: Vec<usize> = (0..node_index).collect();
        self.nodes[node_index].input_indexes = Vec::new();
        for index in indexes.choose_multiple(&mut rand::thread_rng(), num_inputs) {
            self.nodes[node_index].add_input_node(*index);
        }
    }
}

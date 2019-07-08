use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::engine::gameobject::GameObject;

pub fn sigmoid(a: f32) -> f32 {
    1.0 / (1.0 + (-a).exp())
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub input_weights: Vec<f32>,
    pub bias: f32,
}

impl Neuron {
    pub fn generate(num_parent_nodes: usize) -> Self {
        let mut rng = rand::thread_rng();
        Neuron {
            input_weights: (0..num_parent_nodes)
                .map(|_| sigmoid(rng.gen_range(-2.0, 2.0)))
                .collect::<Vec<f32>>(),
            bias: sigmoid(rng.gen_range(-2.0, 2.0)),
        }
    }

    pub fn process(&self, outputs: &[f32]) -> f32 {
        let weight_sum: f32 = outputs
            .iter()
            .zip(self.input_weights.iter())
            .map(|(x, y)| x * y)
            .sum();

        self.bias + weight_sum
    }
}

impl GameObject for Neuron {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(Neuron {
            input_weights: self.input_weights.clone(),
            bias: self.bias,
        })
        .unwrap_or_else(|_| serde_json::json!({}))
    }
    fn from_json(&mut self, data: &serde_json::Value) {
        let n: Neuron = serde_json::from_value(data.clone()).unwrap_or_else(|_| Neuron::default());
        self.input_weights = n.input_weights;
        self.bias = n.bias;
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct NeuronLayer {
    pub nodes: Vec<Neuron>,
}

impl NeuronLayer {
    pub fn generate(num_nodes: usize, num_parent_nodes: usize) -> Self {
        let mut nodes = Vec::new();
        for _ in 0..num_nodes {
            nodes.push(Neuron::generate(num_parent_nodes));
        }

        NeuronLayer { nodes }
    }

    pub fn process(&self, parent_outputs: &[f32]) -> Vec<f32> {
        let mut outputs = Vec::new();
        for node in &self.nodes {
            outputs.push(node.process(parent_outputs));
        }
        outputs
    }
}

impl GameObject for NeuronLayer {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(NeuronLayer {
            nodes: self.nodes.clone(),
        })
        .unwrap_or_else(|_| serde_json::json!([]))
    }
    fn from_json(&mut self, data: &serde_json::Value) {
        let n: NeuronLayer =
            serde_json::from_value(data.clone()).unwrap_or_else(|_| NeuronLayer::default());
        self.nodes = n.nodes;
    }
}

use rand::seq::SliceRandom;
use rand::Rng;
pub enum NodeType {
  Input,
  Not,
  And,
  Or,
  Xor,
  Nand,
  Nor,
  Xnor,
}

// TODO: instead of input nodes, have input indexes. This will make the code much simpler!.
pub struct Node {
  pub node_type: NodeType,
  pub num_inputs: usize,
  pub input_indexes: Vec<usize>,
  pub output: bool,
  pub index: usize,
}

impl Default for Node {
  fn default() -> Self {
    Node {
      node_type: NodeType::Input,
      num_inputs: 1,
      input_indexes: Vec::new(),
      output: false,
      index: 0,
    }
  }
}

impl Node {
  pub fn new_typed(node_type: NodeType) -> Self {
    let num_inputs = match node_type {
      NodeType::Not => 1,
      _ => 2,
    };

    Node {
      node_type,
      num_inputs,
      input_indexes: Vec::new(),
      output: false,
      index: 0,
    }
  }

  pub fn add_input_node(&mut self, index: usize) {
    self.input_indexes.push(index);
    self.num_inputs = self.input_indexes.len();
  }

  pub fn process(&self, inputs: &[bool]) -> bool {
    assert_eq!(inputs.len(), self.num_inputs, "Incorrect number of inputs!");
    match self.node_type {
      NodeType::Not => !inputs[0],
      NodeType::And => inputs[0] && inputs[1],
      NodeType::Or => inputs[0] || inputs[1],
      NodeType::Xor => (inputs[0] || inputs[1]) && !(inputs[0] && inputs[1]),
      NodeType::Nand => !(inputs[0] && inputs[1]),
      NodeType::Nor => !(inputs[0] || inputs[1]),
      NodeType::Xnor => !(inputs[0] || inputs[1]) || (inputs[0] && inputs[1]),
      NodeType::Input => panic!("Cannot process input node!"),
    }
  }

  pub fn update(&mut self, outputs: &[bool]) {
    let inputs: Vec<bool> = self.input_indexes.iter().map(|x| outputs[*x]).collect();
    self.output = self.process(inputs.as_slice());
  }

  pub fn name(&self) -> String {
    String::from(match self.node_type {
      NodeType::Not => "NODE_NOT",
      NodeType::And => "NODE_AND",
      NodeType::Or => "NODE_OR",
      NodeType::Xor => "NODE_XOR",
      NodeType::Nand => "NODE_NAND",
      NodeType::Nor => "NODE_NOR",
      NodeType::Xnor => "NODE_XNOR",
      NodeType::Input => "NODE_INPUT",
    })
  }
}

pub struct NodeOutput {
  pub num_inputs: usize,
  pub input_indexes: Vec<usize>,
  pub output: u32,
}

impl Default for NodeOutput {
  fn default() -> Self {
    NodeOutput {
      num_inputs: 0,
      input_indexes: Vec::new(),
      output: 0,
    }
  }
}

impl NodeOutput {
  pub fn add_input_node(&mut self, index: usize) {
    self.input_indexes.push(index);
    self.num_inputs = self.input_indexes.len();
  }

  pub fn process(&self, inputs: &[bool]) -> u32 {
    assert_eq!(inputs.len(), self.num_inputs, "Incorrect number of inputs!");
    inputs.iter().filter(|x| **x).count() as u32
  }

  pub fn update(&mut self, outputs: &[bool]) {
    let inputs: Vec<bool> = self.input_indexes.iter().map(|x| outputs[*x]).collect();
    self.output = self.process(inputs.as_slice());
  }

  pub fn name(&self) -> String {
    String::from("NODE_OUTPUT")
  }
}

pub fn get_node_instance(class_name: &str) -> Node {
  match class_name {
    "NODE_INPUT" => Node::new_typed(NodeType::Input),
    "NODE_NOT" => Node::new_typed(NodeType::Not),
    "NODE_AND" => Node::new_typed(NodeType::And),
    "NODE_OR" => Node::new_typed(NodeType::Or),
    "NODE_XOR" => Node::new_typed(NodeType::Xor),
    "NODE_NAND" => Node::new_typed(NodeType::Nand),
    "NODE_NOR" => Node::new_typed(NodeType::Nor),
    "NODE_XNOR" => Node::new_typed(NodeType::Xnor),
    "NODE_OUTPUT" => panic!("Can't use get_node_instance() on output node"),
    _ => panic!(format!("Invalid class name: {}", class_name)),
  }
}

pub fn get_random_node_instance() -> Node {
  let node_pool = [
    "NODE_NOT",
    "NODE_AND",
    "NODE_OR",
    "NODE_XOR",
    "NODE_NAND",
    "NODE_NOR",
    "NODE_XNOR",
  ];
  let class_name = node_pool.choose(&mut rand::thread_rng()).unwrap();
  get_node_instance(class_name)
}

use crate::engine::gameobject::GameObject;
use crate::engine::gameplayer::GamePlayer;
use crate::engine::gameresult::GameResult;

#[derive(Clone)]
pub struct GameInfo {
  pub input_count: u32,
  pub output_count: u32,
}

pub trait GameDisplay {
  fn show_game(&self);
}

pub trait GameBase<T>: GameObject<T>
where
  T: Default,
  T: serde::Serialize,
  T: serde::de::DeserializeOwned,
{
  fn get_identities(&self) -> [char; 2];
  fn get_game_info(&self) -> &GameInfo;
  fn get_bots(&self) -> &[Box<dyn GamePlayer>; 2];
  fn set_bots(&mut self, bots: &[Box<dyn GamePlayer>; 2]);
  fn get_current_bot_index(&self) -> usize;
  fn set_current_bot_index(&mut self, index: usize);
  fn get_num_turns_for_index(&self, index: usize) -> u32;
  fn set_num_turns_for_index(&mut self, index: usize, num_turns: u32);
  fn set_disqualified(&mut self, identity: char);
  fn get_disqualified(&self) -> Option<char>;

  fn get_inputs(&self, identity: char) -> (Vec<f32>, Vec<u32>);
  fn update(&self, identity: char, output: u32);
  fn is_ended(&self) -> bool;
  fn get_result(&self) -> GameResult;

  fn get_current_identity(&self) -> char {
    self.get_identities()[self.get_current_bot_index()]
  }

  fn start(&mut self, bots: &[Box<dyn GamePlayer>; 2]) {
    assert!(
      self.get_game_info().input_count > 0,
      "Game input_count has not been set!"
    );

    self.set_bots(bots);

    for bot in bots.iter() {
      bot.setup();
    }

    for x in 0..2 {
      self.set_num_turns_for_index(x, 0);
    }
    self.set_current_bot_index(0);
  }

  fn do_turn(&mut self) -> Vec<serde_json::Value> {
    let game_info = self.get_game_info().clone();
    let mut current_index = self.get_current_bot_index();
    let identities = self.get_identities();
    let mut current_identity = identities[current_index];
    let bots = &self.get_bots();
    let bot = &bots[current_index];
    let num_turns = self.get_num_turns_for_index(current_index);
    self.set_num_turns_for_index(current_index, num_turns + 1);

    let (inputs, available_moves) = self.get_inputs(current_identity);
    assert!(
      inputs.len() == game_info.input_count as usize,
      format!(
        "Incorrect number of inputs returned from get_inputs(): Expected {}, got {}",
        game_info.input_count,
        inputs.len()
      )
    );

    let mut output_states = Vec::new();

    if bot.is_magic() {
      let outputs = bot.process_magic(inputs, &available_moves);

      let cur_state = self.to_json();
      for output in outputs.iter() {
        if !&available_moves.contains(output) {
          self.set_disqualified(current_identity);
        }

        self.update(current_identity, *output);
        current_index += 1;
        if current_index > bots.len() {
          current_index = 0;
        }
        self.set_current_bot_index(current_index);
        current_identity = identities[current_index];

        output_states.push(self.to_json().clone());
        self.from_json(cur_state.clone());
      }
    } else {
      let output = bot.process(inputs, &available_moves);
      if !&available_moves.contains(&output) {
        self.set_disqualified(current_identity);
      } else {
        self.update(current_identity, output);

        current_index += 1;
        if current_index > bots.len() {
          current_index = 0;
        }
        self.set_current_bot_index(current_index);
      }
      output_states.push(self.to_json().clone());
    }

    output_states
  }

  fn process_result(&self) -> GameResult {
    let disqualified = self.get_disqualified();
    let identities = self.get_identities();
    let result = if let Some(x) = disqualified {
        let mut r = GameResult::new();
        for identity in &identities {
          r.set_score(*identity, 0.0);
        }
        r.set_win();
        r.set_score(x, -999.0);
        r
    } else {
      self.get_result()
    };

    let mut bots = self.get_bots();
    for i in 0..bots.len() {
      bots[i].set_score(*result.get_score(identities[i]).unwrap());
    }
    result
  }

  fn run(&mut self) -> GameResult {
    while !self.is_ended() && self.get_disqualified() == None {
      self.do_turn();
    }
    self.process_result()
  }
}
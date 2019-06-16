/// The Board struct provides a representation of a naughts and crosses board.
///
/// It consists of 9 characters, 3 lots of 3, reading left to right, top to bottom.
/// - is a blank space. X and O are represented by those letters (uppercase).

use crate::engine::gameobject::GameObject;
use serde::Deserialize;
use serde::Serialize;

/// BoardData contains just the raw data. This is split out so that we can
/// use this with GameObject.
#[derive(Deserialize, Serialize, Clone)]
pub struct BoardData {
  data: String,
}

impl Default for BoardData {
  fn default() -> Self {
    BoardData {
      data: String::from("---------"),
    }
  }
}

pub struct Board {
  board_data: BoardData,
}

impl Default for Board {
  fn default() -> Self {
    Board {
      board_data: BoardData::default(),
    }
  }
}


impl Board {
  /// Create a new Board object.
  pub fn new() -> Self {
    let board_data = BoardData {
      data: String::from("---------"),
    };
    Board { board_data }
  }

  /// Make and return a copy of this Board.
  pub fn copy(&self) -> Self {
    let board_data = BoardData {
      data: self.board_data.data.clone(),
    };
    Board { board_data }
  }

  /// Get the character at the specified position.
  ///
  /// i.e.
  ///  0 | 1 | 2
  /// -----------
  ///  3 | 4 | 5
  /// -----------
  ///  6 | 7 | 8
  pub fn getat(&self, pos: u8) -> char {
    self.board_data.data.chars().nth(pos as usize).unwrap()
  }

  /// Set the character for the specified position.
  pub fn setat(&mut self, pos: u8, turn: char) {
    let mut newdata = if pos > 0 {
      self.board_data.data[..pos as usize].to_string()
    } else {
      String::new()
    };
    newdata.push(turn);
    if pos < (self.board_data.data.len() as u8 - 1) {
      newdata.push_str(&self.board_data.data[pos as usize + 1..]);
    }
    self.board_data.data = newdata;
  }

  /// Display this board on the screen.
  pub fn show(&self, indent: u8) {
    let prefix = format!("{:1$}", " ", indent as usize);
    let mut i = 0;
    for r in 0..3 {
      println!(
        "{} {} | {} | {} ",
        prefix,
        self.getat(i),
        self.getat(i + 1),
        self.getat(i + 2)
      );
      if r < 2 {
        println!("{}-----------", prefix);
      }
      i += 3;
    }
    println!();
  }

  /// Get the current game state as int.
  /// 0 = Not completed.
  /// 1 = X win.
  /// 2 = O win.
  /// 3 = draw.
  pub fn get_game_state(&self) -> u8 {
    let sequences = ["012", "345", "678", "036", "147", "258", "048", "246"];

    for seq in sequences.iter() {
      let mut val = String::new();
      for c in seq.chars() {
        val.push(self.getat(c.to_digit(10).unwrap() as u8));
      }

      if val == "XXX" {
        return 1;
      } else if val == "OOO" {
        return 2;
      }
    }

    let mut is_draw = true;
    for pos in 0..9 {
      let val = self.getat(pos);
      if val != 'X' && val != 'O' {
        is_draw = false;
      }
    }

    if is_draw {
      3
    } else {
      0
    }
  }

  /// Return true if game has ended, otherwise false.
  pub fn is_ended(&self) -> bool {
    self.get_game_state() != 0
  }

  /// Get the winner identity, if there is one.
  pub fn get_winner(&self) -> char {
    let state = self.get_game_state();
    match state {
      1 => 'X',
      2 => 'O',
      _ => ' ',
    }
  }

  /// Helper methods for use in bots.

  /// Return the contents at the specified positions.
  /// For example, if I pass in '012' it will return  a string identifying the contents of the top
  /// row.
  pub fn getat_multi(&self, pos_str: &str) -> String {
    let mut contents = String::new();
    for c in pos_str.chars() {
      contents.push(self.getat(c.to_digit(10).unwrap() as u8));
    }
    contents
  }

  /// Get a copy of the board, rotated 90/180/270 degrees clockwise.
  pub fn get_rotated_board(&self, rotations: u8) -> Self {
    let rot = rotations % 4;

    let mut board_copy = self.copy();
    let transform_map = [6, 3, 0, 7, 4, 1, 8, 5, 2];

    if rot == 0 {
      return board_copy;
    }

    for _ in 0..rot {
      let mut new_data = String::new();
      for pos in transform_map.iter() {
        new_data.push(
          board_copy
            .board_data
            .data
            .chars()
            .nth(*pos as usize)
            .unwrap(),
        );
      }
      board_copy.board_data.data = new_data;
    }

    board_copy
  }

  /// Get the position of the first empty space from the list of positions.
  pub fn get_first_empty_space(&self, positions: &str) -> Option<u8> {
    for c in positions.chars() {
      let pos = c.to_digit(10).unwrap() as u8;
      if self.getat(pos) == ' ' {
        return Some(pos);
      }
    }

    None
  }

  /// Get all possible moves for the specified board.
  pub fn get_possible_moves(&self) -> Vec<u8> {
    let mut moves = Vec::new();
    for index in 0..9 {
      if self.getat(index) == ' ' {
        moves.push(index);
      }
    }
    moves
  }
}

/// GameObject lets us serialise and deserialise the contents as JSON.
impl GameObject<BoardData> for Board {
  fn get_data(&self) -> &BoardData {
    &self.board_data
  }

  fn set_data(&mut self, data: &BoardData) {
    self.board_data = data.clone();
  }
}

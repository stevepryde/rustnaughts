use crate::engine::gameobject::GameObject;

use log::info;
use serde_json;
///
/// It consists of 9 characters, 3 lots of 3, reading left to right, top to bottom.
/// - is a blank space. X and O are represented by those letters (uppercase).
pub struct Board {
    data: String,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            data: String::from("---------"),
        }
    }
}

impl Board {
    /// Create a new Board object.
    pub fn new() -> Self {
        Board::default()
    }

    pub fn clear(&mut self) {
        self.data = String::from("---------")
    }

    /// Make and return a copy of this Board.
    pub fn copy(&self) -> Self {
        Board {
            data: self.data.clone(),
        }
    }

    /// Get the character at the specified position.
    ///
    /// i.e.
    ///  0 | 1 | 2
    /// -----------
    ///  3 | 4 | 5
    /// -----------
    ///  6 | 7 | 8
    pub fn getat(&self, pos: usize) -> char {
        let c = self.data.chars().nth(pos).unwrap();
        if c == '-' {
            ' '
        } else {
            c
        }
    }

    /// Set the character for the specified position.
    pub fn setat(&mut self, pos: usize, turn: char) {
        let mut newdata = if pos > 0 {
            self.data[..pos].to_string()
        } else {
            String::new()
        };
        newdata.push(turn);
        if pos < (self.data.len() - 1) {
            newdata.push_str(&self.data[pos as usize + 1..]);
        }
        self.data = newdata;
    }

    /// Display this board on the screen.
    pub fn show(&self, indent: u8) {
        let prefix = format!("{:1$}", " ", indent as usize);
        let mut i = 0;
        for r in 0..3 {
            info!(
                "{} {} | {} | {} ",
                prefix,
                self.getat(i),
                self.getat(i + 1),
                self.getat(i + 2)
            );
            if r < 2 {
                info!("{}-----------", prefix);
            }
            i += 3;
        }
        info!("");
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
                val.push(self.getat(c.to_digit(10).unwrap() as usize));
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
            contents.push(self.getat(c.to_digit(10).unwrap() as usize));
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
                new_data.push(board_copy.data.chars().nth(*pos as usize).unwrap());
            }
            board_copy.data = new_data;
        }

        board_copy
    }

    /// Get the position of the first empty space from the list of positions.
    pub fn get_first_empty_space(&self, positions: &str) -> Option<usize> {
        for c in positions.chars() {
            let pos = c.to_digit(10).unwrap() as usize;
            if self.getat(pos) == ' ' {
                return Some(pos);
            }
        }

        None
    }

    /// Get all possible moves for the specified board.
    pub fn get_possible_moves(&self) -> Vec<u32> {
        let mut moves = Vec::with_capacity(9);
        for index in 0..9 {
            if self.getat(index) == ' ' {
                moves.push(index as u32);
            }
        }
        moves
    }
}

/// GameObject lets us serialise and deserialise the contents as JSON.
impl GameObject for Board {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
          "data": self.data.clone()
        })
    }

    fn from_json(&mut self, value: &serde_json::Value) {
        if let Some(x) = value.get("data").and_then(|x| x.as_str()) {
            self.data = String::from(x);
        } else {
            self.data = String::from("---------")
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_result() {
        let b = Board::new();
        let state1 = b.to_json();
        println!("State1 = {}", state1);
        let mut b2 = Board::new();
        b2.from_json(&state1);

        assert_eq!(b.to_json(), state1, "State was exported the same twice");
        assert_eq!(b2.to_json(), state1, "State was imported correctly");
    }
}

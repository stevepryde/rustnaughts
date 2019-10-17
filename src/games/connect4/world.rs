use crate::engine::gameobject::GameObject;

use log::info;
use serde_json;

const WORLD_WIDTH: usize = 7;
const WORLD_HEIGHT: usize = 7;

#[derive(Debug)]
pub enum WorldState {
    InProgress,
    XWin,
    OWin,
    Draw,
}

pub struct World {
    data: String,
}

impl Default for World {
    fn default() -> Self {
        World {
            data: World::empty_world(),
        }
    }
}

impl World {
    pub fn empty_world() -> String {
        String::from(" ").repeat(WORLD_WIDTH * WORLD_HEIGHT)
    }

    /// Create a new World object.
    pub fn new() -> Self {
        World::default()
    }

    pub fn clear(&mut self) {
        self.data = World::empty_world();
    }

    /// Make and return a copy of this World.
    pub fn copy(&self) -> Self {
        World {
            data: self.data.clone(),
        }
    }

    pub fn getindex(&self, col: usize, row: usize) -> usize {
        (row * WORLD_HEIGHT) + col
    }

    fn getrow(&self, row: usize) -> &str {
        let start = row * WORLD_HEIGHT;
        let end = start + 7;
        &self.data[start..end]
    }

    /// Get the character at the specified col/row.
    pub fn getat(&self, col: usize, row: usize) -> char {
        let pos = self.getindex(col, row);
        let c = self.data.chars().nth(pos).unwrap();
        if c == '-' {
            ' '
        } else {
            c
        }
    }

    /// Set the character for the specified col/row, without gravity.
    pub fn setat_raw(&mut self, col: usize, row: usize, turn: char) {
        let pos = self.getindex(col, row);
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

    /// Drop character into the specified column, with gravity.
    pub fn setat(&mut self, col: usize, turn: char) {
        let mut row = 0;
        for r in (0..7).rev() {
            let c = self.getat(col, r);
            if c != ' ' {
                assert!(r < 6, "Cannot place character at position {}", col);
                row = r + 1;
                break;
            }
        }

        self.setat_raw(col, row, turn);
    }

    /// Display this board on the screen.
    pub fn show(&self, indent: u8) {
        let prefix = format!("{:1$}", " ", indent as usize);
        let divider = String::from("-").repeat(27);
        for r in (0..7).rev() {
            if r < 6 {
                info!("{}|{}|", prefix, divider);
            }

            let rowvec: Vec<String> = self.getrow(r).chars().map(|x| x.to_string()).collect();
            info!("{}| {} |", prefix, rowvec.join(" | "));
        }
        info!("{}\\{}/", prefix, divider);
        info!("");
    }

    pub fn get_game_state(&self) -> WorldState {
        for row in 0..7 {
            for col in 0..7 {
                let c = self.getat(col, row);
                if c == ' ' {
                    continue;
                }

                // Start search. Note that we only need to search to the right, up and both upward
                // diagonals.
                if col < 4
                    && self.getat(col + 1, row) == c
                    && self.getat(col + 2, row) == c
                    && self.getat(col + 3, row) == c
                {
                    return if c == 'X' {
                        WorldState::XWin
                    } else {
                        WorldState::OWin
                    };
                }

                // UP.
                if row > 3 {
                    // Can't have 4-in-a-row if the bottom piece is > row 3.
                    continue;
                }

                if self.getat(col, row + 1) == c
                    && self.getat(col, row + 2) == c
                    && self.getat(col, row + 3) == c
                {
                    return if c == 'X' {
                        WorldState::XWin
                    } else {
                        WorldState::OWin
                    };
                }

                // Diagonal left and up.
                if col > 2
                    && self.getat(col - 1, row + 1) == c
                    && self.getat(col - 2, row + 2) == c
                    && self.getat(col - 3, row + 3) == c
                {
                    return if c == 'X' {
                        WorldState::XWin
                    } else {
                        WorldState::OWin
                    };
                }

                // Diagonal right and up.
                if col < 4
                    && self.getat(col + 1, row + 1) == c
                    && self.getat(col + 2, row + 2) == c
                    && self.getat(col + 3, row + 3) == c
                {
                    return if c == 'X' {
                        WorldState::XWin
                    } else {
                        WorldState::OWin
                    };
                }
            }
        }

        if self.data.contains(' ') {
            WorldState::InProgress
        } else {
            WorldState::Draw
        }
    }

    pub fn is_ended(&self) -> bool {
        match self.get_game_state() {
            WorldState::InProgress => false,
            _ => true,
        }
    }

    pub fn get_winner(&self) -> Option<char> {
        match self.get_game_state() {
            WorldState::InProgress => panic!("get_winner() called while game still in progress!"),
            WorldState::XWin => Some('X'),
            WorldState::OWin => Some('O'),
            WorldState::Draw => None,
        }
    }

    pub fn get_possible_moves(&self) -> Vec<u32> {
        let mut v = Vec::with_capacity(7);
        for (i, c) in self.getrow(6).chars().enumerate() {
            if c == ' ' {
                v.push(i as u32);
            }
        }
        v
    }
}

/// GameObject lets us serialise and deserialise the contents as JSON.
impl GameObject for World {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
          "data": self.data.clone()
        })
    }

    fn from_json(&mut self, value: &serde_json::Value) {
        if let Some(x) = value.get("data").and_then(|x| x.as_str()) {
            self.data = String::from(x);
        } else {
            self.data = World::empty_world()
        };
    }
}

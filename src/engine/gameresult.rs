use std::cmp::Ordering;
use std::collections::HashMap;
pub type GameScore = f32;
pub static NULL_SCORE: GameScore = -999.0;

/// Game status enum.
enum GameStatus {
    Open,
    Win,
    Tie,
    Batch,
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus::Open
    }
}

#[derive(Default)]
/// A single game result.
pub struct GameResult {
    scores: HashMap<char, GameScore>,
    status: GameStatus,
    winner: Option<char>,
}

impl GameResult {
    /// Create a new GameResult.
    pub fn new() -> Self {
        GameResult {
            scores: HashMap::new(),
            status: GameStatus::Open,
            winner: None,
        }
    }

    /// Set the score.
    pub fn set_score(&mut self, identity: char, score: GameScore) {
        self.scores.insert(identity, score);
    }

    /// Get the score.
    pub fn get_score(&self, identity: char) -> Option<&GameScore> {
        self.scores.get(&identity)
    }

    /// Get the winner's identity.
    pub fn derive_winner(&mut self) -> char {
        if self.winner.is_none() {
            self.winner = Some(self.get_winner())
        }

        self.winner.unwrap()
    }

    pub fn get_winner(&self) -> char {
        if self.winner.is_none() {
            assert!(!self.scores.is_empty(), "BUG: No winner - scores set yet!");

            match self
                .scores
                .iter()
                .max_by(|(_, v), (_, v2)| match v.partial_cmp(&v2) {
                    Some(x) => x,
                    None => Ordering::Less,
                }) {
                Some((k, _)) => *k,
                _ => panic!("No scores!"),
            }
        } else {
            self.winner.unwrap()
        }
    }

    /// Set game status to Win.
    pub fn set_win(&mut self) {
        self.status = GameStatus::Win;
    }

    /// Set game status to Tie.
    pub fn set_tie(&mut self) {
        self.status = GameStatus::Tie;
    }

    /// Set game status to Batch.
    pub fn set_batch(&mut self) {
        self.status = GameStatus::Batch;
    }

    /// Return true if game has a winner.
    pub fn is_win(&self) -> bool {
        match self.status {
            GameStatus::Win => true,
            _ => false,
        }
    }

    /// Return true if game is a tie.
    pub fn is_tie(&self) -> bool {
        match self.status {
            GameStatus::Tie => true,
            _ => false,
        }
    }

    /// Return true if game result is from a batch.
    pub fn is_batch(&self) -> bool {
        match self.status {
            GameStatus::Batch => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self.status {
            GameStatus::Open => String::from("Game in progress"),
            GameStatus::Tie => String::from("RESULT: Tie"),
            GameStatus::Win => format!("RESULT: {} wins!", self.get_winner()),
            GameStatus::Batch => format!("BATCH RESULT: {:?}", self.scores),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_result() {
        let mut r = GameResult::new();
        r.set_score('X', 1.0);
        r.set_score('O', 1.1);
        assert_eq!(r.get_winner(), 'O');

        // Once set, the winner does not change.
        r.set_score('A', 2.0);
        assert_eq!(r.get_winner(), 'O');

        assert!(!r.is_win());
        r.set_win();
        assert!(r.is_win());

        r = GameResult::new();
        r.set_score('A', 0.0);
        r.set_score('B', 5.1);
        r.set_score('C', 3.2);
        assert_eq!(r.get_winner(), 'B');
    }
}

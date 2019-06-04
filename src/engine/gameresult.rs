
use std::cmp::Ordering;
use std::collections::HashMap;
type GameScore = f32;

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
    scores: HashMap<String, GameScore>,
    status: GameStatus,
    winner: String,
}

impl GameResult {
    /// Create a new GameResult.
    pub fn new() -> Self {
        GameResult {
            scores: HashMap::new(),
            status: GameStatus::Open,
            winner: String::new(),
        }
    }

    /// Set the score.
    pub fn set_score(&mut self, identity: &str, score: GameScore) {
        self.scores.insert(identity.to_string(), score);
    }

    /// Get the score.
    pub fn get_score(&self, identity: &str) -> Option<&GameScore> {
        self.scores.get(identity)
    }

    /// Get the winner's identity.
    pub fn get_winner(&mut self) -> &str {
        if self.winner.is_empty() {
            assert!(!self.scores.is_empty(), "BUG: No winner - scores set yet!");

            self.winner =
                match self
                    .scores
                    .iter()
                    .max_by(|(_, v), (_, v2)| match v.partial_cmp(&v2) {
                        Some(x) => x,
                        None => Ordering::Less,
                    }) {
                    Some((k, _)) => k.clone(),
                    _ => panic!("No scores!"),
                };
            // let mut highest = -999.0 as f32;
            // for (k, v) in self.scores.iter() {
            //     if *v > highest {
            //         highest = *v;
            //         self.winner = k.clone();
            //     }
            // }

        }

        &self.winner
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
}

#[test]
fn test_result() {
    let mut r = GameResult::new();
    r.set_score("X", 1.0);
    r.set_score("O", 1.1);
    assert_eq!(r.get_winner(), "O");

    // Once set, the winner does not change.
    r.set_score("A", 2.0);
    assert_eq!(r.get_winner(), "O");

    assert!(!r.is_win());
    r.set_win();
    assert!(r.is_win());

    r = GameResult::new();
    r.set_score("A", 0.0);
    r.set_score("B", 5.1);
    r.set_score("C", 3.2);
    assert_eq!(r.get_winner(), "B");
}
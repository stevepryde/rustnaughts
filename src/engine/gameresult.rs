use serde::export::Formatter;
use std::fmt;

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
    scores: [GameScore; 2],
    identities: [char; 2],
    status: GameStatus,
}

impl GameResult {
    /// Create a new GameResult.
    pub fn new(identities: [char; 2]) -> Self {
        GameResult {
            scores: [0.0, 0.0],
            identities,
            status: GameStatus::Open,
        }
    }

    /// Set the score.
    pub fn set_score1(&mut self, score: GameScore) {
        self.scores[0] = score
    }

    /// Set the score.
    pub fn set_score2(&mut self, score: GameScore) {
        self.scores[1] = score
    }

    /// Get the score.
    pub fn get_score1(&self) -> GameScore {
        self.scores[0]
    }

    pub fn get_score2(&self) -> GameScore {
        self.scores[1]
    }

    pub fn get_winner(&self) -> Option<usize> {
        if self.scores[0] > self.scores[1] {
            Some(0)
        } else if self.scores[1] > self.scores[0] {
            Some(1)
        } else {
            None
        }
    }

    pub fn get_winner_identity(&self) -> Option<char> {
        self.get_winner().map(|x| self.identities[x])
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

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.status {
            GameStatus::Open => write!(f, "Game in progress"),
            GameStatus::Tie => write!(f, "RESULT: Tie"),
            GameStatus::Win => match self.get_winner_identity() {
                Some(x) => write!(f, "RESULT: {} wins!", x),
                None => write!(f, "RESULT: Tie"),
            },

            GameStatus::Batch => write!(f, "BATCH RESULT: {:?}", self.scores),
        }
    }
}

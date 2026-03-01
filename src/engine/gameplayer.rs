use crate::engine::gameobject::GameObject;
use crate::engine::gameresult::{GameScore, NULL_SCORE};

#[derive(Debug, Clone, Copy)]
pub struct MutationIntensity {
    pub count: u32,
    pub step_scale: f32,
    pub correlated: bool,
}

#[derive(Default)]
pub struct PlayerData {
    pub name: String,
    pub identity: char,
    pub other_identity: char,
    pub score: GameScore,
    pub should_show_result: bool,
    pub is_magic: bool,
    pub is_genetic: bool,
}

pub trait GamePlayer: GameObject + Send {
    fn get_data(&self) -> &PlayerData;
    fn get_data_mut(&mut self) -> &mut PlayerData;

    fn setup(&mut self, identity: char, other_identity: char) {
        let data = self.get_data_mut();
        data.identity = identity;
        data.other_identity = other_identity;
    }

    fn process(&mut self, _inputs: Vec<f32>, available_moves: &[u32]) -> u32 {
        available_moves[0]
    }
    fn process_magic(&mut self, _inputs: Vec<f32>, available_moves: &[u32]) -> Vec<u32> {
        assert!(
            self.is_magic(),
            "Bot is not magic - shouldn't be doing process_magic!()"
        );
        available_moves.to_vec()
    }

    fn get_name(&self) -> &str {
        self.get_data().name.as_str()
    }

    fn get_identity(&self) -> char {
        self.get_data().identity
    }

    fn get_other_identity(&self) -> char {
        self.get_data().other_identity
    }

    fn get_score(&self) -> GameScore {
        self.get_data().score
    }

    fn set_score(&mut self, score: GameScore) {
        self.get_data_mut().score = score;
    }

    fn mutate(&mut self) {}
    fn mutate_with_intensity(&mut self, intensity: MutationIntensity) {
        for _ in 0..intensity.count {
            self.mutate();
        }
    }
    fn crossover(&mut self, _other: &serde_json::Value) {}
    fn should_show_result(&self) -> bool {
        self.get_data().should_show_result
    }

    fn clear_score(&mut self) {
        self.set_score(NULL_SCORE);
    }

    fn is_magic(&self) -> bool {
        self.get_data().is_magic
    }

    fn is_genetic(&self) -> bool {
        self.get_data().is_genetic
    }

    fn label(&self) -> String {
        format!("{} {}", self.get_name(), self.get_identity())
    }
}

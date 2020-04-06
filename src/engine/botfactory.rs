use crate::bots::genbot3::gbot::GenBot3;
use crate::bots::nbot1::nbot::NBot1;
use crate::bots::omnibot::obot::OmniBot;
use crate::bots::randombot::rbot::RandomBot;

use crate::engine::gamebase::GameInfo;
use crate::engine::gameconfig::BotConfig;
use crate::engine::gameplayer::GamePlayer;

use crate::games::connect4;
use crate::games::naughts;

#[derive(Debug, Clone)]
pub enum NaughtsBot {
    Human,
}

#[derive(Debug, Clone)]
pub enum Connect4Bot {
    Human,
}

#[derive(Debug, Clone)]
pub enum BotType {
    RandomBot,
    GenBot3,
    NBot1,
    OmniBot,
    Naughts(NaughtsBot),
    Connect4(Connect4Bot),
}

impl<S> From<S> for BotType
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        let svalue: String = value.into();
        match svalue.to_ascii_lowercase().as_str() {
            "randombot" => BotType::RandomBot,
            "genbot3" => BotType::GenBot3,
            "nbot1" => BotType::NBot1,
            "omnibot" => BotType::OmniBot,
            x if x.starts_with("naughts") => {
                println!("Matched human");
                match svalue.splitn(2, '.').nth(1) {
                    Some("human") => BotType::Naughts(NaughtsBot::Human),
                    _ => panic!("Unknown bot: {:?}", svalue.splitn(2, '.').nth(2)),
                }
            }
            x if x.starts_with("connect4") => {
                println!("matched connect4");
                match svalue.splitn(2, '.').nth(1) {
                    Some("human") => BotType::Connect4(Connect4Bot::Human),
                    _ => panic!("Unknown bot: {}", svalue),
                }
            }
            _ => panic!("Unknown bot: {}", svalue),
        }
    }
}

pub type DynBot = Box<dyn GamePlayer>;

#[derive(Debug, Clone)]
pub struct BotFactory {
    game_info: GameInfo,
    bot_config: BotConfig,
    recipes: [serde_json::Value; 2],
    genetic_index: Option<usize>,
}

impl BotFactory {
    pub fn new(game_info: GameInfo, bot_config: BotConfig) -> Self {
        let recipe1 = bot_config.recipe.clone();
        let recipe2 = bot_config.recipe.clone();
        Self {
            game_info,
            bot_config,
            recipes: [recipe1, recipe2],
            genetic_index: None,
        }
    }

    pub fn set_recipe(&mut self, index: usize, recipe: serde_json::Value) {
        self.recipes[index] = recipe;
    }

    pub fn set_genetic_index(&mut self, index: usize) {
        self.genetic_index = Some(index);
    }

    pub fn set_genetic_recipe(&mut self, recipe: serde_json::Value) {
        if let Some(x) = self.genetic_index {
            self.set_recipe(x, recipe);
        } else {
            panic!("Attempted to set genetic recipe before index");
        }
    }

    pub fn create_bot(&self, bot_type: &BotType) -> DynBot {
        match bot_type {
            BotType::RandomBot => Box::new(RandomBot::new(&self.game_info)),
            BotType::GenBot3 => Box::new(GenBot3::new(&self.game_info)),
            BotType::NBot1 => Box::new(NBot1::new(&self.game_info)),
            BotType::OmniBot => Box::new(OmniBot::new(&self.game_info)),
            BotType::Naughts(NaughtsBot::Human) => {
                Box::new(naughts::bots::hbot::HumanBot::new(&self.game_info))
            }
            BotType::Connect4(Connect4Bot::Human) => {
                Box::new(connect4::bots::hbot::HumanConnect4Bot::new(&self.game_info))
            }
        }
    }

    fn create_bot_with_recipe(&self, index: usize, bot_type: &BotType) -> DynBot {
        let mut bot = self.create_bot(bot_type);
        if bot.is_genetic() && !self.recipes[index].is_null() {
            bot.from_json(&self.recipes[index]);
        }
        bot
    }

    pub fn create_bot_with_custom_recipe(
        &self,
        bot_type: &BotType,
        recipe: &serde_json::Value,
    ) -> DynBot {
        let mut bot = self.create_bot(bot_type);
        if bot.is_genetic() && !recipe.is_null() {
            bot.from_json(&recipe);
        }
        bot
    }

    pub fn create_genetic_bot_with_custom_recipe(&self, recipe: &serde_json::Value) -> DynBot {
        match self.genetic_index {
            Some(x) => self.create_bot_with_custom_recipe(&self.bot_config.bot_types[x], recipe),
            None => panic!("Can't create genetic bot before setting genetic index!"),
        }
    }

    pub fn create_bots(&self) -> (DynBot, DynBot) {
        (
            self.create_bot_with_recipe(0, &self.bot_config.bot_types[0]),
            self.create_bot_with_recipe(1, &self.bot_config.bot_types[1]),
        )
    }

    pub fn create_bots_with_custom_recipe(
        &self,
        recipes: [&serde_json::Value; 2],
    ) -> (DynBot, DynBot) {
        (
            self.create_bot_with_custom_recipe(&self.bot_config.bot_types[0], recipes[0]),
            self.create_bot_with_custom_recipe(&self.bot_config.bot_types[1], recipes[1]),
        )
    }

    pub fn bot_names(&self) -> (&str, &str) {
        (&self.bot_config.bot_names[0], &self.bot_config.bot_names[1])
    }
}

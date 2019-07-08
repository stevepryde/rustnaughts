use crate::bots::genbot3::gbot::GenBot3;
use crate::bots::nbot1::nbot::NBot1;
use crate::bots::omnibot::obot::OmniBot;
use crate::bots::randombot::rbot::RandomBot;

use crate::engine::gamebase::GameInfo;
use crate::engine::gameconfig::BotConfig;
use crate::engine::gamefactory::create_game;
use crate::engine::gameplayer::GamePlayer;

use crate::games::connect4;
use crate::games::naughts;

pub type BotList = [Box<dyn GamePlayer>];
pub type BotListMut = Vec<Box<dyn GamePlayer>>;

pub fn create_bot(bot_name: &str, game_info: &GameInfo) -> Box<dyn GamePlayer> {
    match bot_name {
        "randombot" => Box::new(RandomBot::new(game_info)),
        "genbot3" => Box::new(GenBot3::new(game_info)),
        "nbot1" => Box::new(NBot1::new(game_info)),
        "omnibot" => Box::new(OmniBot::new(game_info)),
        "naughts.human" => Box::new(naughts::bots::hbot::HumanBot::new(game_info)),
        "connect4.human" => Box::new(connect4::bots::hbot::HumanConnect4Bot::new(game_info)),
        _ => {
            println!("Unknown bot: {}", bot_name);
            panic!("Bailing out");
        }
    }
}

pub fn create_bots(bot_config: &BotConfig) -> BotListMut {
    let game_obj = create_game(bot_config.game.as_str());
    let game_info = game_obj.get_game_info();
    vec![
        create_bot(bot_config.bot_names[0].as_str(), &game_info),
        create_bot(bot_config.bot_names[1].as_str(), &game_info),
    ]
}

pub fn clone_bot(
    bot_name: &str,
    game_info: &GameInfo,
    src_bot_data: &serde_json::Value,
) -> Box<dyn GamePlayer> {
    let mut bot = create_bot(bot_name, game_info);
    bot.from_json(src_bot_data);
    bot
}

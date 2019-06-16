use crate::engine::gameplayer::GamePlayer;

pub type BotList = [Box<dyn GamePlayer>; 2];

use serenity::utils::Colour;
use serenity::model::id::{GuildId, ChannelId};

// A few useful ones
pub const WEEK: usize = 60*60*24*7;
pub const DAY: usize = 60*60*24;
pub const HOUR: usize = 60*60;
pub const MIN: usize = 60;

pub const MESSAGE_CACHE: u32 = 500;

pub const ERROR_LOG: ChannelId = ChannelId(376422808852627457);
pub const COMMAND_LOG: ChannelId = ChannelId(376422940570419200);
pub const GUILD_LOG: ChannelId = ChannelId(406115496833056789);
pub const SUPPORT_SERVER: GuildId = GuildId(373561057639268352);

// Colours
pub enum Colours {
    Main,
    Blue,
    Red,
    Green,
}

impl Colours {
    pub fn val(&self) -> Colour {
        match *self {
            Colours::Main  => Colour::new(0x5da9ff),
            Colours::Blue  => Colour::new(0x7979ff),
            Colours::Red   => Colour::new(0xff4040),
            Colours::Green => Colour::new(0x00ff7f),
        }
    }
}

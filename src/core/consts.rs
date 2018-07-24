use serenity::model::id::{GuildId, ChannelId, RoleId};

// A few useful ones
pub const WEEK: usize = 60*60*24*7;
pub const DAY: usize = 60*60*24;
pub const HOUR: usize = 60*60;
pub const MIN: usize = 60;

pub const MESSAGE_CACHE: usize = 100;

pub const ERROR_LOG: ChannelId = ChannelId(376422808852627457);
pub const COMMAND_LOG: ChannelId = ChannelId(376422940570419200);
pub const GUILD_LOG: ChannelId = ChannelId(406115496833056789);
pub const SUPPORT_SERVER: GuildId = GuildId(373561057639268352);
pub const TRANSCEND: GuildId = GuildId(348660188951216129);
pub const NOW_LIVE: RoleId = RoleId(370395740406546432);

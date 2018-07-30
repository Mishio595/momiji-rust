use serenity::model::id::{GuildId, ChannelId, RoleId};
use db::Database;

lazy_static!{
    pub static ref DB: Database = Database::connect();
}

pub const WEEK: usize = 60*60*24*7;
pub const DAY:  usize = 60*60*24;
pub const HOUR: usize = 60*60;
pub const MIN:  usize = 60;

pub const SLICE_SIZE: usize = 65535;
pub const USER_SLICE_SIZE: usize = 65535/5;
pub const MESSAGE_CACHE: usize = 100;

pub const ERROR_LOG: ChannelId      = ChannelId(376422808852627457);
pub const COMMAND_LOG: ChannelId    = ChannelId(376422940570419200);
pub const GUILD_LOG: ChannelId      = ChannelId(406115496833056789);
pub const SUPPORT_SERVER: GuildId   = GuildId(373561057639268352);
pub const TRANSCEND: GuildId        = GuildId(348660188951216129);
pub const NOW_LIVE: RoleId          = RoleId(370395740406546432);

pub const DB_ROLES_FAIL: &str       = "Failed to select roles";
pub const DB_USER_FAIL: &str        = "Failed to select user entry";
pub const DB_USER_ENTRY_FAIL: &str  = "Failed to create user entry";
pub const DB_GUILD_FAIL: &str       = "Failed to select guild entry";
pub const DB_GUILD_ENTRY_FAIL: &str = "Failed to create guild entry";
pub const DB_GUILD_DEL_FAIL: &str   = "Failed to delete guild entry";
pub const API_FAIL: &str            = "Failed to get API";
pub const API_GET_FAIL: &str        = "Failed while making a GET request";
pub const CACHE_GUILD_FAIL: &str    = "Failed to get guild lock from CACHE";
pub const CACHE_CHANNEL_FAIL: &str  = "Failed to get channel lock from CACHE";
pub const GUILDID_FAIL: &str        = "Failed to get GuildId";
pub const USER_FAIL: &str           = "Failed to get user";
pub const MEMBER_FAIL: &str         = "Failed to get member";
pub const TC_FAIL: &str             = "Failed to get TimerClient";

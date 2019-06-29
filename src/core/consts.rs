use crate::db::Database;
use serenity::model::id::{GuildId, ChannelId, RoleId};

lazy_static!{
    pub static ref DB: Database = Database::connect();
    pub static ref LOG_TYPES: Vec<&'static str> = vec![
        "member_ban",
        "member_join",
        "member_kick",
        "member_leave",
        "member_unban",
        "message_delete",
        "message_edit",
        "nickname_change",
        "role_change",
        "username_change"];
}

pub const WEEK: usize = 60*60*24*7;
pub const DAY:  usize = 60*60*24;
pub const HOUR: usize = 60*60;
pub const MIN:  usize = 60;

pub const MESSAGE_CACHE: usize = 100;
pub const SLICE_SIZE: usize = 65535;
pub const USER_SLICE_SIZE: usize = 65535/5;

pub const COMMAND_LOG: ChannelId    = ChannelId(376422940570419200);
pub const ERROR_LOG: ChannelId      = ChannelId(376422808852627457);
pub const GUILD_LOG: ChannelId      = ChannelId(406115496833056789);
pub const NOW_LIVE: RoleId          = RoleId(370395740406546432);
pub const SUPPORT_SERVER: GuildId   = GuildId(373561057639268352);
pub const TRANSCEND: GuildId        = GuildId(348660188951216129);

pub const SUPPORT_SERV_INVITE: &str = "https://discord.gg/YYdpsNc";
pub const BOT_INVITE: &str          = "https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025";
pub const GITLAB_LINK: &str         = "https://gitlab.com/Mishio595/momiji-rust";
pub const PATREON_LINK: &str        = "https://www.patreon.com/momijibot";

pub const API_FAIL: &str            = "Failed to get API";
pub const CACHE_CHANNEL_FAIL: &str  = "Failed to get channel lock from CACHE";
pub const CACHE_GUILD_FAIL: &str    = "Failed to get guild lock from CACHE";
pub const DB_GUILD_FAIL: &str       = "Failed to select Guild";
pub const DB_GUILD_DEL_FAIL: &str   = "Failed to delete Guild";
pub const DB_GUILD_ENTRY_FAIL: &str = "Failed to insert Guild";
pub const DB_USER_ENTRY_FAIL: &str  = "Failed to insert User";
pub const GUILD_FAIL: &str          = "Failed to get Guild";
pub const GUILDID_FAIL: &str        = "Failed to get GuildId";
pub const MEMBER_FAIL: &str         = "Failed to get member";
pub const TC_FAIL: &str             = "Failed to get TimerClient";
pub const USER_FAIL: &str           = "Failed to get user";

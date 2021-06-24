use twilight_model::id::{GuildId, ChannelId};

lazy_static::lazy_static!{
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
pub const SUPPORT_SERVER: GuildId   = GuildId(373561057639268352);

pub const SUPPORT_SERV_INVITE: &str = "https://discord.gg/YYdpsNc";
pub const BOT_INVITE: &str          = "https://discordapp.com/oauth2/authorize/?permissions=268823760&scope=bot&client_id=345316276098433025";
pub const GITLAB_LINK: &str         = "https://gitlab.com/Mishio595/momiji-rust";

pub mod colors {
    pub const MAIN: u32 = 0x5da9ff;
    pub const BLUE: u32 = 0x6969ff;
    pub const RED: u32 = 0xff4040;
    pub const GREEN: u32 = 0x00ff7f;
}
pub mod misc;
pub mod nsfw;
pub mod roles;
pub mod tags;

use self::misc::*;
use self::nsfw::*;
use self::roles::*;
use self::tags::*;
use serenity::framework::standard::CreateGroup;

pub fn init_misc() -> CreateGroup{
    CreateGroup::default()
        .help_available(true)
        .cmd("anime", Anime)
        .cmd("botinfo", BotInfo)
        .cmd("cat", Cat)
        .cmd("dog", Dog)
        .cmd("joke", DadJoke)
        .cmd("manga", Manga)
        .cmd("now", Now)
        .cmd("ping", Ping)
        .cmd("prefix", Prefix)
        .cmd("remind", Reminder)
        .cmd("roleinfo", RoleInfo)
        .cmd("roll", Roll)
        .cmd("serverinfo", ServerInfo)
        .cmd("tags", TagList)
        .cmd("urban", Urban)
        .cmd("userinfo", UserInfo)
        .cmd("weather", Weather)
}

pub fn init_nsfw() -> CreateGroup {
    CreateGroup::default()
        .help_available(true)
        .check(|_,message,_,_| {
            if let Ok(channel) = message.channel_id.to_channel() {
                if channel.is_nsfw() {
                    true
                } else {
                    check_error!(message.channel_id.say("Command only available in NSFW channels."));
                    false
                }
            } else {
                check_error!(message.channel_id.say("Failed to get the channel info. I can't tell if this channel is NSFW."));
                false
        }})
        .cmd("e621", Furry)
}

pub fn init_roles() -> CreateGroup {
    CreateGroup::default()
        .help_available(true)
        .guild_only(true)
        .cmd("role", AddSelfRole)
        .cmd("derole", RemoveSelfRole)
        .cmd("roles", ListSelfRoles)
}

pub fn init_tags() -> CreateGroup {
    CreateGroup::default()
        .help_available(true)
        .guild_only(true)
        .prefix("tag")
        .default_cmd(TagSingle)
        .cmd("show", TagSingle)
        .cmd("add", TagAdd)
        .cmd("del", TagRemove)
        .cmd("edit", TagEdit)
        .cmd("list", TagList)
}
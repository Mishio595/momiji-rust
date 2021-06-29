pub mod misc;
// pub mod nsfw;
pub mod roles;
pub mod tags;

use self::misc::*;
// use self::nsfw::*;
use self::roles::*;
use self::tags::*;
use momiji::framework::command::{CommandOrAlias::*, ModuleBuilder};
use std::sync::Arc;

pub fn init_misc(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(true)
        // .add_command("anime", Command(Arc::new(Anime)))
        .add_command("botinfo", Command(Arc::new(BotInfo)))
        .add_command("bi", Alias("botinfo".to_string()))
        // .add_command("cat", Command(Arc::new(Cat)))
        // .add_command("dog", Command(Arc::new(Dog)))
        // .add_command("joke", Command(Arc::new(DadJoke)))
        // .add_command("manga", Command(Arc::new(Manga)))
        // .add_command("now", Command(Arc::new(Now)))
        .add_command("ping", Command(Arc::new(Ping)))
        .add_command("prefix", Command(Arc::new(Prefix)))
        .add_command("remind", Command(Arc::new(Reminder)))
        // .add_command("roleinfo", Command(Arc::new(RoleInfo)))
        // .add_command("roll", Command(Arc::new(Roll)))
        // .add_command("serverinfo", Command(Arc::new(ServerInfo)))
        .add_command("tags", Command(Arc::new(TagList)))
        // .add_command("urban", Command(Arc::new(Urban)))
        // .add_command("uid", Command(Arc::new(UserId)))
        // .add_command("userinfo", Command(Arc::new(UserInfo)))
        // .add_command("weather", Command(Arc::new(Weather)))
        // .add_command("stats", Command(Arc::new(Stats)))
}

// pub fn init_nsfw() -> CreateGroup {
//     CreateGroup::default()
//         .help_available(true)
//         .check(|_,message,_,_| {
//             if let Ok(channel) = message.channel_id.to_channel() {
//                 if channel.is_nsfw() {
//                     true
//                 } else {
//                     check_error!(message.channel_id.say("Command only available in NSFW channels."));
//                     false
//                 }
//             } else {
//                 check_error!(message.channel_id.say("Failed to get the channel info. I can't tell if this channel is NSFW."));
//                 false
//         }})
//         .cmd("e621", Furry)
// }

pub fn init_roles(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(true)
        .guild_only(true)
        .add_command("asr", Command(Arc::new(AddSelfRole)))
        .add_command("role", Alias("asr".to_string()))
        .add_command("rsr", Command(Arc::new(RemoveSelfRole)))
        .add_command("derole", Alias("rsr".to_string()))
        .add_command("lsr", Command(Arc::new(ListSelfRoles)))
        .add_command("roles", Alias("lsr".to_string()))
}

pub fn init_tags(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(true)
        .guild_only(true)
        .prefix("tag")
        .default_command(Command(Arc::new(TagSingle)))
        .add_command("show", Command(Arc::new(TagSingle)))
        .add_command("add", Command(Arc::new(TagAdd)))
        .add_command("del", Command(Arc::new(TagRemove)))
        .add_command("edit", Command(Arc::new(TagEdit)))
        .add_command("list", Command(Arc::new(TagList)))
}

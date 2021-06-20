// pub mod hackbans;
// pub mod info;
// pub mod kickbans;
// pub mod mute;
// pub mod notes;
pub mod roles;
// pub mod watchlist;

// use self::hackbans::*;
// use self::info::*;
// use self::kickbans::*;
// use self::mute::*;
// use self::notes::*;
use self::roles::*;
// use self::watchlist::*;
use momiji::framework::command::{CommandOrAlias::*, ModuleBuilder};
use std::sync::Arc;

// pub fn init_info(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .guild_only(true)
//         .help_available(true)
//         .add_command("modinfo", Command(Arc::new(ModInfo)))
// }

// pub fn init_kickbans(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .guild_only(true)
//         .help_available(true)
//         .add_command("ban", Command(Arc::new(BanUser)))
//         .add_command("kick", Command(Arc::new(KickUser)))
// }

// pub fn init_mute(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .guild_only(true)
//         .help_available(true)
//         .add_command("mute", Command(Arc::new(Mute)))
//         .add_command("unmute", Command(Arc::new(Unmute)))
// }

// pub fn init_notes(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .prefix("note")
//         .guild_only(true)
//         .help_available(true)
//         .add_command("add", Command(Arc::new(NoteAdd)))
//         .add_command("del", Command(Arc::new(NoteRemove)))
//         .add_command("list", Command(Arc::new(NoteList)))
// }

pub fn init_roles(module: ModuleBuilder) -> ModuleBuilder {
    module
        .guild_only(true)
        .help_available(true)
        .add_command("register", Command(Arc::new(Register)))
        .add_command("reg", Alias("register".to_string()))
        .add_command("ar", Command(Arc::new(AddRole)))
        .add_command("addrole", Alias("ar".to_string()))
        .add_command("rr", Command(Arc::new(RemoveRole)))
        .add_command("removerole", Alias("rr".to_string()))
        // .add_command("rolecolour", Command(Arc::new(RoleColour)))
}

// pub fn init_watchlist(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .prefixes(vec!["watchlist", "wl"])
//         .guild_only(true)
//         .help_available(true)
//         .default_cmd(WatchlistList)
//         .add_command("add", Command(Arc::new(WatchlistAdd)))
//         .add_command("del", Command(Arc::new(WatchlistRemove)))
//         .add_command("list", Command(Arc::new(WatchlistList)))
// }
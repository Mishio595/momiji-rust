pub mod hackbans;
pub mod info;
pub mod mute;
pub mod notes;
pub mod roles;
pub mod watchlist;

use self::hackbans::*;
use self::info::*;
use self::mute::*;
use self::notes::*;
use self::roles::*;
use self::watchlist::*;
use serenity::framework::standard::CreateGroup;

pub fn init_hackbans() -> CreateGroup {
    CreateGroup::default()
        .prefixes(vec!["hackban", "hb"])
        .guild_only(true)
        .help_available(true)
        .cmd("add", HackbanAdd)
        .cmd("remove", HackbanRemove)
        .cmd("list", HackbanList)
}

pub fn init_info() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .cmd("modinfo", ModInfo)
}

pub fn init_mute() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .cmd("mute", Mute)
        .cmd("unmute", Unmute)
}

pub fn init_notes() -> CreateGroup {
    CreateGroup::default()
        .prefix("note")
        .guild_only(true)
        .help_available(true)
        .cmd("add", NoteAdd)
        .cmd("del", NoteRemove)
        .cmd("list", NoteList)
}
pub fn init_roles() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .cmd("register", Register)
        .cmd("addrole", AddRole)
        .cmd("removerole", RemoveRole)
        .cmd("rolecolour", RoleColour)
}

pub fn init_watchlist() -> CreateGroup {
    CreateGroup::default()
        .prefixes(vec!["watchlist", "wl"])
        .guild_only(true)
        .help_available(true)
        .default_cmd(WatchlistList)
        .cmd("add", WatchlistAdd)
        .cmd("del", WatchlistRemove)
        .cmd("list", WatchlistList)
}
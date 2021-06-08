pub mod log;
pub mod premium;
pub mod db;
pub mod cache;

use self::log::*;
use self::premium::*;
use self::db::*;
use self::cache::*;
use momiji::framework::command::{CommandOrAlias::*, ModuleBuilder};
use std::sync::Arc;

pub fn init(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(false)
        .add_command("log", Command(Arc::new(Log)))
        .add_command("op", Command(Arc::new(Premium)))
        .add_command("cache", Command(Arc::new(CacheStats)))
}

pub fn init_db(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(false)
        .prefix("db")
        .add_command("new_guild", Command(Arc::new(NewGuild)))
}
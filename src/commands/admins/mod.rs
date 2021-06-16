pub mod config;
// pub mod ignore;
pub mod management;
pub mod register_control;
pub mod roles;
// pub mod tests;

use self::config::*;
// use self::ignore::*;
use self::management::*;
use self::register_control::*;
use self::roles::*;
// use self::tests::*;
use momiji::framework::command::{CommandOrAlias::*, ModuleBuilder};
use std::sync::Arc;

pub fn init_config(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(true)
        .guild_only(true)
        .prefix("config")
        .default_command(Command(Arc::new(ConfigList)))
        .add_command("list", Command(Arc::new(ConfigList)))
        .add_command("raw", Command(Arc::new(ConfigRaw)))
        .add_command("prefix", Command(Arc::new(ConfigPrefix)))
        .add_command("autorole", Command(Arc::new(ConfigAutorole)))
        .add_command("admin", Command(Arc::new(ConfigAdmin)))
        .add_command("mod", Command(Arc::new(ConfigMod)))
        .add_command("audit", Command(Arc::new(ConfigAudit)))
        .add_command("modlog", Command(Arc::new(ConfigModlog)))
        .add_command("welcome", Command(Arc::new(ConfigWelcome)))
        .add_command("introduction", Command(Arc::new(ConfigIntroduction)))
        .add_command("cmd", Command(Arc::new(ConfigCommands)))
        .add_command("log", Command(Arc::new(ConfigLogs)))
        .add_command("register_member", Command(Arc::new(RegisterMember)))
        .add_command("register_cooldown", Command(Arc::new(RegisterCooldown)))
        .add_command("register_duration", Command(Arc::new(RegisterDuration)))
        .add_command("register_roles", Command(Arc::new(RegisterRestrictions)))
        .add_command("reg_member", Alias("register_member".to_string()))
        .add_command("reg_cooldown", Alias("register_cooldown".to_string()))
        .add_command("reg_duration", Alias("register_duration".to_string()))
        .add_command("reg_roles", Alias("register_roles".to_string()))
}

// pub fn init_ignore(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .guild_only(true)
//         .help_available(true)
//         .prefix("ignore")
//         .default_command(IgnoreList)
//         .add_command("add", Command(Arc::new(IgnoreAdd)))
//         .add_command("remove", Command(Arc::new(IgnoreRemove)))
//         .add_command("list", Command(Arc::new(IgnoreList)))
//         .add_command("level", Command(Arc::new(IgnoreLevel)))
// }

pub fn init_management(module: ModuleBuilder) -> ModuleBuilder {
    module
        .guild_only(true)
        .help_available(true)
        // .add_command("setup", Command(Arc::new(SetupMute)))
        .add_command("prune", Command(Arc::new(Prune)))
        .add_command("purge", Alias("prune".to_string()))
        // .add_command("cleanup", Command(Arc::new(Cleanup)))
}

pub fn init_roles(module: ModuleBuilder) -> ModuleBuilder {
    module
        .help_available(true)
        .guild_only(true)
        .add_command("csr", Command(Arc::new(CreateSelfRole)))
        .add_command("dsr", Command(Arc::new(DeleteSelfRole)))
        .add_command("esr", Command(Arc::new(EditSelfRole)))
}

// pub fn init_tests(module: ModuleBuilder) -> ModuleBuilder {
//     module
//         .guild_only(true)
//         .help_available(true)
//         .prefix("test")
//         .add_command("welcome", Command(Arc::new(TestWelcome)))
//         .add_command("intro", Command(Arc::new(TestIntro)))
// }

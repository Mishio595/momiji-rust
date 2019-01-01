pub mod config;
pub mod ignore;
pub mod management;
pub mod premium;
pub mod roles;
pub mod tests;

use self::config::*;
use self::ignore::*;
use self::management::*;
use self::premium::*;
use self::roles::*;
use self::tests::*;
use serenity::framework::standard::CreateGroup;

pub fn init_config() -> CreateGroup {
    CreateGroup::default()
        .help_available(true)
        .guild_only(true)
        .prefixes(vec!["config", "conf"])
        .default_cmd(ConfigList)
        .cmd("list", ConfigList)
        .cmd("raw", ConfigRaw)
        .cmd("prefix", ConfigPrefix)
        .cmd("autorole", ConfigAutorole)
        .cmd("admin", ConfigAdmin)
        .cmd("mod", ConfigMod)
        .cmd("audit", ConfigAudit)
        .cmd("modlog", ConfigModlog)
        .cmd("welcome", ConfigWelcome)
        .cmd("introduction", ConfigIntroduction)
        .cmd("cmd", ConfigCommands)
        .cmd("log", ConfigLogs)
}

pub fn init_ignore() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .prefix("ignore")
        .default_cmd(IgnoreList)
        .cmd("add", IgnoreAdd)
        .cmd("remove", IgnoreRemove)
        .cmd("list", IgnoreList)
}

pub fn init_management() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .cmd("setup", SetupMute)
        .cmd("prune", Prune)
        .cmd("cleanup", Cleanup)
}

pub fn init_premium() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .prefixes(vec!["p", "premium", "prem"])
        .cmd("register_member", PRegisterMember)
        .cmd("register_cooldown", PRegisterCooldown)
        .cmd("register_duration", PRegisterDuration)
        .cmd("register_roles", PRegisterRestrictions)
}

pub fn init_roles() -> CreateGroup {
    CreateGroup::default()
        .help_available(true)
        .guild_only(true)
        .cmd("csr", CreateSelfRole)
        .cmd("dsr", DeleteSelfRole)
        .cmd("esr", EditSelfRole)
}

pub fn init_tests() -> CreateGroup {
    CreateGroup::default()
        .guild_only(true)
        .help_available(true)
        .prefix("test")
        .cmd("welcome", TestWelcome)
}

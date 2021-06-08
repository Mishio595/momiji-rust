use crate::commands;
use momiji::framework::{Config, Framework};
use momiji::framework::parser::{StandardParser as Parser};
use std::collections::HashSet;
use twilight_model::id::{GuildId, UserId};
use twilight_gateway::Cluster;

pub struct StandardFramework(Framework<Parser>);

impl StandardFramework {
    pub fn new(owners: HashSet<UserId>, cluster: Cluster) -> Framework<Parser> {
        let parser = Parser;
        
        let config = Config::builder()
            .prefix("m!")
            .dynamic_prefix(|message, db| {
                if message.guild_id.is_none() {
                    return Some(String::new());
                } else {
                    let gid = message.guild_id.unwrap_or(GuildId(0));
                    if let Ok(settings) = db.get_guild(gid.0 as i64) {
                        return Some(settings.prefix);
                    }
                }

                None
            })
            .owners(owners)
            .build();

        Framework::builder()
            .config(config)
            .parser(parser)
            .cluster(cluster)
            .add_module("Config", commands::admins::init_config)
            .add_module("Management", commands::admins::init_management)
            .add_module("Miscellaneous", commands::general::init_misc)
            .add_module("Self Roles", commands::general::init_roles)
            .add_module("Self Role Management", commands::admins::init_roles)
            .add_module("Tags", commands::general::init_tags)
            .add_module("Database Controls", commands::owner::init_db)
            .add_module("Owner Tools", commands::owner::init)
            .add_module("Mod Role Tools", commands::mods::init_roles)
            .build()
    }
}
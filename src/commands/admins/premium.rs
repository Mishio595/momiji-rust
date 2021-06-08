use crate::core::consts::*;
use crate::core::consts::DB as db;
use crate::core::utils::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct PRegisterMember;
impl Command for PRegisterMember {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set the member role used by register. This role is automatically either after cooldown, if cooldown is set, or right away.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            example: Some("member".to_string()),
            aliases: vec!["reg_m", "reg_member"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = db.get_premium(guild_id.0 as i64)?;
            if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
                settings.register_member_role = Some(role_id.0 as i64);
                db.update_premium(guild_id.0 as i64, settings)?;
                message.channel_id.say(format!("Set member role to {}", role.name))?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct PRegisterCooldown;
impl Command for PRegisterCooldown {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set the cooldown role used by register. This is applied automatically before member and removed after cooldown_duration".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            example: Some("cooldown".to_string()),
            aliases: vec!["reg_c", "reg_cooldown"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = db.get_premium(guild_id.0 as i64)?;
            if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
                settings.register_cooldown_role = Some(role_id.0 as i64);
                db.update_premium(guild_id.0 as i64, settings)?;
                message.channel_id.say(format!("Set cooldown role to {}", role.name))?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct PRegisterDuration;
impl Command for PRegisterDuration {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set the duration cooldown is applied for. Default value is 24 hours.".to_string()),
            usage: Some("<time_resolvable>".to_string()),
            example: Some("24h".to_string()),
            aliases: vec!["reg_dur", "reg_duration"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = db.get_premium(guild_id.0 as i64)?;
            if let Ok(dur) = args.full().parse::<String>() {
                let dur = hrtime_to_seconds(dur);
                settings.register_cooldown_duration = Some(dur as i32);
                db.update_premium(guild_id.0 as i64, settings)?;
                message.channel_id.say(format!("Set duration of cooldown to {}", seconds_to_hrtime(dur as usize)))?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct PRegisterRestrictions;
impl Command for PRegisterRestrictions {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Manage the roles people on cooldown cannot self-assign. These are also ignored in register command usage. Valid options: `add`, `remove`, `set`".to_string()),
            usage: Some("<option> [values]".to_string()),
            example: Some("set selfies, nsfw".to_string()),
            aliases: vec!["reg_roles", "reg_restrict"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let op = args.single::<String>().unwrap_or(String::new());
            let mut sec = "";
            let mut val = String::new();
            let mut settings = db.get_premium(guild_id.0 as i64)?;
            match op.as_str() {
                "add" => {
                    if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
                        settings.cooldown_restricted_roles.push(role_id.0 as i64);
                        sec = "Added";
                        val = role.name;
                    }
                },
                "remove" => {
                    if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
                        settings.cooldown_restricted_roles.push(role_id.0 as i64);
                        sec = "Removed";
                        val = role.name;
                    }
                },
                "set" => {
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut roles = Vec::new();
                    let mut role_names = Vec::new();
                    for role in list {
                        if let Some((role_id, role)) = parse_role(role, guild_id) {
                            roles.push(role_id.0 as i64);
                            role_names.push(role.name);
                        }
                    }
                    settings.cooldown_restricted_roles = roles;
                    sec = "Set to";
                    val = role_names.join(", ");
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`, `set`. For more information see `help p reg_roles`")?; },
            }
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Successfully modified restricted roles. {} {}", sec, val))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
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

pub struct ConfigRaw;
impl Command for ConfigRaw {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Lists current configuration as raw output.".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            message.channel_id.say(format!("{:?}", guild_data))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigList;
impl Command for ConfigList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Lists current configuration.".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .colour(*colours::MAIN)
                    .description(format!("{}", guild_data))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigPrefix;
impl Command for ConfigPrefix {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set a new prefix.".to_string()),
            usage: Some("<prefix>".to_string()),
            example: Some("!!".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let pre = args.single::<String>()?;
            guild_data.prefix = pre;
            match db.update_guild(guild_id.0 as i64, guild_data) {
                Ok(guild) => {
                    message.channel_id.say(format!("Set prefix to {}", guild.prefix))?;
                },
                Err(_) => {
                    message.channel_id.say("Failed to change prefix")?;
                },
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAutorole;
impl Command for ConfigAutorole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change autorole settings. A role must be provided for add or remove.".to_string()),
            usage: Some("<add|remove|enable|disable> <role_resolvable|_>".to_string()),
            example: Some("add member".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.autoroles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                "enable" => {
                    guild_data.autorole = true;
                },
                "disable" => {
                    guild_data.autorole = false;
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`, `enable`, `disable`. For more information see `help config autorole`")?;
                    return Ok(())
                },
            }
            let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Autorole Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { guild.autorole.to_string() } else { val } ,
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAdmin;
impl Command for ConfigAdmin {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add or remove roles from the bot's admin list.".to_string()),
            usage: Some("<add|remove> <role_resolvable>".to_string()),
            example: Some("add admin".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.admin_roles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config admin`")?;
                    return Ok(())
                },
            }
            db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Admin Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigMod;
impl Command for ConfigMod {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add or remove roles from the bot's admin list.".to_string()),
            usage: Some("<add|remove> <role_resolvable>".to_string()),
            example: Some("add staff".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.mod_roles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that role.")?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config mod`")?;
                    return Ok(())
                },
            }
            db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Mod Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAudit;
impl Command for ConfigAudit {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change audit log settings. A channel must be provided for channel.".to_string()),
            usage: Some("<enable|disable|channel> <channel_resolvable>".to_string()),
            example: Some("channel #audit-logs".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.audit = true;
                },
                "disable" => {
                    guild_data.audit = false;
                },
                "channel" => {
                    match parse_channel(val.to_string(), guild_id) {
                        Some((channel_id, channel)) => {
                            guild_data.audit_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name, channel_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that channel.")?;
                            return Ok(())
                        },
                    }
                },
                "threshold" => {
                    match val.parse::<i16>() {
                        Ok(th) => {
                            guild_data.audit_threshold = th;
                            val = th.to_string();
                        },
                        Err(_) => { message.channel_id.say("Please input a number as the threshold")?; }
                    }
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `threshold`. For more information see `help config audit`")?;
                    return Ok(())
                },
            }
            let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Audit Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.audit) } else { val },
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigModlog;
impl Command for ConfigModlog {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change moderation log settings. A channel must be provided for channel.".to_string()),
            usage: Some("<enable|disable|channel> <channel_resolvable>".to_string()),
            example: Some("channel #mod-logs".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.modlog = true;
                },
                "disable" => {
                    guild_data.modlog = false;
                },
                "channel" => {
                    match parse_channel(val.to_string(), guild_id) {
                        Some((channel_id, channel)) => {
                            guild_data.modlog_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name, channel_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that channel.")?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`. For more information see `help config modlog`")?;
                    return Ok(())
                },
            }
            let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Modlog Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { guild.modlog.to_string() } else { val },
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigWelcome;
impl Command for ConfigWelcome {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change welcome message settings.\nOption is one of enable, disable, channel, message, type and the respective values should be none, none, channel_resolvable, desired message.\nType designates if the message is plain or embed. Anything other than embed will result in plain.".to_string()),
            usage: Some("<option> <value>".to_string()),
            example: Some("message Welcome to {guild}, {user}!".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.welcome = true;
                },
                "disable" => {
                    guild_data.welcome = false;
                },
                "channel" => {
                    match parse_channel(val.to_string(), guild_id) {
                        Some((channel_id, channel)) => {
                            guild_data.welcome_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name, channel_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that channel.")?;
                            return Ok(())
                        },
                    }
                },
                "message" => {
                    guild_data.welcome_message = val.to_string();
                },
                "type" => {
                    guild_data.welcome_type = val.to_string();
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config welcome`")?;
                    return Ok(())
                },
            }
            let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Welcome Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { guild.welcome.to_string() } else { val },
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigIntroduction;
impl Command for ConfigIntroduction {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change introduction message settings. This is exactly like welcome: `help config welcome` for more info. This is a premium only feature related to the Register command.".to_string()),
            usage: Some("<option> <value>".to_string()),
            example: Some("message Hey there {user}, mind introducting yourself?".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.introduction = true;
                },
                "disable" => {
                    guild_data.introduction = false;
                },
                "channel" => {
                    match parse_channel(val.to_string(), guild_id) {
                        Some((channel_id, channel)) => {
                            guild_data.introduction_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name, channel_id.0);
                        },
                        None => {
                            message.channel_id.say("I couldn't find that channel.")?;
                            return Ok(())
                        },
                    }
                },
                "message" => {
                    guild_data.introduction_message = val.to_string();
                },
                "type" => {
                    guild_data.introduction_type = val.to_string();
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config introduction`")?;
                    return Ok(())
                },
            }
            let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Introduction Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { guild.introduction.to_string() } else { val },
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigCommands;
impl Command for ConfigCommands {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change which commands are disabled. A command name must be provided.".to_string()),
            usage: Some("<enable|disable> <command_name>".to_string()),
            example: Some("disable e621".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.commands.retain(|e| *e != val);
                },
                "disable" => {
                    if !val.starts_with("conf") {
                        guild_data.commands.push(val.clone());
                    } else {
                        message.channel_id.say("Config commands cannot be disabled.")?;
                        return Ok(());
                    }
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config command`")?;
                    return Ok(())
                },
            }
            db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Command Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigLogs;
impl Command for ConfigLogs {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change which log messages are disabled. A log type must be provided.".to_string()),
            usage: Some("<enable|disable|types> [type]".to_string()),
            example: Some("disable message_edit".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.logging.retain(|e| *e != val);
                },
                "disable" => {
                    if LOG_TYPES.contains(&val.as_str()) {
                        guild_data.logging.push(val.clone());
                    } else {
                        message.channel_id.say("Invalid log type. See `config log types` for valid types.")?;
                        return Ok(());
                    }
                },
                "types" => {
                    message.channel_id.say(LOG_TYPES.iter()
                        .map(|e| format!("`{}`", e))
                        .collect::<Vec<String>>()
                        .join(", "))?;
                    return Ok(());
                },
                _ => {
                    message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config log`")?;
                    return Ok(())
                },
            }
            db.update_guild(guild_id.0 as i64, guild_data)?;
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Log Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

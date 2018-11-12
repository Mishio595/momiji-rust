use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use serenity::builder::GetMessages;
use serenity::CACHE;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::{
    Message,
    PermissionOverwrite,
    PermissionOverwriteType
};
use serenity::model::Permissions;
use serenity::model::id::*;
use serenity::prelude::*;
use std::sync::Arc;

pub struct ConfigRaw;
impl Command for ConfigRaw {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Lists current configuration as raw output.".to_string()),
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
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                "enable" => {
                    guild_data.autorole = true;
                },
                "disable" => {
                    guild_data.autorole = false;
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`, `enable`, `disable`. For more information see `help config autorole`")?; },
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
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config admin`")?; },
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
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id) {
                        Some((role_id, role)) => {
                            guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => { message.channel_id.say("I couldn't find that role.")?; }
                    }
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config mod`")?; },
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
                        None => { message.channel_id.say("I couldn't find that channel.")?; }
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
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `threshold`. For more information see `help config audit`")?; },
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
                        None => { message.channel_id.say("I couldn't find that channel.")?; }
                    }
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`. For more information see `help config modlog`")?; },
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
                        None => { message.channel_id.say("I couldn't find that channel.")?; }
                    }
                },
                "message" => {
                    guild_data.welcome_message = val.to_string();
                },
                "type" => {
                    guild_data.welcome_type = val.to_string();
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config welcome`")?; },
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
                        None => { message.channel_id.say("I couldn't find that channel.")?; }
                    }
                },
                "message" => {
                    guild_data.introduction_message = val.to_string();
                },
                "type" => {
                    guild_data.introduction_type = val.to_string();
                },
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config introduction`")?; },
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
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config command`")?; },
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
                _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config log`")?; },
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

pub struct IgnoreAdd;
impl Command for IgnoreAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Tell the bot to ignore a channel.".to_string()),
            usage: Some("<channel_resolvable>".to_string()),
            example: Some("#general".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            if let Some((channel_id, channel)) = parse_channel(args.full().to_string(), guild_id) {
                if !guild_data.ignored_channels.contains(&(channel_id.0 as i64)) {
                    guild_data.ignored_channels.push(channel_id.0 as i64);
                    db.update_guild(guild_id.0 as i64, guild_data)?;
                    message.channel_id.say(format!(
                        "I will now ignore messages in {}",
                        channel.name
                    ))?;
                } else {
                    message.channel_id.say("That channel is already being ignored.")?;
                }
            } else {
                message.channel_id.say("I couldn't find that channel.")?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct IgnoreRemove;
impl Command for IgnoreRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Tell the bot to stop ignoring a channel.".to_string()),
            usage: Some("<channel_resolvable>".to_string()),
            example: Some("#general".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            if let Some((channel_id, channel)) = parse_channel(args.full().to_string(), guild_id) {
                if guild_data.ignored_channels.contains(&(channel_id.0 as i64)) {
                    guild_data.ignored_channels.retain(|e| *e != channel_id.0 as i64);
                    db.update_guild(guild_id.0 as i64, guild_data)?;
                    message.channel_id.say(format!(
                        "I will no longer ignore messages in {}",
                        channel.name
                    ))?;
                } else {
                    message.channel_id.say("That channel isn't being ignored.")?;
                }
            } else {
                message.channel_id.say("I couldn't find that channel.")?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct IgnoreList;
impl Command for IgnoreList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List all ignored channels.".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            if !guild_data.ignored_channels.is_empty() {
                let channel_out = guild_data.ignored_channels.clone()
                    .iter()
                    .map(|c| format!("<#{}>", c))
                    .collect::<Vec<String>>()
                    .join("\n");
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .title("Ignored Channels")
                        .description(channel_out)
                        .colour(*colours::MAIN)
                ))?;
            } else {
                message.channel_id.say("I'm not ignoring any channels.")?;
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct CreateSelfRole;
impl Command for CreateSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Create a self role from a discord role. Also optionally takes a category and/or aliases.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV]".to_string()),
            example: Some("NSFW /c Opt-in /a porn, lewd".to_string()),
            aliases: vec!["createselfrole".to_string()],
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args.full().to_string());
            let backup = String::new();
            let rest = switches.get("rest").unwrap_or(&backup);
            if let Some((role_id, _)) = parse_role(rest.clone(), guild_id) {
                let category = match switches.get("c") {
                    Some(s) => Some( s.clone()),
                    None => None,
                };
                let aliases = match switches.get("a") {
                    Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
                    None => None,
                };
                let data = db.new_role(role_id.0 as i64, guild_id.0 as i64, category, aliases)?;
                message.channel_id.say(format!("Successfully added role {} to category {} {}",
                    data.id,
                    data.category,
                    if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct DeleteSelfRole;
impl Command for DeleteSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Delete a self role.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            example: Some("NSFW".to_string()),
            aliases: vec!["deleteselfrole".to_string()],
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some((role_id, _)) = parse_role(args.full().to_string(), guild_id) {
                let data = db.del_role(role_id.0 as i64, guild_id.0 as i64)?;
                message.channel_id.say(format!("Successfully deleted role {}", data))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct EditSelfRole;
impl Command for EditSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Edit a self role. Optionally takes a category and/or aliases. This operation is lazy and won't change anything you don't specify. Replace switch tells the bot to override aliases instead of append.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV] [/replace]".to_string()),
            example: Some("NSFW /c Opt-in /a porn, lewd /replace".to_string()),
            aliases: vec!["editselfrole".to_string()],
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args.full().to_string());
            let backup = String::new();
            let rest = switches.get("rest").unwrap_or(&backup);
            if let Some((role_id, _)) = parse_role(rest.clone(), guild_id) {
                let category = match switches.get("c") {
                    Some(s) => Some(s.clone()),
                    None => None,
                };
                let aliases = match switches.get("a") {
                    Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
                    None => None,
                };
                let mut role = db.get_role(role_id.0 as i64, guild_id.0 as i64)?;
                if let Some(s) = category { role.category = s; }
                if let Some(mut a) = aliases {
                    match switches.get("replace") {
                        Some(_) => { role.aliases = a; },
                        None => { role.aliases.append(&mut a); },
                    }
                }
                let data = db.update_role(role_id.0 as i64, guild_id.0 as i64, role)?;
                message.channel_id.say(format!("Successfully update role {} in category {} {}",
                    data.id,
                    data.category,
                    if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct PRegisterMember;
impl Command for PRegisterMember {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set the member role used by register. This role is automatically either after cooldown, if cooldown is set, or right away.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            example: Some("member".to_string()),
            aliases: vec!["reg_m", "reg_member"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
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
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
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
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
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
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
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

pub struct Prune;
impl Command for Prune {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Bulk delete messages. Filter is one of bot, attachment, !pin, mention, or a user_resolvable.\n`bot` will prune only messages from bots.\n`attachment` will prune only messages with attachments.\n`!pin` will prune all but pinned messages.\n`mention` will prune only messages that mention a user or everyone.\nMentioning a user will prune only that user's messages.".to_string()),
            usage: Some("<count> [filter]".to_string()),
            example: Some("20 bot".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            let mut count = args.single::<usize>().unwrap_or(0);
            let fsel = args.single::<String>().unwrap_or(String::new());
            let mut filter = get_filter(fsel, guild_id);
            let mut deletions = message.channel_id.messages(|_| re_retriever(100))?;
            let mut next_deletions;
            let mut num_del = 0;
            message.delete()?;
            if count<=1000 {
                while count>0 {
                    deletions.retain(|m| filter(m));
                    let mut len = deletions.len();
                    if len<=0 { break; }
                    if len>count {
                        deletions.truncate(count);
                        len = count;
                    }
                    count -= len;
                    if count>0 {
                        next_deletions = message.channel_id.messages(|_| be_retriever(deletions[0].id, 100)).ok();
                    } else {
                        next_deletions = None;
                    }
                    match message.channel_id.delete_messages(deletions) {
                        Ok(_) => {
                            num_del += len;
                            deletions = match next_deletions {
                                Some(s) => s,
                                None => { break; },
                            }
                        },
                        Err(why) => {
                            error!("{:?}", why);
                            break;
                        },
                    }
                }
                if num_del > 0 {
                    if guild_data.modlog {
                        let channel = {
                            let cache = CACHE.read();
                            cache.guild_channel(message.channel_id)
                        };
                        ChannelId(guild_data.modlog_channel as u64).send_message(|m| m
                            .embed(|e| e
                                .title("Messages Pruned")
                                .description(format!("**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {}",
                                    num_del,
                                    message.author.mention(),
                                    message.author.tag(),
                                    match channel {
                                        Some(ch) => {
                                            let ch = ch.read();
                                            format!(
                                                "{} ({})",
                                                ch.mention(),
                                                ch.name)
                                        },
                                        None => message.channel_id.0.to_string(),
                                    }))
                                .timestamp(now!())
                                .colour(*colours::RED)
                        ))?;
                    } else {
                        message.channel_id.say(format!("Pruned {} message!", num_del))?;
                    }
                } else {
                    message.channel_id.say("I wasn't able to delete any messages.")?;
                }
            } else {
                message.channel_id.say("Please enter a number no greater than 1000.")?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TestWelcome;
impl Command for TestWelcome {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Generates a welcome message to test your current setup.".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some(member) = message.member() {
                let guild_data = db.get_guild(guild_id.0 as i64)?;
                if guild_data.welcome {
                    let channel = ChannelId(guild_data.welcome_channel as u64);
                    if guild_data.welcome_type.as_str() == "embed" {
                        send_welcome_embed(guild_data.welcome_message, &member, channel)?;
                    } else {
                        channel.say(parse_welcome_items(guild_data.welcome_message, &member))?;
                    }
                }
            } else { failed!(MEMBER_FAIL); }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct SetupMute;
impl Command for SetupMute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Sets up mute for the server. This command requires the Manage Channels and Manage Roles permissions. It creates the Muted role if it doesn't exist, then iterates through every channel and category to disable Send Messages, Speak, and Add Reactions.".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild = {
                let cache = CACHE.read();
                cache.guild(guild_id)
            };
            if let Some(guild_lock) = guild {
                let guild = guild_lock.read().clone();
                let mut guild_data = db.get_guild(guild_id.0 as i64)?;
                let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                    Some(role) => role.clone(),
                    None => {
                        message.channel_id.say("Role `Muted` created")?;
                        guild.create_role(|r| r.name("Muted"))?
                    },
                };
                let allow = Permissions::empty();
                let deny = Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS | Permissions::SPEAK;
                let overwrite = PermissionOverwrite {
                    allow,
                    deny,
                    kind: PermissionOverwriteType::Role(mute_role.id),
                };
                for channel in guild.channels.values() {
                    let mut channel = channel.read();
                    channel.create_permission(&overwrite)?;
                }
                guild_data.mute_setup = true;
                db.update_guild(guild.id.0 as i64, guild_data)?;
                message.channel_id.say(format!("Setup permissions for {} channels.", guild.channels.len()))?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

// Helper functions for commands::prune
fn re_retriever(limit: u64) -> GetMessages {
    GetMessages::default()
        .limit(limit)
}

fn be_retriever(id: MessageId, limit: u64) -> GetMessages {
    GetMessages::default()
        .before(id)
        .limit(limit)
}

fn get_filter(input: String, guild_id: GuildId) -> Box<FnMut(&Message) -> bool> {
    match input.as_str() {
        "bot" => Box::new(|m| m.author.bot),
        "mention" => Box::new(|m| !m.mentions.is_empty() && m.mention_everyone),
        "attachment" => Box::new(|m| !m.attachments.is_empty()),
        "!pin" => Box::new(|m| !m.pinned),
        _ => {
            match parse_user(input.to_string(), guild_id) {
                Some((user_id, _)) => {
                    Box::new(move |m| m.author.id == user_id)
                },
                None => {
                    Box::new(|_| true)
                },
            }
        },
    }
}

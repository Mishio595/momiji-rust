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

pub struct IgnoreAdd;
impl Command for IgnoreAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Tell the bot to ignore a channel.".to_string()),
            usage: Some("<channel_resolvable>".to_string()),
            example: Some("#general".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
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
            required_permissions: Permissions::MANAGE_GUILD,
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
            required_permissions: Permissions::MANAGE_GUILD,
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
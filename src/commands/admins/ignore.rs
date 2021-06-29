use crate::core::colours;
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

pub struct IgnoreAdd;
impl Command for IgnoreAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Tell the bot to ignore a channel.".to_string()),
            usage: Some("<channel_resolvable>".to_string()),
            example: Some("#general".to_string()),
            min_args: Some(1),
            max_args: Some(1),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            min_args: Some(1),
            max_args: Some(1),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            max_args: Some(0),
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
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

pub struct IgnoreLevel;
impl Command for IgnoreLevel {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Set the rank threshold required to bypass ignored channels. 4 = bot owner, 3 = guild owner, 2 = admin, 1 = mod, 0 = everyone.".to_string()),
            usage: Some("<0..4>".to_string()),
            example: Some("2".to_string()),
            min_args: Some(1),
            max_args: Some(1),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            match args.single::<i16>() {
                Ok(level) => {
                    guild_data.ignore_level = level;
                    db.update_guild(guild_id.0 as i64, guild_data)?;
                    message.channel_id.say(format!("Successfully set ignore level to {}", level))?;
                },
                Err(_) => {
                    message.channel_id.say("Please enter an integer between 0 and 4.")?;
                },
            }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

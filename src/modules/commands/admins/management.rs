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
use serenity::model::id::{
    ChannelId,
    GuildId,
    MessageId
};
use serenity::model::Permissions;
use serenity::prelude::{
    Context,
    Mentionable
};
use std::sync::Arc;

pub struct Prune;
impl Command for Prune {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Bulk delete messages. Filter is one of bot, attachment, !pin, mention, or a user_resolvable.\n`bot` will prune only messages from bots.\n`attachment` will prune only messages with attachments.\n`!pin` will prune all but pinned messages.\n`mention` will prune only messages that mention a user or everyone.\nMentioning a user will prune only that user's messages.".to_string()),
            usage: Some("<count> [filter]".to_string()),
            example: Some("20 bot".to_string()),
            min_args: Some(1),
            required_permissions: Permissions::MANAGE_GUILD,
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

pub struct Cleanup;
impl Command for Cleanup {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Cleans up all commands and responses for Momiji sent in the past 10 minutes in the current channel.".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            max_args: Some(0),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            let user = CACHE.read().user.clone();
            let mut deletions = message.channel_id.messages(|_| re_retriever(100))?;
            let mut next_deletions;
            let mut num_del = 0;
            message.delete()?;
            loop {
                deletions.retain(|m|
                    (Utc::now() - m.timestamp.with_timezone(&Utc)).num_seconds() <= 10*MIN as i64
                    && (m.author.id == user.id
                    || m.content.starts_with(&guild_data.prefix)
                    || m.content.starts_with(&user.mention()))
                );
                let mut len = deletions.len();
                if len<=0 { break; }
                next_deletions = message.channel_id.messages(|_| be_retriever(deletions[0].id, 100)).ok();
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
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct SetupMute;
impl Command for SetupMute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Sets up mute for the server. This command requires the Manage Channels and Manage Roles permissions. It creates the Muted role if it doesn't exist, then iterates through every channel and category to disable Send Messages, Speak, and Add Reactions. Add `bypass` as an arg to skip permission setting.".to_string()),
            max_args: Some(1),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let guild = {
                let cache = CACHE.read();
                cache.guild(guild_id)
            };
            if let Some(guild_lock) = guild {
                let guild = guild_lock.read().clone();
                let mut guild_data = db.get_guild(guild_id.0 as i64)?;
                let bypass = args.single::<String>().unwrap_or("".to_string());
                let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                    Some(role) => role.clone(),
                    None => {
                        message.channel_id.say("Role `Muted` created")?;
                        guild.create_role(|r| r.name("Muted"))?
                    },
                };
                if bypass != "bypass" {
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

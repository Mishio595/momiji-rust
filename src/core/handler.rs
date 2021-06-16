use chrono::Utc;
use crate::Context;
use crate::core::consts::*;
use crate::core::utils::*;
use crate::core::utils::user_tag;
use crate::framework::Framework;
use futures::stream::StreamExt;
use levenshtein::levenshtein;
use tracing::{event, Level};
use twilight_mention::Mention;
use std::error::Error;
use std::sync::Arc;
use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder, ImageSource};
use twilight_gateway::{Event, EventType};
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

use super::utils::build_welcome_embed;
use super::utils::parse_welcome_items;

pub struct EventHandler {
    framework: Arc<Framework>,
    ctx: Context,
}

impl EventHandler {
    pub fn new(framework: Framework, ctx: Context) -> Self {
        Self {
            framework: Arc::new(framework),
            ctx
        }
    }
 
    pub async fn start(&self) {
        let mut events = self.ctx.cluster.events();
        while let Some((shard_id, event)) = events.next().await {
            // Bypass cache update for message delete to enable logging
            if event.kind() != EventType::MessageDelete {
                self.ctx.cache.update(&event);
            }
    
            tokio::spawn(handle_event(shard_id, event, self.ctx.clone(), self.framework.clone()));
        }
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    ctx: Context,
    framework: Arc<Framework>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (cache, db, http) = {
        let c = ctx.clone();
        (c.cache, c.db, c.http)
    };
    match event {
        Event::MessageCreate(message) => {
            if let Err(e) = (*framework).handle_command(message.0, ctx.clone()).await {
                event!(Level::DEBUG, "{}", e);
            }
        }
        Event::MessageDelete(message) => {
            if let Some(guild_id) = message.guild_id {
                let channel_name = cache.guild_channel(message.channel_id).and_then(|c| Some(c.name().to_string())).unwrap_or("unknown".to_string());
                let guild_data = db.get_guild(guild_id.0 as i64)?;
                if guild_data.logging.contains(&String::from("message_delete")) { return Ok(()); }
                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                if guild_data.audit && audit_channel.0 > 0 {
                    if let Some(cached_message) = cache.message(message.channel_id, message.id) {
                        if let Some(author) = cache.user(cached_message.author) {
                            if author.bot { return Ok(()); }
                            let embed = EmbedBuilder::new()
                                .title("Message Deleted")
                                .color(colors::RED)
                                .footer(EmbedFooterBuilder::new(format!("ID: {}", message.id.0)))
                                .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - <#{}>\n**Content:**\n{}",
                                    user_tag(author.clone()),
                                    author.id.0,
                                    author.mention(),
                                    channel_name,
                                    message.channel_id.0,
                                    message.channel_id.0,
                                    cached_message.content))
                                .timestamp(chrono::Utc::now().to_rfc3339())
                                .build()?;
                            
                            http.create_message(audit_channel).embed(embed)?.await?;
                        }
                    } else {
                        let channel_name = cache.guild_channel(message.channel_id).and_then(|c| Some(c.name().to_string())).unwrap_or("unknown".to_string());
                        let embed = EmbedBuilder::new()
                            .title("Uncached Message Deleted")
                            .color(colors::RED)
                            .footer(EmbedFooterBuilder::new(format!("ID: {}", message.id.0)))
                            .description(format!("**Channel:** {} ({}) - <#{}>",
                                channel_name,
                                message.channel_id.0,
                                message.channel_id.0,))
                            .timestamp(chrono::Utc::now().to_rfc3339())
                            .build()?;

                        http.create_message(audit_channel).embed(embed)?.await?;
                    }
                }
            }
        }
        //TODO establish method for getting old message content
        // Event::MessageUpdate(message) => {
        //     if message.author.clone().map(|u| u.bot).unwrap_or(false) { return Ok(()) }
        //     if let None = message.edited_timestamp { return Ok(()) }
        //     if let Some(old_message) = cache.message(message.channel_id, message.id) {
        //         if let Some(guild_id) = message.guild_id {
        //             let channel_name = cache.guild_channel(message.channel_id)
        //                 .map(|c| c.name().to_string())
        //                 .unwrap_or("unknown".to_string());
        //             match db.get_guild(guild_id.0 as i64) {
        //                 Ok(guild_data) => {
        //                     if guild_data.logging.contains(&String::from("message_edit")) { return Ok(()) }
        //                     let audit_channel = ChannelId(guild_data.audit_channel as u64);
        //                     let new_content = message.content.unwrap_or("".to_string());
        //                     let distance = levenshtein(old_message.content.as_str(), new_content.as_str());
        //                     if guild_data.audit && audit_channel.0 > 0 && distance >= guild_data.audit_threshold as usize {
        //                         let (author_tag, author_mention) = if let Some(user) = message.author {
        //                             let tag = format!("{}#{}", user.name, user.discriminator);
        //                             (tag, user.mention().to_string())
        //                         } else if let Some(user) = cache.user(old_message.author) {
        //                             let tag = format!("{}#{}", user.name, user.discriminator);
        //                             (tag, user.mention().to_string())
        //                         } else {
        //                             ("Unknown".to_string(), "Unknown".to_string())
        //                         };
        //                         let embed = EmbedBuilder::new()
        //                             .title("Message Edited")
        //                             .color(colors::MAIN)
        //                             .timestamp(Utc::now().to_rfc3339())
        //                             .footer(EmbedFooterBuilder::new(format!("ID: {}", message.id.0)))
        //                             .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - <#{}>\n**Old Content:**\n{}\n**New Content:**\n{}",
        //                                 author_tag,
        //                                 old_message.author.0,
        //                                 author_mention,
        //                                 channel_name,
        //                                 message.channel_id.0,
        //                                 message.channel_id.0,
        //                                 old_message.content,
        //                                 new_content
        //                                 ))
        //                             .build()?;
        //                             http.create_message(audit_channel)
        //                                 .embed(embed)?
        //                                 .await?;
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //     }
        // }
        Event::ShardConnected(_) => {
            event!(Level::DEBUG, "Connected on shard {}", shard_id);
        }
        // TODO join/leave log. Need solution to restart spam, maybe compare to ready
        Event::GuildCreate(guild) => {
            event!(Level::DEBUG, "Guild received: {} ({})", guild.name, guild.id);
            match db.new_guild(guild.id.0 as i64) {
                Err(why) => { event!(Level::DEBUG, "{}: {}", DB_GUILD_ENTRY_FAIL, why); }
                _ => {}
            }
        }
        Event::GuildDelete(guild) => {
            match db.del_guild(guild.id.0 as i64) {
                Ok(_) => { //TODO no point in leave logs until we have join logs
                }
                Err(why) => { event!(Level::DEBUG, "Failed to delete {}: {}", guild.id, why) }
            }
        }
        Event::MemberAdd(member) => {
            // TODO maybe hackbans still. Think about it
            match db.get_guild(member.guild_id.0 as i64) {
                Ok(guild_data) => {
                    if guild_data.logging.contains(&String::from("member_join")) { return Ok(()) }
                    let user_update = crate::db::models::UserUpdate {
                        id: member.user.id.0 as i64,
                        guild_id: member.guild_id.0 as i64,
                        username: member.user.name.clone()
                    };
                    match db.upsert_user(user_update) {
                        Ok(mut user_data) => {
                            if guild_data.audit && guild_data.audit_channel > 0 {
                                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                                let embed = EmbedBuilder::new()
                                    .title("Member Joined")
                                    .color(colors::GREEN)
                                    .thumbnail(ImageSource::url(user_avatar_url(&member.user))?)
                                    .timestamp(Utc::now().to_rfc3339())
                                    .description(format!("<@{}>\n{}\n{}", member.user.id.0, format!("{}#{}", member.user.name, member.user.discriminator), member.user.id.0))
                                    .build()?;
                                http.create_message(audit_channel).embed(embed)?.await?;
                            }
                            if guild_data.welcome && guild_data.welcome_channel > 0 {
                                let channel_id = ChannelId(guild_data.welcome_channel as u64);
                                if guild_data.welcome_type.as_str() == "embed" {
                                    let embed = build_welcome_embed(guild_data.welcome_message, &member, &cache)?.build()?;
                                    http.create_message(channel_id).embed(embed)?.await?;
                                } else {
                                    let content = parse_welcome_items(guild_data.welcome_message, &member, &cache);
                                    http.create_message(channel_id).content(content)?.await?;
                                }
                            }
                            if guild_data.autorole && !guild_data.autoroles.is_empty() {
                                for id in guild_data.autoroles.iter() {
                                    http.add_guild_member_role(member.guild_id, member.user.id, RoleId(*id as u64)).await?;
                                }
                            }
                            user_data.username = format!("{}#{}", member.user.name, member.user.discriminator);
                            user_data.nickname = member.nick.clone().unwrap_or(member.user.name.clone());
                            user_data.roles = member.roles.iter().map(|r| r.0 as i64).collect();
                            db.update_user(member.user.id.0 as i64, member.guild_id.0 as i64, user_data);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Event::MemberRemove(member) => {
            match db.get_guild(member.guild_id.0 as i64) {
                Ok(guild_data) => {
                    db.del_user(member.user.id.0 as i64, member.guild_id.0 as i64);
                    if guild_data.logging.contains(&String::from("member_leave")) { return Ok(()) }
                    if guild_data.audit && guild_data.audit_channel > 0 {
                        let audit_channel = ChannelId(guild_data.audit_channel as u64);
                        let embed = EmbedBuilder::new()
                            .title("Member Left")
                            .color(colors::RED)
                            // .thumbnail()
                            .timestamp(Utc::now().to_rfc3339())
                            .description(format!("<@{}>\n{}#{}\n{}", member.user.id.0, member.user.name, member.user.discriminator , member.user.id))
                            .build()?;
                        http.create_message(audit_channel).embed(embed)?.await?;
                    }
                    //TODO kick log
                }
                _ => {}
            }
        }
        Event::MemberUpdate(member) => {
            //TODO Figure out how to get old member data
        }
        Event::BanAdd(ban) => {
            use twilight_model::guild::audit_log::AuditLogEvent;
            let audit_request = ctx.http.audit_log(ban.guild_id)
                .action_type(AuditLogEvent::MemberBanAdd)
                .limit(1)?;
            if let Some(audit_log) = audit_request.await? {
                if let Some(audit) = audit_log.audit_log_entries.first() {
                    match ctx.db.get_guild(ban.guild_id.0 as i64) {
                        Ok(guild_data) => {
                            if guild_data.logging.contains(&String::from("member_ban")) { return Ok(()) }
                            let target_id = audit.target_id.clone()
                                .map(|ref s| UserId(s.parse::<u64>().unwrap_or(0)))
                                .unwrap();
                            if guild_data.modlog && guild_data.modlog_channel > 0 && target_id == ban.user.id {
                                let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                                let moderator = match audit.user_id {
                                    Some(user_id) => { match ctx.http.user(user_id).await? {
                                        Some(user) => { format!("{}#{} ({})", user.name, user.discriminator, user.id.0) }
                                        None => { "unknown".to_string() }
                                    }}
                                    None => { "unknown".to_string() }
                                };
                                let embed = EmbedBuilder::new()
                                    .title("Member Banned")
                                    .color(colors::RED)
                                    .thumbnail(ImageSource::url(user_avatar_url(&ban.user))?)
                                    .timestamp(chrono::Utc::now().to_rfc3339())
                                    .description(format!("**Member:** {}#{} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                                        ban.user.name,
                                        ban.user.discriminator,
                                        ban.user.id.0,
                                        ban.user.mention(),
                                        moderator,
                                        audit.reason.clone().unwrap_or("None".to_string())
                                    )).build()?;
                                ctx.http.create_message(modlog_channel).embed(embed)?.await?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Event::BanRemove(ban) => {
            use twilight_model::guild::audit_log::AuditLogEvent;
            let audit_request = ctx.http.audit_log(ban.guild_id)
                .action_type(AuditLogEvent::MemberBanRemove)
                .limit(1)?;
            if let Some(audit_log) = audit_request.await? {
                if let Some(audit) = audit_log.audit_log_entries.first() {
                    match ctx.db.get_guild(ban.guild_id.0 as i64) {
                        Ok(guild_data) => {
                            if guild_data.logging.contains(&String::from("member_unban")) { return Ok(()) }
                            let target_id = audit.target_id.clone()
                                .map(|ref s| UserId(s.parse::<u64>().unwrap_or(0)))
                                .unwrap();
                            if guild_data.modlog && guild_data.modlog_channel > 0 && target_id == ban.user.id {
                                let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                                let moderator = match audit.user_id {
                                    Some(user_id) => { match ctx.http.user(user_id).await? {
                                        Some(user) => { format!("{}#{} ({})", user.name, user.discriminator, user.id.0) }
                                        None => { "unknown".to_string() }
                                    }}
                                    None => { "unknown".to_string() }
                                };
                                let embed = EmbedBuilder::new()
                                    .title("Member Unbanned")
                                    .color(colors::GREEN)
                                    .thumbnail(ImageSource::url(user_avatar_url(&ban.user))?)
                                    .timestamp(chrono::Utc::now().to_rfc3339())
                                    .description(format!("**Member:** {}#{} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                                        ban.user.name,
                                        ban.user.discriminator,
                                        ban.user.id.0,
                                        ban.user.mention(),
                                        moderator,
                                        audit.reason.clone().unwrap_or("None".to_string())
                                    )).build()?;
                                ctx.http.create_message(modlog_channel).embed(embed)?.await?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Event::Ready(ready) => {
            event!(Level::DEBUG, "Connected with session_id {}", ready.session_id);
        }
        Event::Resumed => {
            event!(Level::DEBUG, "Session resumed");
        }
        _ => { event!(Level::DEBUG, "Unhandled event: {:?}", event.kind()); }
    }

    Ok(())
}
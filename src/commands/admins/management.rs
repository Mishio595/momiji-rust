use chrono::Utc;
use momiji::Context;
use momiji::core::consts::*;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::{event, Level};
use twilight_cache_inmemory::InMemoryCache;
use twilight_embed_builder::EmbedBuilder;
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;
use twilight_model::id::{ChannelId, GuildId, MessageId};
use twilight_mention::Mention;
use std::sync::Arc;
use std::error::Error;

pub struct Prune;
#[async_trait]
impl Command for Prune {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Bulk delete messages. Filter is one of bot, attachment, !pin, mention, or a user_resolvable.\n`bot` will prune only messages from bots.\n`attachment` will prune only messages with attachments.\n`!pin` will prune all but pinned messages.\n`mention` will prune only messages that mention a user or everyone.\nMentioning a user will prune only that user's messages.".to_string()),
            usage: Some("<count> [filter]".to_string()),
            examples: vec!["20 bot".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        ctx.http.delete_message(message.channel_id, message.id).await?;
        if let Some(guild_id) = message.guild_id {
            let count = args.single::<usize>().unwrap_or(0);
            if count<=1000 {
                let guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
                let fsel = args.single::<String>().unwrap_or(String::new());
                let mut filter = get_filter(fsel, guild_id, &ctx.cache);
                let mut deletions = ctx.http.channel_messages(message.channel_id)
                    .limit(u64::min(100, count as u64))?
                    .await?;
                let mut next_deletions;
                let mut deleted_messages = Vec::new();
                while deleted_messages.len() < count {
                    next_deletions = ctx.http.channel_messages(message.channel_id)
                        .before(deletions[deletions.len() - 1].id)
                        .limit(u64::min(100, (count - deleted_messages.len()) as u64))?
                        .await?;
                    deletions.retain(|m| filter(m) && is_deletable(m));
                    match (deletions.len(), count - deleted_messages.len()) {
                        (n,_) if n <= 0 => { break; },
                        (n,c) if n > c => {
                            deletions.truncate(c);
                        },
                        _ => (),
                    }
                    let message_ids: Vec<MessageId> = deletions.iter().map(|m| m.id).collect();
                    match ctx.http.delete_messages(message.channel_id, message_ids).await {
                        Ok(_) => {
                            deleted_messages.append(&mut deletions);
                            deletions = if next_deletions.is_empty() {
                                next_deletions
                            } else { break; };
                        },
                        Err(why) => {
                            event!(Level::ERROR, "Prune Error: {:?}", why);
                            break;
                        },
                    }
                }
                if deleted_messages.len() > 0 {
                    if guild_data.modlog {
                        let channel_name = ctx.cache.guild_channel(message.channel_id)
                            .map(|c| c.name().to_string())
                            .unwrap_or(message.channel_id.to_string());
                        let embed = EmbedBuilder::new()
                            .title("Messages Pruned")
                            .description(format!(
                                "**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {}",
                                deleted_messages.len(),
                                message.author.mention(),
                                format!("{}#{}", message.author.name, message.author.discriminator),
                                channel_name))
                            .timestamp(Utc::now().to_rfc3339())
                            .color(colors::RED)
                            .build()?;
                        ctx.http.create_message(ChannelId(guild_data.modlog_channel as u64))
                            .embed(embed)?
                            .await?;
                    } else {
                        ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Pruned {} message!", deleted_messages.len()))?.await?;
                    }
                    if guild_data.audit {
                        deleted_messages.reverse();
                        let prune_log = deleted_messages.iter()
                            .map(|m| format!(
                                "[{}] {} ({}): {}",
                                m.timestamp,
                                format!("{}#{}", m.author.name, m.author.discriminator),
                                m.author.id.0,
                                m.content
                                ))
                            .collect::<Vec<String>>()
                            .join("\r\n");
                        ctx.http.create_message(ChannelId(guild_data.audit_channel as u64))
                            .file(format!("prune-log-{}.txt", Utc::now().format("%FT%T")).as_str(), prune_log.as_bytes())
                            .await?;
                    }
                } else {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I wasn't able to delete any messages.")?.await?;
                }
            } else {
                ctx.http.create_message(message.channel_id).reply(message.id).content("Please enter a number no greater than 1000.")?.await?;
            }
        }
        Ok(())
    }
}

// pub struct Cleanup;
// #[async_trait]
// impl Command for Cleanup {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Cleans up all commands and responses for Momiji sent in the past 10 minutes in the current channel.".to_string()),
//             required_permissions: Permissions::MANAGE_GUILD,
//             ..default
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             let guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
//             let user = ctx.cache.read().user.clone();
//             let mut deletions = message.channel_id.messages(|_| re_retriever(100))?;
//             let mut next_deletions;
//             let mut num_del = 0;
//             message.delete()?;
//             loop {
//                 deletions.retain(|m|
//                     (Utc::now() - m.timestamp.with_timezone(&Utc)).num_seconds() <= 10*MIN as i64
//                     && (m.author.id == user.id
//                     || m.content.starts_with(&guild_data.prefix)
//                     || m.content.starts_with(&user.mention()))
//                 );
//                 let mut len = deletions.len();
//                 if len<=0 { break; }
//                 next_deletions = message.channel_id.messages(|_| be_retriever(deletions[0].id, 100)).ok();
//                 match message.channel_id.delete_messages(deletions) {
//                     Ok(_) => {
//                         num_del += len;
//                         deletions = match next_deletions {
//                             Some(s) => s,
//                             None => { break; },
//                         }
//                     },
//                     Err(why) => {
//                         error!("{:?}", why);
//                         break;
//                     },
//                 }
//             }
//             if num_del > 0 {
//                 if guild_data.modlog {
//                     let channel = {
//                         let ctx.cache = ctx.cache.read();
//                         ctx.cache.guild_channel(message.channel_id)
//                     };
//                     ChannelId(guild_data.modlog_channel as u64).send_message(|m| m
//                         .embed(|e| e
//                             .title("Messages Pruned")
//                             .description(format!("**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {}",
//                                 num_del,
//                                 message.author.mention(),
//                                 message.author.tag(),
//                                 match channel {
//                                     Some(ch) => {
//                                         let ch = ch.read();
//                                         format!(
//                                             "{} ({})",
//                                             ch.mention(),
//                                             ch.name)
//                                     },
//                                     None => message.channel_id.0.to_string(),
//                                 }))
//                             .timestamp(now!())
//                             .colour(*colours::RED)
//                     ))?;
//                 } else {
//                     message.channel_id.say(format!("Pruned {} message!", num_del))?;
//                 }
//             } else {
//                 message.channel_id.say("I wasn't able to delete any messages.")?;
//             }
//         } else { failed!(GUILDID_FAIL); }
//         Ok(())
//     }
// }

// pub struct SetupMute;
// #[async_trait]
// impl Command for SetupMute {
//     fn options(&self) -> Arc<Options> {
//         let options = Options {
//             description: Some("Sets up mute for the server. This command requires the Manage Channels and Manage Roles permissions. It creates the Muted role if it doesn't exist, then iterates through every channel and category to disable Send Messages, Speak, and Add Reactions. Add `bypass` as an arg to skip permission setting.".to_string()),
//             required_permissions: Permissions::MANAGE_GUILD,
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             let guild = {
//                 let ctx.cache = ctx.cache.read();
//                 ctx.cache.guild(guild_id)
//             };
//             if let Some(guild_lock) = guild {
//                 let guild = guild_lock.read().clone();
//                 let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
//                 let bypass = args.single::<String>().unwrap_or("".to_string());
//                 let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
//                     Some(role) => role.clone(),
//                     None => {
//                         message.channel_id.say("Role `Muted` created")?;
//                         guild.create_role(|r| r.name("Muted"))?
//                     },
//                 };
//                 if bypass != "bypass" {
//                     let allow = Permissions::empty();
//                     let deny = Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS | Permissions::SPEAK;
//                     let overwrite = PermissionOverwrite {
//                         allow,
//                         deny,
//                         kind: PermissionOverwriteType::Role(mute_role.id),
//                     };
//                     for channel in guild.channels.values() {
//                         let mut channel = channel.read();
//                         channel.create_permission(&overwrite)?;
//                     }
//                 }
//                 guild_data.mute_setup = true;
//                 ctx.db.update_guild(guild.id.0 as i64, guild_data)?;
//                 message.channel_id.say(format!("Setup permissions for {} channels.", guild.channels.len()))?;
//             }
//         } else { failed!(GUILDID_FAIL); }
//         Ok(())
//     }
// }

fn is_deletable(message: &Message) -> bool {
    let now = Utc::now().timestamp();
    let then = match chrono::DateTime::parse_from_rfc3339(message.timestamp.as_str()) {
        Ok(ts) => ts.timestamp(),
        _ => { return false; }
    };
    now - then < (WEEK as i64)*2
}

fn get_filter(input: String, guild_id: GuildId, cache: &InMemoryCache) -> Box<dyn FnMut(&Message) -> bool + Send + Sync> {
    match input.as_str() {
        "bot" => Box::new(|m| m.author.bot),
        "mention" => Box::new(|m| !m.mentions.is_empty() || m.mention_everyone),
        "attachment" => Box::new(|m| !m.attachments.is_empty()),
        "!pin" => Box::new(|m| !m.pinned),
        _ => {
            match parse_user(input.to_string(), guild_id, cache) {
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

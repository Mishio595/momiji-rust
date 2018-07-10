use serenity::prelude::*;
use serenity::CACHE;
use serenity::model::{
    id::*,
    guild::*,
    user::*,
    event::*,
    gateway::{Ready, Game}
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use chrono::Utc;
use core::model::*;
use core::consts::*;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        CACHE.write().settings_mut().max_messages(MESSAGE_CACHE);
        info!("Logged in as {}", ready.user.name);
    }

    // TODO create/update users on launch
    fn cached(&self, ctx: Context, guilds: Vec<GuildId>) {
        let mut data = ctx.data.lock();
        let cache = CACHE.read();
        let db = data.get::<DB>().unwrap().lock();
        for guild_id in guilds.iter() {
            db.new_guild(guild_id.0 as i64).ok();
        }

        let api = data.get::<ApiClient>().unwrap();
        let guild_count = guilds.len();
        api.stats_update(cache.user.id.0, guild_count);
        ctx.set_game(Game::listening(&format!("{} guilds | m!help", guild_count)));
    }

    fn message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        let cache = CACHE.read();
        if let Some(channel) = channel_id.get().unwrap().guild() {
            let channel = channel.read();
            let guild_id = channel.guild_id;
            let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
            let audit_channel = ChannelId(guild_data.audit_channel as u64);
            if let Some(messages) = cache.messages.get(&channel_id) {
                if let Some(message) = messages.get(&message_id) {
                    audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Message Deleted")
                            .colour(Colours::Red.val())
                            .footer(|f| f.text(format!("ID: {}", message_id.0)))
                            .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - {}\n**Content:**\n{}",
                                message.author.tag(),
                                message.author.id.0,
                                message.author.mention(),
                                channel.name,
                                channel.id.0,
                                channel.mention(),
                                message.content_safe()))
                            .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                    )).expect("Failed to send message");
                } else {
                    audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Uncached Message Deleted")
                            .colour(Colours::Red.val())
                            .footer(|f| f.text(format!("ID: {}", message_id.0)))
                            .description(format!("**Channel:** {} ({}) - {}",
                                channel.name,
                                channel.id.0,
                                channel.mention()))
                            .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                    )).expect("Failed to send message");
                };
            };
        };
    }

    fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, ids: Vec<MessageId>) {

    }

    fn message_update(&self, ctx: Context, event: MessageUpdateEvent) {

    }

    // Username changes and Now Live! role
    // TODO now live role
    fn presence_update(&self, ctx: Context, event: PresenceUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            let data = ctx.data.lock();
            let db = data.get::<DB>().unwrap().lock();
            let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
            let user = event.presence.user_id.get().unwrap();
            if let Ok(_) = guild_id.member(user.id) {
                let mut user_data = db.get_user(event.presence.user_id.0 as i64, guild_id.0 as i64).unwrap_or_else(|_| {
                    db.new_user(event.presence.user_id.0 as i64, guild_id.0 as i64).unwrap()
                });
                if guild_data.audit {
                    let audit_channel = ChannelId(guild_data.audit_channel as u64);
                    if user.tag() != user_data.username {
                        audit_channel.send_message(|m| m
                            .embed(|e| e
                                .title("Username changed")
                                .colour(Colours::Main.val())
                                .thumbnail(user.face())
                                .description(format!("**Old:** {}\n**New:** {}", user_data.username, user.tag()))
                        )).expect("Failed to send Message");
                        user_data.username = user.tag();
                    }
                }
                db.update_user(event.presence.user_id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
            };
        };
    }

    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let data = ctx.data.lock();
            let db = data.get::<DB>().unwrap().lock();
            db.new_guild(guild.id.0 as i64).expect("Failed to create guild entry");
            let owner = guild.owner_id.get().unwrap();
            GUILD_LOG.send_message(|m| m
                .embed(|e| e
                    .title("Joined Guild")
                    .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                    .colour(Colours::Green.val())
                    .description(format!("**Name:** {}\n**ID:** {}\n**Owner:** {} ({})",
                        guild.name,
                        guild.id.0,
                        owner.tag(),
                        owner.id.0))
                )).expect("Failed to send message");
        }
    }

    fn guild_delete(&self, ctx: Context, partial_guild: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {
        let owner = partial_guild.owner_id.get().unwrap();
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        db.del_guild(partial_guild.id.0 as i64).expect("Failed to delete guild entry");
        GUILD_LOG.send_message(|m| m
            .embed(|e| e
                .title("Left Guild")
                .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                .colour(Colours::Red.val())
                .description(format!("**Name:** {}\n**ID:** {}\n**Owner:** {} ({})",
                    partial_guild.name,
                    partial_guild.id.0,
                    owner.tag(),
                    owner.id.0))
            )).expect("Failed to send message");
    }

    // Join log and welcome message
    fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
        let user = member.user.read();
        let mut user_data = db.new_user(user.id.0 as i64, guild_id.0 as i64).unwrap();
        if guild_data.audit {
            let audit_channel = ChannelId(guild_data.audit_channel as u64);
            audit_channel.send_message(|m| m
                .embed(|e| e
                    .title("Member Joined")
                    .colour(Colours::Green.val())
                    .thumbnail(user.face())
                    .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                    .description(format!("<@{}>\n{}\n{}", user.id, user.tag(), user.id))
            )).expect("Failed to send message");
        }
        if guild_data.welcome {
            let welcome_channel = ChannelId(guild_data.welcome_channel as u64);
            welcome_channel.send_message(|m| m
                .content(guild_data.welcome_message)
            ).expect("Failed to send message");
        }
        user_data.username = user.tag();
        user_data.nickname = member.display_name().into_owned();
        user_data.roles = member.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>();
        db.update_user(user.id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
    }

    // Leave and kick log
    fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _: Option<Member>) {
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
        if guild_data.audit {
            let audit_channel = ChannelId(guild_data.audit_channel as u64);
            audit_channel.send_message(|m| m
                .embed(|e| e
                    .title("Member Left")
                    .colour(Colours::Red.val())
                    .thumbnail(user.face())
                    .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                    .description(format!("<@{}>\n{}\n{}", user.id, user.tag(), user.id))
            )).expect("Failed to send message");
        }
        db.del_user(user.id.0 as i64, guild_id.0 as i64).expect("Failed to delete user entry");
        thread::sleep(Duration::from_secs(3));
        if let Ok(audits) = guild_id.audit_logs(Some(20), None, None, Some(1)) {
            let (audit_id, audit) = audits.entries.iter().next().unwrap();
            if guild_data.modlog && audit.target_id == user.id.0 && (Utc::now().timestamp()-audit_id.created_at().timestamp())<5 {
                let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                modlog_channel.send_message(|m| m
                    .embed(|e| e
                        .title("Member Kicked")
                        .colour(Colours::Red.val())
                        .thumbnail(user.face())
                        .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                        .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                            user.tag(),
                            user.id.0,
                            user.mention(),
                            audit.user_id.get().unwrap().tag(),
                            audit.reason.clone().unwrap_or("None".to_string())
                        ))
                )).expect("Failed to send message");
            }
        };
    }

    // Nickname and Role changes
    fn guild_member_update(&self, ctx: Context, _: Option<Member>, new: Member) {
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        let guild_id = new.guild_id;
        let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
        let user = new.user.read();
        let mut user_data = db.get_user(user.id.0 as i64, guild_id.0 as i64).unwrap_or_else(|_| {
            db.new_user(user.id.0 as i64, guild_id.0 as i64).unwrap()
        });
        if guild_data.audit {
            let audit_channel = ChannelId(guild_data.audit_channel as u64);
            if let Some(nick) = new.nick {
                if nick != user_data.nickname {
                    audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Username changed")
                            .colour(Colours::Main.val())
                            .thumbnail(user.face())
                            .description(format!("**User: ** {}\n**Old:** {}\n**New:** {}", user.tag(), user_data.nickname, nick))
                    )).expect("Failed to send Message");
                    user_data.nickname = nick;
                }
            };
            let roles = new.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>();
            let mut roles_added = roles.clone();
            roles_added.retain(|e| !user_data.roles.contains(e));
            let mut roles_removed = user_data.roles.clone();
            roles_removed.retain(|e| !roles.contains(e));
            if !roles_added.is_empty() {
                    audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Roles changed")
                            .colour(Colours::Main.val())
                            .thumbnail(user.face())
                            .description(format!("**User: ** {}\n**Added:** {}", user.tag(), roles_added.iter().map(|r| RoleId(*r as u64).find().unwrap().name).collect::<Vec<String>>().join(", ")))
                    )).expect("Failed to send Message");
            }
            if !roles_removed.is_empty() {
                    audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Roles changed")
                            .colour(Colours::Main.val())
                            .thumbnail(user.face())
                            .description(format!("**User: ** {}\n**Removed:** {}", user.tag(), roles_removed.iter().map(|r| RoleId(*r as u64).find().unwrap().name).collect::<Vec<String>>().join(", ")))
                    )).expect("Failed to send Message");
            }
            user_data.roles = roles;
        }
        db.update_user(user.id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
    }

    fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, user: User) {
        thread::sleep(Duration::from_secs(3));
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        if let Ok(audits) = guild_id.audit_logs(Some(22), None, None, Some(1)) {
            let audit = audits.entries.values().next().unwrap();
            let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
            if guild_data.modlog && audit.target_id == user.id.0 {
                let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                modlog_channel.send_message(|m| m
                    .embed(|e| e
                        .title("Member Banned")
                        .colour(Colours::Red.val())
                        .thumbnail(user.face())
                        .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                        .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                            user.tag(),
                            user.id.0,
                            user.mention(),
                            audit.user_id.get().unwrap().tag(),
                            audit.reason.clone().unwrap_or("None".to_string())
                        ))
                )).expect("Failed to send message");
            }
        };
    }

    fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, user: User) {
        thread::sleep(Duration::from_secs(3));
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        if let Ok(audits) = guild_id.audit_logs(Some(23), None, None, Some(1)) {
            let audit = audits.entries.values().next().unwrap();
            let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
            if guild_data.modlog && audit.target_id == user.id.0 {
                let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                modlog_channel.send_message(|m| m
                    .embed(|e| e
                        .title("Member Unbanned")
                        .colour(Colours::Green.val())
                        .thumbnail(user.face())
                        .timestamp(format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%S")))
                        .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}",
                            user.tag(),
                            user.id.0,
                            user.mention(),
                            audit.user_id.get().unwrap().tag()
                        ))
                )).expect("Failed to send message");
            }
        };
    }
}

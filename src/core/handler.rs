use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::*;
use core::utils::*;
use db::models::UserUpdate;
use levenshtein::levenshtein;
use serenity::CACHE;
use serenity::model::gateway::{
    Game,
    GameType,
    Ready
};
use serenity::model::channel::Message;
use serenity::model::event::PresenceUpdateEvent;
use serenity::model::guild::{
    Guild,
    Member,
    PartialGuild
};
use serenity::model::id::{
    ChannelId,
    GuildId,
    MessageId,
    RoleId,
};
use serenity::model::user::User;
use serenity::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        debug!("Locking cache");
        CACHE.write().settings_mut().max_messages(MESSAGE_CACHE);
        let data = ctx.data.lock();
        if let Some(tc_lock) = data.get::<TC>() {
            let tc = tc_lock.lock();
            tc.load();
        }
        info!("Logged in as {}", ready.user.name);
    }

    fn cached(&self, ctx: Context, guilds: Vec<GuildId>) {
        let data = ctx.data.lock();
        let mut users  = Vec::new();
        for guild_id in guilds.iter() {
            debug!("Locking cache");
            let guild_lock = CACHE.read().guild(guild_id);
            if let Some(guild_lock) = guild_lock {
                let guild = guild_lock.read();
                let members = &guild.members;
                // TODO implement mass user update to expedite startup
                for (_, member) in members.iter() {
                    let u = member.user.read();
                    users.push(UserUpdate {
                        id: u.id.0 as i64,
                        guild_id: guild_id.0 as i64,
                        roles: member.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>(),
                        nickname: member.display_name().into_owned(),
                        username: u.tag()
                    });
                }
            } else { failed!(CACHE_GUILD_FAIL); }
        }
        for slice in guilds.iter().map(|e| e.0 as i64).collect::<Vec<i64>>().chunks(SLICE_SIZE) {
            check_error!(db.new_guilds(slice));
        }
        for slice in users.chunks(USER_SLICE_SIZE) {
            check_error!(db.upsert_users(slice));
        }

        let guild_count = guilds.len();
        if let Some(api) = data.get::<ApiClient>() {
            debug!("Locking cache");
            api.stats_update(CACHE.read().user.id.0, guild_count);
        } else { failed!(API_FAIL); }
        ctx.set_game(Game::listening(&format!("{} guilds | m!help", guild_count)));
        info!("Caching complete");
    }

    // Handle XP and last_message
    /*
    fn message(&self, _: Context, message: Message) {
        // These are only relevant in a guild context
        if message.author.bot { return; }
        if let Some(guild_id) = message.guild_id {
            match db.get_user(message.author.id.0 as i64, guild_id.0 as i64) {
                Ok(mut user_data) => {
                    user_data.last_message = message.timestamp.with_timezone(&Utc);
                    // TODO change xp logic
                    user_data.xp += 1;
                    check_error!(db.update_user(message.author.id.0 as i64, guild_id.0 as i64, user_data));
                },
                Err(why) => { failed!(DB_USER_FAIL, why); },
            }
        } else { failed!(GUILDID_FAIL); }
    }
    */

    fn message_delete(&self, _: Context, channel_id: ChannelId, message_id: MessageId) {
        debug!("Locking cache");
        let channel_lock = CACHE.read().guild_channel(&channel_id);
        if let Some(channel_lock) = channel_lock {
            let channel = channel_lock.read();
            let guild_id = channel.guild_id;
            match db.get_guild(guild_id.0 as i64) {
                Ok(guild_data) => {
                    let audit_channel = ChannelId(guild_data.audit_channel as u64);
                    if guild_data.audit && audit_channel.0 > 0 {
                        debug!("Locking cache");
                        let message = CACHE.read().message(&channel_id, &message_id);
                        if let Some(message) = message {
                            if message.author.bot { return; }
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Message Deleted")
                                    .colour(*colours::RED)
                                    .footer(|f| f.text(format!("ID: {}", message_id.0)))
                                    .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - {}\n**Content:**\n{}",
                                        message.author.tag(),
                                        message.author.id.0,
                                        message.author.mention(),
                                        channel.name,
                                        channel.id.0,
                                        channel.mention(),
                                        message.content_safe()))
                                    .timestamp(now!())
                            )));
                        } else {
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Uncached Message Deleted")
                                    .colour(*colours::RED)
                                    .footer(|f| f.text(format!("ID: {}", message_id.0)))
                                    .description(format!("**Channel:** {} ({}) - {}",
                                        channel.name,
                                        channel.id.0,
                                        channel.mention()))
                                    .timestamp(now!())
                            )));
                        }
                    }
                },
                Err(why) => { failed!(DB_GUILD_FAIL, why); },
            }
        } else { failed!(CACHE_CHANNEL_FAIL); }
    }

    // Edit logs
    fn message_update(&self, _: Context, old: Option<Message>, new: Message) {
        if new.author.bot { return; }
        if let Some(message) = old {
            let channel_lock = CACHE.read().guild_channel(&new.channel_id);
            if let Some(channel_lock) = channel_lock {
                let channel = channel_lock.read();
                let guild_id = channel.guild_id;
                match db.get_guild(guild_id.0 as i64) {
                    Ok(guild_data) => {
                        let audit_channel = ChannelId(guild_data.audit_channel as u64);
                        let distance = levenshtein(message.content.as_str(), new.content.as_str());
                        if guild_data.audit && audit_channel.0 > 0 && distance >= guild_data.audit_threshold as usize {
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Message Edited")
                                    .colour(*colours::MAIN)
                                    .footer(|f| f.text(format!("ID: {}", message.id.0)))
                                    .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - {}\n**Old Content:**\n{}\n**New Content:**\n{}",
                                        message.author.tag(),
                                        message.author.id.0,
                                        message.author.mention(),
                                        channel.name,
                                        channel.id.0,
                                        channel.mention(),
                                        message.content_safe(),
                                        new.content))
                                    .timestamp(now!())
                            )));
                        }
                    },
                    Err(why) => { failed!(DB_GUILD_FAIL, why); },
                }
            } else { failed!(CACHE_CHANNEL_FAIL); }
        }
    }

    // Username changes and Now Live! role
    fn presence_update(&self, _: Context, event: PresenceUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            match event.presence.user {
                Some(ref user_lock) => {
                    debug!("Getting user from lock: {:?} | {:?}", guild_id, event.presence.user_id);
                    let user = user_lock.read().clone();
                    debug!("User cloned and lock released: {:?} | {:?}", guild_id, event.presence.user_id);
                    if !user.bot {
                        let mut user_data = db.get_user(event.presence.user_id.0 as i64, guild_id.0 as i64).unwrap_or_else(|why| {
                            // TODO figure out how this is failing due to unique violation
                            debug!("{}", why);
                            db.new_user(event.presence.user_id.0 as i64, guild_id.0 as i64).expect("Failed to create user entry")
                        });
                        if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                            if guild_data.audit && guild_data.audit_channel > 0 {
                                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                                if user.tag() != user_data.username {
                                    audit_channel.send_message(|m| m
                                        .embed(|e| e
                                            .title("Username changed")
                                            .colour(*colours::MAIN)
                                            .thumbnail(user.face())
                                            .description(format!("**Old:** {}\n**New:** {}", user_data.username, user.tag()))
                                    )).expect("Failed to send Message");
                                    user_data.username = user.tag();
                                }
                            }
                        } else { failed!(DB_GUILD_FAIL); }
                        db.update_user(event.presence.user_id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
                        if guild_id == TRANSCEND {
                            debug!("Attempt to get member");
                            let member = CACHE.read().member(guild_id, user.id);
                            if let Some(mut member) = member {
                                debug!("Got member");
                                match event.presence.game {
                                    Some(ref game) => {
                                        if let GameType::Streaming = game.kind {
                                            if member.roles.contains(&NOW_LIVE) {
                                                let _ = member.add_role(NOW_LIVE);
                                            }
                                        }
                                    },
                                    None => {
                                        if !member.roles.contains(&NOW_LIVE) {
                                            let _ = member.remove_role(NOW_LIVE);
                                        }
                                    },
                                }
                            } else { failed!(MEMBER_FAIL); }
                        }
                    }
                },
                None => {},
            }
        } else { failed!(GUILDID_FAIL); }
    }

    fn guild_create(&self, _: Context, guild: Guild, is_new: bool) {
        if is_new {
            match db.new_guild(guild.id.0 as i64) {
                Ok(_) => {
                    match guild.owner_id.get() {
                        Ok(owner) => {
                        check_error!(GUILD_LOG.send_message(|m| m
                            .embed(|e| e
                                .title("Joined Guild")
                                .timestamp(now!())
                                .colour(*colours::GREEN)
                                .description(format!("**Name:** {}\n**ID:** {}\n**Owner:** {} ({})",
                                    guild.name,
                                    guild.id.0,
                                    owner.tag(),
                                    owner.id.0))
                        )));
                        },
                        Err(why) => { failed!(USER_FAIL, why); },
                    }
                },
                Err(why) => { failed!(DB_GUILD_ENTRY_FAIL, why); }
            }
        }
    }

    fn guild_delete(&self, _: Context, partial_guild: PartialGuild, _: Option<Arc<RwLock<Guild>>>) {
        match db.del_guild(partial_guild.id.0 as i64) {
            Ok(_) => {
                match partial_guild.owner_id.get() {
                    Ok(owner) => {
                        check_error!(GUILD_LOG.send_message(|m| m
                            .embed(|e| e
                                .title("Left Guild")
                                .timestamp(now!())
                                .colour(*colours::RED)
                                .description(format!("**Name:** {}\n**ID:** {}\n**Owner:** {} ({})",
                                    partial_guild.name,
                                    partial_guild.id.0,
                                    owner.tag(),
                                    owner.id.0))
                        )));
                    },
                    Err(why) => { failed!(USER_FAIL, why); },
                }
            },
            Err(why) => { failed!(DB_GUILD_DEL_FAIL, why); }
        }
    }

    // Join log and welcome message
    fn guild_member_addition(&self, _: Context, guild_id: GuildId, member: Member) {
        match db.get_guild(guild_id.0 as i64) {
            Ok(guild_data) => {
                let user = member.user.read();
                match db.new_user(user.id.0 as i64, guild_id.0 as i64) {
                    Ok(mut user_data) => {
                        if guild_data.audit && guild_data.audit_channel > 0 {
                            let audit_channel = ChannelId(guild_data.audit_channel as u64);
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Member Joined")
                                    .colour(*colours::GREEN)
                                    .thumbnail(user.face())
                                    .timestamp(now!())
                                    .description(format!("<@{}>\n{}\n{}", user.id, user.tag(), user.id))
                            )));
                        }
                        if guild_data.welcome && guild_data.welcome_channel > 0 {
                            let channel = ChannelId(guild_data.welcome_channel as u64);
                            if guild_data.welcome_type.as_str() == "embed" {
                                check_error!(send_welcome_embed(guild_data.welcome_message, &member, channel));
                            } else {
                                check_error!(channel.say(parse_welcome_items(guild_data.welcome_message, &member)));
                            }
                        }
                        user_data.username = user.tag();
                        user_data.nickname = member.display_name().into_owned();
                        user_data.roles = member.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>();
                        check_error!(db.update_user(user.id.0 as i64, guild_id.0 as i64, user_data));
                    },
                    Err(why) => { failed!(DB_USER_ENTRY_FAIL, why); }
                }
            },
            Err(why) => { failed!(DB_GUILD_FAIL, why); }
        }
    }

    // Leave and kick log
    fn guild_member_removal(&self, _: Context, guild_id: GuildId, user: User, _: Option<Member>) {
        match db.get_guild(guild_id.0 as i64) {
            Ok(guild_data) => {
                if guild_data.audit && guild_data.audit_channel > 0 {
                    let audit_channel = ChannelId(guild_data.audit_channel as u64);
                    check_error!(audit_channel.send_message(|m| m
                        .embed(|e| e
                            .title("Member Left")
                            .colour(*colours::RED)
                            .thumbnail(user.face())
                            .timestamp(now!())
                            .description(format!("<@{}>\n{}\n{}", user.id, user.tag(), user.id))
                    )));
                }
                check_error!(db.del_user(user.id.0 as i64, guild_id.0 as i64));
                thread::sleep(Duration::from_secs(3));
                if let Ok(audits) = guild_id.audit_logs(Some(20), None, None, Some(1)) {
                    if let Some((audit_id, audit)) = audits.entries.iter().next() {
                        if guild_data.modlog && guild_data.modlog_channel > 0 && audit.target_id == user.id.0 && (Utc::now().timestamp()-audit_id.created_at().timestamp())<5 {
                            let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                            check_error!(modlog_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Member Kicked")
                                    .colour(*colours::RED)
                                    .thumbnail(user.face())
                                    .timestamp(now!())
                                    .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                                        user.tag(),
                                        user.id.0,
                                        user.mention(),
                                        match audit.user_id.get() {
                                            Ok(u) => u.tag(),
                                            Err(_) => format!("{}", audit.user_id.0)
                                        },
                                        audit.reason.clone().unwrap_or("None".to_string())
                                    ))
                            )));
                        }
                    }
                }
            },
            Err(why) => { failed!(DB_GUILD_FAIL, why); }
        }
    }

    // Nickname and Role changes
    fn guild_member_update(&self, _: Context, _: Option<Member>, new: Member) {
        let guild_id = new.guild_id;
        match db.get_guild(guild_id.0 as i64) {
            Ok(guild_data) => {
                let user = new.user.read();
                let mut user_data = db.get_user(user.id.0 as i64, guild_id.0 as i64).unwrap_or_else(|_| {
                    db.new_user(user.id.0 as i64, guild_id.0 as i64).expect("Failed to create user")
                });
                if guild_data.audit && guild_data.audit_channel > 0 {
                    let audit_channel = ChannelId(guild_data.audit_channel as u64);
                    if let Some(nick) = new.nick {
                        if nick != user_data.nickname {
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Username changed")
                                    .colour(*colours::MAIN)
                                    .thumbnail(user.face())
                                    .description(format!("**User: ** {}\n**Old:** {}\n**New:** {}", user.tag(), user_data.nickname, nick))
                            )));
                            user_data.nickname = nick;
                        }
                    };
                    let roles = new.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>();
                    let mut roles_added = roles.clone();
                    roles_added.retain(|e| !user_data.roles.contains(e));
                    let mut roles_removed = user_data.roles.clone();
                    roles_removed.retain(|e| !roles.contains(e));
                    if !roles_added.is_empty() {
                        let roles_added = roles_added.iter().map(|r| {
                            match RoleId(*r as u64).find() {
                                Some(role) => role.name,
                                None => format!("{}", r),
                            }
                        }).collect::<Vec<String>>();
                        check_error!(audit_channel.send_message(|m| m
                            .embed(|e| e
                                .title("Roles changed")
                                .colour(*colours::MAIN)
                                .thumbnail(user.face())
                                .description(format!("**User: ** {}\n**Added:** {}", user.tag(), roles_added.join(", ")))
                        )));
                    }
                    if !roles_removed.is_empty() {
                        let roles_removed = roles_added.iter().map(|r| {
                            match RoleId(*r as u64).find() {
                                Some(role) => role.name,
                                None => format!("{}", r),
                            }
                        }).collect::<Vec<String>>();
                        check_error!(audit_channel.send_message(|m| m
                            .embed(|e| e
                                .title("Roles changed")
                                .colour(*colours::MAIN)
                                .thumbnail(user.face())
                                .description(format!("**User: ** {}\n**Removed:** {}", user.tag(), roles_removed.join(", ")))
                        )));
                    }
                    user_data.roles = roles;
                }
                check_error!(db.update_user(user.id.0 as i64, guild_id.0 as i64, user_data));
            },
            Err(why) => { failed!(DB_GUILD_FAIL, why); }
        }
    }

    fn guild_ban_addition(&self, _: Context, guild_id: GuildId, user: User) {
        thread::sleep(Duration::from_secs(3));
        if let Ok(audits) = guild_id.audit_logs(Some(22), None, None, Some(1)) {
            if let Some(audit) = audits.entries.values().next() {
                match db.get_guild(guild_id.0 as i64) {
                    Ok(guild_data) => {
                        if guild_data.modlog && guild_data.modlog_channel > 0 && audit.target_id == user.id.0 {
                            let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                            check_error!(modlog_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Member Banned")
                                    .colour(*colours::RED)
                                    .thumbnail(user.face())
                                    .timestamp(now!())
                                    .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                                        user.tag(),
                                        user.id.0,
                                        user.mention(),
                                        match audit.user_id.get() {
                                            Ok(u) => u.tag(),
                                            Err(_) => format!("{}", audit.user_id.0)
                                        },
                                        audit.reason.clone().unwrap_or("None".to_string())
                                    ))
                            )));
                        }
                    },
                    Err(why) => { failed!(DB_GUILD_FAIL, why); }
                }
            }
        }
    }

    fn guild_ban_removal(&self, _: Context, guild_id: GuildId, user: User) {
        thread::sleep(Duration::from_secs(3));
        if let Ok(audits) = guild_id.audit_logs(Some(23), None, None, Some(1)) {
            if let Some(audit) = audits.entries.values().next() {
                match db.get_guild(guild_id.0 as i64) {
                    Ok(guild_data) => {
                        if guild_data.modlog && guild_data.modlog_channel > 0 && audit.target_id == user.id.0 {
                            let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                            check_error!(modlog_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Member Unbanned")
                                    .colour(*colours::GREEN)
                                    .thumbnail(user.face())
                                    .timestamp(now!())
                                    .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}\n**Reason:** {}",
                                        user.tag(),
                                        user.id.0,
                                        user.mention(),
                                        match audit.user_id.get() {
                                            Ok(u) => u.tag(),
                                            Err(_) => format!("{}", audit.user_id.0)
                                        },
                                        audit.reason.clone().unwrap_or("None".to_string())
                                    ))
                            )));
                        }
                    },
                    Err(why) => { failed!(DB_GUILD_FAIL, why); }
                }
            }
        }
    }
}

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
};
use serenity::model::user::User;
use serenity::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        CACHE.write().settings_mut().max_messages(MESSAGE_CACHE);
        let data = ctx.data.lock();
        if let Some(tc_lock) = data.get::<TC>() {
            let tc = tc_lock.lock();
            tc.load();
        }
        info!("Logged in as {}", ready.user.name);
    }

    fn cached(&self, ctx: Context, guilds: Vec<GuildId>) {
        let mut users  = Vec::new();
        for guild_id in guilds.iter() {
            let guild_lock = CACHE.read().guild(guild_id);
            if let Some(guild_lock) = guild_lock {
                let guild = guild_lock.read();
                let members = &guild.members;
                for (_, member) in members.iter() {
                    let u = member.user.read();
                    users.push(UserUpdate {
                        id: u.id.0 as i64,
                        guild_id: guild_id.0 as i64,
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
        let data = ctx.data.lock();
        if let Some(api) = data.get::<ApiClient>() {
            api.stats_update(CACHE.read().user.id.0, guild_count);
        } else { failed!(API_FAIL); }
        ctx.set_game(Game::listening(&format!("{} guilds | m!help", guild_count)));
        info!("Caching complete");
    }

    // Handle XP and last_message
    fn message(&self, _: Context, message: Message) {
        // These are only relevant in a guild context and for non-bots
        if message.author.bot { return; }
        if message.is_private() { return; }
        if let Some(guild_id) = message.guild_id {
            if let Ok(mut user_data) = db.get_or_new_user(message.author.id.0 as i64, guild_id.0 as i64) {
                let now = message.timestamp.with_timezone(&Utc);
                let diff = now.timestamp() - user_data.last_message.timestamp();
                user_data.last_message = now;
                if diff > MIN as i64 {
                    user_data.xp += 1;
                }
                check_error!(db.update_user(message.author.id.0 as i64, guild_id.0 as i64, user_data));
            }
        } else { failed!(GUILDID_FAIL); }
    }

    fn message_delete(&self, _: Context, channel_id: ChannelId, message_id: MessageId) {
        let (channel_name, guild_id) = {
            let channel_lock = CACHE.read().guild_channel(&channel_id);
            if let Some(channel_lock) = channel_lock {
                let ch = channel_lock.read();
                (ch.name.clone(), ch.guild_id.clone())
            } else {
                ("unknown".to_string(), GuildId(0))
            }
        };
        match db.get_guild(guild_id.0 as i64) {
            Ok(guild_data) => {
                if guild_data.logging.contains(&String::from("message_delete")) { return; }
                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                if guild_data.audit && audit_channel.0 > 0 {
                    let message = CACHE.read().message(&channel_id, &message_id);
                    if let Some(message) = message {
                        if message.author.bot { return; }
                        check_error!(audit_channel.send_message(|m| m
                            .embed(|e| e
                                .title("Message Deleted")
                                .colour(*colours::RED)
                                .footer(|f| f.text(format!("ID: {}", message_id.0)))
                                .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - <#{}>\n**Content:**\n{}",
                                    message.author.tag(),
                                    message.author.id.0,
                                    message.author.mention(),
                                    channel_name,
                                    channel_id.0,
                                    channel_id.0,
                                    message.content_safe()))
                                .timestamp(now!())
                        )));
                    } else {
                        check_error!(audit_channel.send_message(|m| m
                            .embed(|e| e
                                .title("Uncached Message Deleted")
                                .colour(*colours::RED)
                                .footer(|f| f.text(format!("ID: {}", message_id.0)))
                                .description(format!("**Channel:** {} ({}) - <#{}>",
                                    channel_name,
                                    channel_id.0,
                                    channel_id.0))
                                .timestamp(now!())
                        )));
                    }
                }
            },
            Err(why) => { failed!(DB_GUILD_FAIL, why); },
        }
    }

    // Edit logs
    fn message_update(&self, _: Context, old: Option<Message>, new: Message) {
        if new.author.bot { return; }
        if let None = new.edited_timestamp { return; }
        if let Some(message) = old {
            if let Some(guild_id) = new.guild_id {
                let channel_id = new.channel_id;
                let channel_name = {
                    let channel_lock = CACHE.read().guild_channel(&channel_id);
                    if let Some(channel_lock) = channel_lock {
                        let ch = channel_lock.read();
                        ch.name.clone()
                    } else {
                        "unknown".to_string()
                    }
                };
                match db.get_guild(guild_id.0 as i64) {
                    Ok(guild_data) => {
                        if guild_data.logging.contains(&String::from("message_edit")) { return; }
                        let audit_channel = ChannelId(guild_data.audit_channel as u64);
                        let distance = levenshtein(message.content.as_str(), new.content.as_str());
                        if guild_data.audit && audit_channel.0 > 0 && distance >= guild_data.audit_threshold as usize {
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Message Edited")
                                    .colour(*colours::MAIN)
                                    .footer(|f| f.text(format!("ID: {}", message.id.0)))
                                    .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - <#{}>\n**Old Content:**\n{}\n**New Content:**\n{}",
                                        message.author.tag(),
                                        message.author.id.0,
                                        message.author.mention(),
                                        channel_name,
                                        channel_id.0,
                                        channel_id.0,
                                        message.content_safe(),
                                        new.content))
                                    .timestamp(now!())
                            )));
                        }
                    },
                    Err(why) => { failed!(DB_GUILD_FAIL, why); },
                }
            } else { failed!(GUILDID_FAIL); }
        }
    }

    // Username changes and Now Live! role
    fn presence_update(&self, _: Context, event: PresenceUpdateEvent) {
        if let Some(guild_id) = event.guild_id {
            match event.presence.user {
                Some(ref user_lock) => {
                    let (user_bot, user_tag, user_face) = {
                        let u = user_lock.read();
                        (u.bot, u.tag(), u.face())
                    };
                    if !user_bot {
                        if guild_id == TRANSCEND {
                            let member = CACHE.read().member(guild_id, event.presence.user_id);
                            if let Some(mut member) = member {
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
                        if let Ok(mut user_data) = db.get_or_new_user(event.presence.user_id.0 as i64, guild_id.0 as i64) {
                            if user_tag != user_data.username && user_data.username != String::new() {
                                if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                                    if guild_data.logging.contains(&String::from("username_change")) { return; }
                                    if guild_data.audit && guild_data.audit_channel > 0 {
                                    let audit_channel = ChannelId(guild_data.audit_channel as u64);
                                        audit_channel.send_message(|m| m
                                            .embed(|e| e
                                                .title("Username changed")
                                                .colour(*colours::MAIN)
                                                .thumbnail(user_face)
                                                .description(format!("**Old:** {}\n**New:** {}", user_data.username, user_tag))
                                        )).expect("Failed to send Message");
                                    }
                                } else { failed!(DB_GUILD_FAIL); }
                                user_data.username = user_tag;
                                db.update_user(event.presence.user_id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
                            }
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
                    match guild.owner_id.to_user() {
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
                match partial_guild.owner_id.to_user() {
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
        let mut banned = false;
        let mut reason = None;
        if let Ok(hackbans) = db.get_hackbans(guild_id.0 as i64) {
            hackbans.iter().for_each(|e| {
                if e.id as u64 == member.user.read().id.0 {
                    banned = true;
                    reason = e.reason.clone();
                }
            });
        }
        if banned {
            if let Some(ref r) = reason {
                check_error!(member.ban::<String>(r));
            } else {
                check_error!(member.ban(&0));
            }
        } else {
            match db.get_guild(guild_id.0 as i64) {
                Ok(guild_data) => {
                    if guild_data.logging.contains(&String::from("member_join")) { return; }
                    let (user_id, user_face, user_tag) = {
                        let u = member.user.read();
                        (u.id, u.face(), u.tag())
                    };
                    match db.new_user(user_id.0 as i64, guild_id.0 as i64) {
                        Ok(mut user_data) => {
                            if guild_data.audit && guild_data.audit_channel > 0 {
                                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                                check_error!(audit_channel.send_message(|m| m
                                    .embed(|e| e
                                        .title("Member Joined")
                                        .colour(*colours::GREEN)
                                        .thumbnail(user_face)
                                        .timestamp(now!())
                                        .description(format!("<@{}>\n{}\n{}", user_id, user_tag, user_id))
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
                            user_data.username = user_tag;
                            user_data.nickname = member.display_name().into_owned();
                            user_data.roles = member.roles.iter().map(|e| e.0 as i64).collect::<Vec<i64>>();
                            check_error!(db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data));
                        },
                        Err(why) => { failed!(DB_USER_ENTRY_FAIL, why); }
                    }
                },
                Err(why) => { failed!(DB_GUILD_FAIL, why); }
            }
        }
    }

    // Leave and kick log
    fn guild_member_removal(&self, _: Context, guild_id: GuildId, user: User, _: Option<Member>) {
        match db.get_guild(guild_id.0 as i64) {
            Ok(guild_data) => {
                check_error!(db.del_user(user.id.0 as i64, guild_id.0 as i64));
                if guild_data.logging.contains(&String::from("member_leave")) { return; }
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
                if guild_data.logging.contains(&String::from("member_kick")) { return; }
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
                                        match audit.user_id.to_user() {
                                            Ok(u) => u.tag(),
                                            Err(_) => audit.user_id.0.to_string()
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
    fn guild_member_update(&self, _: Context, old: Option<Member>, new: Member) {
        let guild_id = new.guild_id;
        if let Some(old) = old {
            match db.get_guild(guild_id.0 as i64) {
                Ok(guild_data) => {
                    let (user_face, user_tag) = {
                        let u = new.user.read();
                        (u.face(), u.tag())
                    };
                    if guild_data.audit && guild_data.audit_channel > 0 {
                        if guild_data.logging.contains(&String::from("nickname_change")) { return; }
                        let audit_channel = ChannelId(guild_data.audit_channel as u64);
                        if new.nick != old.nick {
                            let old_nick = old.nick.clone().unwrap_or("None".to_string());
                            let new_nick = new.nick.clone().unwrap_or("None".to_string());
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Nickname changed")
                                    .colour(*colours::MAIN)
                                    .thumbnail(&user_face)
                                    .description(format!(
                                        "**User: ** {}\n**Old:** {}\n**New:** {}",
                                        user_tag,
                                        old_nick,
                                        new_nick
                                    ))
                            )));
                        };
                        if guild_data.logging.contains(&String::from("role_change")) { return; }
                        let mut roles_added = new.roles.clone();
                        roles_added.retain(|e| !old.roles.contains(e));
                        if !roles_added.is_empty() {
                            let roles_added = roles_added.iter()
                                .map(|r| match r.to_role_cached() {
                                    Some(role) => role.name,
                                    None => r.0.to_string(),
                                })
                                .collect::<Vec<String>>();
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Roles changed")
                                    .colour(*colours::MAIN)
                                    .thumbnail(&user_face)
                                    .description(format!("**User: ** {}\n**Added:** {}", user_tag, roles_added.join(", ")))
                            )));
                        }
                        let mut roles_removed = old.roles.clone();
                        roles_removed.retain(|e| !new.roles.contains(e));
                        if !roles_removed.is_empty() {
                            let roles_removed = roles_removed.iter()
                                .map(|r| match r.to_role_cached() {
                                    Some(role) => role.name,
                                    None => r.0.to_string(),
                                })
                                .collect::<Vec<String>>();
                            check_error!(audit_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Roles changed")
                                    .colour(*colours::MAIN)
                                    .thumbnail(&user_face)
                                    .description(format!("**User: ** {}\n**Removed:** {}", user_tag, roles_removed.join(", ")))
                            )));
                        }
                    }
                },
                Err(why) => { failed!(DB_GUILD_FAIL, why); }
            }
        }
    }

    fn guild_ban_addition(&self, _: Context, guild_id: GuildId, user: User) {
        thread::sleep(Duration::from_secs(3));
        if let Ok(audits) = guild_id.audit_logs(Some(22), None, None, Some(1)) {
            if let Some(audit) = audits.entries.values().next() {
                match db.get_guild(guild_id.0 as i64) {
                    Ok(guild_data) => {
                        if guild_data.logging.contains(&String::from("member_ban")) { return; }
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
                                        match audit.user_id.to_user() {
                                            Ok(u) => u.tag(),
                                            Err(_) => audit.user_id.0.to_string(),
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
                        if guild_data.logging.contains(&String::from("member_unban")) { return; }
                        if guild_data.modlog && guild_data.modlog_channel > 0 && audit.target_id == user.id.0 {
                            let modlog_channel = ChannelId(guild_data.modlog_channel as u64);
                            check_error!(modlog_channel.send_message(|m| m
                                .embed(|e| e
                                    .title("Member Unbanned")
                                    .colour(*colours::GREEN)
                                    .thumbnail(user.face())
                                    .timestamp(now!())
                                    .description(format!("**Member:** {} ({}) - {}\n**Responsible Moderator:** {}",
                                        user.tag(),
                                        user.id.0,
                                        user.mention(),
                                        match audit.user_id.to_user() {
                                            Ok(u) => u.tag(),
                                            Err(_) => audit.user_id.0.to_string(),
                                        }
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

use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::Owner;
use core::utils::check_rank;
use modules::commands::{
    admins,
    everyone,
    moderators,
    owners
};
use serenity::framework::{
    StandardFramework,
    standard::{
        help_commands,
        HelpBehaviour,
    }
};
use serenity::model::channel::{
    Channel,
    Message,
};
use serenity::model::id::{
    GuildId,
    UserId
};
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::collections::HashSet;

pub struct MomijiFramework;

impl MomijiFramework {
    pub fn new(owners: HashSet<UserId>) -> StandardFramework {
        StandardFramework::new()
            .configure(|c| c
                .allow_whitespace(true)
                .allow_dm(true)
                .on_mention(true)
                .ignore_bots(true)
                .case_insensitivity(true)
                .delimiters(vec![","," "])
                .owners(owners)
                .prefix("m!")
                .dynamic_prefix(|_, message| {
                    if message.is_private() {
                        return Some(String::new());
                    } else {
                        let guild_id = message.guild_id.unwrap_or(GuildId(0));
                        if let Ok(settings) = db.get_guild(guild_id.0 as i64) {
                            return Some(settings.prefix);
                        }
                    }
                    None
                }))
            .before(|ctx, message, command_name| {
                debug!("Received command {} from {}", command_name, message.author.tag());
                if let false = message.is_private() {
                    let guild_id = message.guild_id.unwrap_or(GuildId(0));
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if guild_data.ignored_channels.contains(&(message.channel_id.0 as i64)) {
                            if get_highest_rank(ctx, message) < guild_data.ignore_level {
                                return false;
                            }
                        }
                        if guild_data.commands.contains(&command_name.to_string()) {
                            return false;
                        }
                    }
                }
                true
            })
            .after(|_, message, cmd_name, error| {
                let guild = match message.guild() {
                    Some(lock) => {
                        let g = lock.read();
                        format!("{} ({})", g.name, g.id.0)
                    },
                    None => String::from("Private"),
                };
                let channel = if let Some(ch) = message.channel() {
                    match ch {
                        Channel::Guild(c) => {
                            let c = c.read();
                            format!("{} ({})", c.name, c.id.0)
                        },
                        Channel::Private(_) => ch.id().0.to_string(),
                        _ => String::new(),
                    }
                } else { String::new() };
                check_error!(COMMAND_LOG.send_message(|m| m
                    .embed(|e| e
                        .description(format!("**Guild:** {}\n**Channel:** {}\n**Author:** {} ({})\n**ID:** {}",
                           guild,
                           channel,
                           message.author.tag(),
                           message.author.id.0,
                           message.id.0
                        ))
                        .field("Content", message.content_safe(), false)
                        .timestamp(now!())
                        .colour(*colours::MAIN)
                )));
                if let Err(why) = error {
                    // TODO do some actual matching here so you can provide more details
                    check_error!(message.channel_id.say(format!("Something went wrong with the command. Here's the error: {:?}", why)));
                    check_error!(ERROR_LOG.send_message(|m| m
                        .embed(|e| e
                            .description(format!("{:?}", why))
                            .field("Message", message.id.0.to_string(), true)
                            .field("Channel", message.channel_id.0.to_string(), true)
                            .field("Command", cmd_name, true)
                            .field("Message Content", message.content_safe(), false)
                            .timestamp(now!())
                            .colour(*colours::RED)
                    )));
                }
            })
            .customised_help(help_commands::plain, |c| c
                .no_help_available_text("No help is available on this command.")
                .usage_label("Usage")
                .usage_sample_label("Example")
                .aliases_label("Aliases")
                .guild_only_text("Guild only")
                .dm_only_text("DM only")
                .dm_and_guilds_text("DM or Guild")
                .command_not_found_text("Command not found.")
                .lacking_role(HelpBehaviour::Hide)
                .lacking_permissions(HelpBehaviour::Hide)
                .wrong_channel(HelpBehaviour::Strike)
                .embed_success_colour(*colours::MAIN)
                .embed_error_colour(*colours::RED))
            .bucket("weather", 30, DAY as i64, 1000)
            .group("For Everyone", |g| g
                .help_available(true)
                .command("anime", |c| c
                    .cmd(everyone::Anime))
                .command("botinfo", |c| c
                    .cmd(everyone::BotInfo))
                .command("cat", |c| c
                    .cmd(everyone::Cat))
                .command("dog", |c| c
                    .cmd(everyone::Dog))
                .command("joke", |c| c
                    .cmd(everyone::DadJoke))
                .command("manga", |c| c
                    .cmd(everyone::Manga))
                .command("now", |c| c
                    .cmd(everyone::Now))
                .command("ping", |c| c
                    .cmd(everyone::Ping))
                .command("prefix", |c| c
                    .cmd(everyone::Prefix))
                .command("remind", |c| c
                    .cmd(everyone::Reminder))
                .command("roleinfo", |c| c
                    .cmd(everyone::RoleInfo))
                .command("roll", |c| c
                    .cmd(everyone::Roll))
                .command("serverinfo", |c| c
                    .cmd(everyone::ServerInfo))
                .command("tags", |c| c
                    .cmd(everyone::TagList))
                .command("urban", |c| c
                    .cmd(everyone::Urban))
                .command("userinfo", |c| c
                    .cmd(everyone::UserInfo))
                .command("weather", |c| c
                    .cmd(everyone::Weather)))
            .group("Tags", |g| g
                .help_available(true)
                .guild_only(true)
                .prefix("tag")
                .default_cmd(everyone::TagSingle)
                .command("show", |c| c
                    .cmd(everyone::TagSingle))
                .command("add", |c| c
                    .cmd(everyone::TagAdd))
                .command("del", |c| c
                    .cmd(everyone::TagRemove))
                .command("edit", |c| c
                    .cmd(everyone::TagEdit))
                .command("list", |c| c
                    .cmd(everyone::TagList)))
            .group("NSFW", |g| g
                .help_available(true)
                .check(|_,message,_,_| {
                    if let Ok(channel) = message.channel_id.to_channel() {
                        if channel.is_nsfw() {
                            true
                        } else {
                            check_error!(message.channel_id.say("Command only available in NSFW channels."));
                            false
                        }
                    } else {
                        check_error!(message.channel_id.say("Failed to get the channel info. I can't tell if this channel is NSFW."));
                        false
                }})
                .command("e621", |c| c
                    .cmd(everyone::Furry)))
            .group("Self Roles", |g| g
                .help_available(true)
                .guild_only(true)
                .command("role", |c| c
                    .cmd(everyone::AddSelfRole))
                .command("derole", |c| c
                    .cmd(everyone::RemoveSelfRole))
                .command("roles", |c| c
                    .cmd(everyone::ListSelfRoles)))
            .group("Role Management", |g| g
                .guild_only(true)
                .help_available(true)
                .required_permissions(Permissions::MANAGE_ROLES)
                .command("register", |c| c
                    .cmd(moderators::Register))
                .command("addrole", |c| c
                    .cmd(moderators::AddRole))
                .command("removerole", |c| c
                    .cmd(moderators::RemoveRole))
                .command("rolecolour", |c| c
                    .cmd(moderators::RoleColour)))
            .group("Miscellaneous", |g| g
                .guild_only(true)
                .help_available(true)
                // TODO Fix these perms
                .required_permissions(Permissions::MANAGE_MESSAGES)
                .command("mute", |c| c
                    .cmd(moderators::Mute))
                .command("unmute", |c| c
                    .cmd(moderators::Unmute))
                .command("modinfo", |c| c
                    .cmd(moderators::ModInfo)))
            .group("Hackbans", |g| g
                .prefixes(vec!["hackban", "hb"])
                .guild_only(true)
                .help_available(true)
                .required_permissions(Permissions::BAN_MEMBERS)
                .command("add", |c| c
                    .cmd(moderators::HackbanAdd))
                .command("remove", |c| c
                    .cmd(moderators::HackbanRemove))
                .command("list", |c| c
                    .cmd(moderators::HackbanList)))
            .group("Notes", |g| g
                .prefix("note")
                .guild_only(true)
                .help_available(true)
                .required_permissions(Permissions::MANAGE_MESSAGES)
                .command("add", |c| c
                    .cmd(moderators::NoteAdd))
                .command("del", |c| c
                    .cmd(moderators::NoteRemove))
                .command("list", |c| c
                    .cmd(moderators::NoteList)))
            .group("Watchlist", |g| g
                .prefixes(vec!["watchlist", "wl"])
                .guild_only(true)
                .help_available(true)
                .default_cmd(moderators::WatchlistList)
                .required_permissions(Permissions::MANAGE_MESSAGES)
                .command("add", |c| c
                    .cmd(moderators::WatchlistAdd))
                .command("del", |c| c
                    .cmd(moderators::WatchlistRemove))
                .command("list", |c| c
                    .cmd(moderators::WatchlistList)))
            .group("Management", |g| g
                .guild_only(true)
                .help_available(true)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("setup", |c| c
                    .cmd(admins::SetupMute))
                .command("prune", |c| c
                    .cmd(admins::Prune)))
            .group("Ignore Channels", |g| g
                .guild_only(true)
                .help_available(true)
                .prefix("ignore")
                .default_cmd(admins::IgnoreList)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("add", |c| c
                    .cmd(admins::IgnoreAdd))
                .command("remove", |c| c
                    .cmd(admins::IgnoreRemove))
                .command("list", |c| c
                    .cmd(admins::IgnoreList)))
            .group("Premium", |g| g
                .guild_only(true)
                .help_available(true)
                .prefixes(vec!["p", "premium", "prem"])
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("register_member", |c| c
                    .cmd(admins::PRegisterMember))
                .command("register_cooldown", |c| c
                    .cmd(admins::PRegisterCooldown))
                .command("register_duration", |c| c
                    .cmd(admins::PRegisterDuration))
                .command("register_roles", |c| c
                    .cmd(admins::PRegisterRestrictions)))
            .group("Tests", |g| g
                .guild_only(true)
                .help_available(true)
                .prefix("test")
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("welcome", |c| c
                    .cmd(admins::TestWelcome)))
            .group("Self Role Management", |g| g
                .help_available(true)
                .guild_only(true)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("csr", |c| c
                    .cmd(admins::CreateSelfRole))
                .command("dsr", |c| c
                    .cmd(admins::DeleteSelfRole))
                .command("esr", |c| c
                    .cmd(admins::EditSelfRole)))
            .group("Config", |g| g
                .help_available(true)
                .guild_only(true)
                .prefixes(vec!["config", "conf"])
                .default_cmd(admins::ConfigList)
                .required_permissions(Permissions::MANAGE_GUILD)
                .command("list", |c| c
                    .cmd(admins::ConfigList))
                .command("raw", |c| c
                    .cmd(admins::ConfigRaw))
                .command("prefix", |c| c
                    .cmd(admins::ConfigPrefix))
                .command("autorole", |c| c
                    .cmd(admins::ConfigAutorole))
                .command("admin", |c| c
                    .cmd(admins::ConfigAdmin))
                .command("mod", |c| c
                    .cmd(admins::ConfigMod))
                .command("audit", |c| c
                    .cmd(admins::ConfigAudit))
                .command("modlog", |c| c
                    .cmd(admins::ConfigModlog))
                .command("welcome", |c| c
                    .cmd(admins::ConfigWelcome))
                .command("introduction", |c| c
                    .cmd(admins::ConfigIntroduction))
                .command("command", |c| c
                    .cmd(admins::ConfigCommands))
                .command("log", |c| c
                    .cmd(admins::ConfigLogs)))
            .group("Owner Only", |g| g
                .owners_only(true)
                .help_available(false)
                .command("op", |c| c
                    .cmd(owners::Premium))
                .command("log", |c| c
                    .cmd(owners::Log)))
    }
}

fn get_highest_rank(ctx: &mut Context, message: &Message) -> i16 {
    {
        let data = ctx.data.lock();
        if let Some(owner) = data.get::<Owner>() {
            if *owner == message.author.id {
                return 4;
            }
        }
    }
    if let Some(guild_lock) = message.guild() {
        let (guild_id, owner_id) = {
            let guild = guild_lock.read();
            (guild.id, guild.owner_id)
        };
        if message.author.id == owner_id { return 3; }
        if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
            if let Ok(member) = guild_id.member(message.author.id.clone()) {
                if check_rank(guild_data.admin_roles, &member.roles) {
                    return 2;
                } else if check_rank(guild_data.mod_roles, &member.roles) {
                    return 1;
                }
            }
        }
    }
    0
}

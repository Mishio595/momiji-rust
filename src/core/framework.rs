use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::Owner;
use core::utils::check_rank;
use modules::commands::{
    admins,
    general,
    mods,
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
                .lacking_role(HelpBehaviour::Strike)
                .lacking_permissions(HelpBehaviour::Strike)
                .wrong_channel(HelpBehaviour::Strike)
                .embed_success_colour(*colours::MAIN)
                .embed_error_colour(*colours::RED))
            .bucket("weather", 30, DAY as i64, 1000)
            .group("Miscellaneous",         |_| general::init_misc())
            .group("Tags",                  |_| general::init_tags())
            .group("NSFW",                  |_| general::init_nsfw())
            .group("Self Roles",            |_| general::init_roles())
            .group("Role Management",       |_| mods::init_roles())
            .group("Mod Info",              |_| mods::init_info())
            .group("Mute",                  |_| mods::init_mute())
            .group("Hackbans",              |_| mods::init_hackbans())
            .group("Notes",                 |_| mods::init_notes())
            .group("Watchlist",             |_| mods::init_watchlist())
            .group("Management",            |_| admins::init_management())
            .group("Ignore Channels",       |_| admins::init_ignore())
            .group("Premium",               |_| admins::init_premium())
            .group("Tests",                 |_| admins::init_tests())
            .group("Self Role Management",  |_| admins::init_roles())
            .group("Config",                |_| admins::init_config())
            .group("Owner Only",            |_| owners::init())
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

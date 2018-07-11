use serenity::framework::{
    StandardFramework,
    standard::{help_commands, HelpBehaviour},
};
use serenity::model::id::{UserId, RoleId};
use modules::commands::*;
use core::model::DB;
use std::collections::HashSet;
use core::consts::*;
use chrono::Utc;

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
            .dynamic_prefix(|ctx, message|
                if message.is_private() {
                    Some(String::new())
                } else {
                    let mut data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let settings = db.get_guild(message.guild_id.unwrap().0 as i64).unwrap();
                    Some(settings.prefix)
                }
            ))
        .before(|ctx, message, command_name| {
            println!("Got command {} by user {}",
                command_name,
                message.author.name);
            if let false = message.is_private() {
                let data = ctx.data.lock();
                let db = data.get::<DB>().expect("Failed to get DB").lock();
                let guild_id = message.guild_id.unwrap();
                let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                return !guild_data.ignored_channels.contains(&(message.channel_id.0 as i64));
            }
            true
        })
        .after(|ctx, message, cmd_name, error| {
            if let Err(why) = error {
                ERROR_LOG.send_message(|m| m
                    .embed(|e| e
                        .description(format!("{:?}", why))
                        .field("Message", format!("{}", message.id.0), true)
                        .field("Channel", format!("{}", message.channel_id.0), true)
                        .timestamp(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string())
                        .colour(Colours::Red.val())
                )).expect("Failed to send message");
            }
        })
        .customised_help(help_commands::with_embeds, |c| c
            .suggestion_text("Couldn't find that command, but I did find a similar one. `{}`")
            .no_help_available_text("No help is available on this command.")
            .usage_label("Usage")
            .usage_sample_label("Example")
            //.ungrouped_label("Ungrouped")
            //.grouped_label("")
            .aliases_label("Aliases")
            .guild_only_text("Guild only")
            .dm_only_text("DM only")
            .dm_and_guilds_text("DM or Guild")
            //.available_text("")
            .command_not_found_text("Command not found.")
            //.individual_command_tip("")
            //.group_prefix("Prefix")
            .lacking_role(HelpBehaviour::Hide)
            .lacking_permissions(HelpBehaviour::Hide)
            .wrong_channel(HelpBehaviour::Strike)
            .striked_commands_tip(Some(String::from("")))
            //.striked_commands_tip_in_guild()
            //.striked_commands_tip_in_direct_message()
            .embed_success_colour(Colours::Main.val())
            .embed_error_colour(Colours::Red.val()))
        .group("Miscellaneous", |g| g
            .help_available(true)
            .command("ping", |c| c
                .cmd(ping)
                .desc("Make sure the bot is alive.")
                .usage(""))
            .command("prefix", |c| c
                .cmd(prefix)
                .desc("Echoes the prefix of the current guild.")
                .usage("")
                .guild_only(true)
                .known_as("pre"))
            .command("roll", |c| c
                .cmd(roll)
                .desc("Roll some dice. Defaults to 6-sided.")
                .usage("<Nd>[X]")
                .example("2d10")
                .min_args(1))
            .command("now", |c| c
                .cmd(now)
                .desc("Current time. Optionally provide an amount of hours to offset by.")
                .usage("[hour]")
                .example("-5")
                .known_as("time"))
            .command("cat", |c| c
                .cmd(cat)
                .desc("Random cat photo or gif."))
            .command("dog", |c| c
                .cmd(dog)
                .desc("Random dog photo or gif."))
            .command("joke", |c| c
                .cmd(dad_joke)
                .desc("Dad pun, now in discord."))
            .command("urban", |c| c
                .cmd(urban)
                .desc("Look something up on UrbanDictionary.")
                .usage(r#"<"term"> [count]"#)
                .example(r#""boku no pico" 5"#)
                .batch_known_as(vec!["ud", "urbandict"])
                .min_args(1))
            .command("remind", |c| c
                .cmd(remind)
                .desc("Set a reminder. The reminder is sent as a DM, so make sure your privacy settings allow it.")
                .usage("<reminder text> </t time_resolvable>")
                .example("do the thing /t 1 day 10 min 25 s")))
        .group("NSFW", |g| g
            .help_available(true)
            .command("e621", |c| c
                .cmd(e621)
                .desc("Random image from e621.net. Provide your own tags like you would on the website.")
                .usage("[tags]")
                .example("male/male dragon double_penetration")
                .check(|_,message,_,_| {
                    if message.channel_id.get().unwrap().is_nsfw() {
                        true
                    } else {
                        message.channel_id.say("Command only available in NSFW channels.").expect("Failed to send message");
                        false
                    }})
                .known_as("furry")))
        .group("Information", |g| g
            .help_available(true)
            .command("botinfo", |c| c
                .cmd(bot_info)
                .desc("Information about the bot.")
                .usage("")
                .batch_known_as(vec!["bi", "binfo"]))
            .command("serverinfo", |c| c
                .cmd(server_info)
                .desc("Information about the current server (guild).")
                .usage("")
                .guild_only(true)
                .batch_known_as(vec!["si", "sinfo"]))
            .command("userinfo", |c| c
                .cmd(user_info)
                .desc("Information about a user. Defaults to the author of the command.")
                .usage("[user_resolvable]")
                .example("@Adelyn")
                .guild_only(true)
                .batch_known_as(vec!["ui", "uinfo"]))
            .command("roleinfo", |c| c
                .cmd(role_info)
                .desc("Information about a role.")
                .usage("<role_resolvable>")
                .example("@example role")
                .guild_only(true)
                .batch_known_as(vec!["ri", "rinfo"])
                .min_args(1)))
        .group("Self Roles", |g| g
            .help_available(true)
            .guild_only(true)
            .command("asr", |c| c
                .cmd(asr)
                .desc("Add self roles.")
                .usage("<role_resolvables as CSV>")
                .example("red, green")
                .min_args(1)
                .known_as("role"))
            .command("rsr", |c| c
                .cmd(rsr)
                .desc("Remove self role(s).")
                .usage("<role_resolvables as CSV>")
                .example("red, green")
                .min_args(1)
                .known_as("derole"))
            .command("lsr", |c| c
                .cmd(lsr)
                .desc("List self roles")
                .usage("")
                .known_as("roles")))
        // Needs check for rank
        .group("Mod+", |g| g
            .guild_only(true)
            .help_available(true)
            .command("modinfo", |c| c
                .cmd(mod_info)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("View some useful information on a user.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .batch_known_as(vec!["mi", "minfo"])
                .min_args(1)))
        .group("Role Management", |g| g
            .help_available(true)
            .guild_only(true)
            .command("ar", |c| c
                .cmd(ar)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("Add role(s) to a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("addrole"))
            .command("rr", |c| c
                .cmd(rr)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("Remove role(s) from a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("removerole"))
            .command("rolecolour", |c| c
                .cmd(role_colour)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("Change the colour of a role.")
                .usage("<role_resolvable> <colour>")
                .example("418130449089691658 00ff00")
                .batch_known_as(vec!["rc", "rolecolor"])
                .min_args(2)))
        .group("Notes", |g| g
            .prefix("note")
            .guild_only(true)
            .help_available(true)
            .command("add", |c| c
                .cmd(note_add)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("Add a note to a user.")
                .usage("<user_resolvable> <note>")
                .example("@Adelyn test note")
                .min_args(2))
            .command("del", |c| c
                .cmd(note_del)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("Delete a note from a user.")
                .usage("<user_resolvable> <index>")
                .example("@Adelyn 3")
                .min_args(2))
            .command("list", |c| c
                .cmd(note_list)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.mod_roles, member.roles)
                })
                .desc("List all notes for a user.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1)))
        // Needs check for rank
        .group("Admin+", |g| g
            .guild_only(true)
            .help_available(true)
            .command("ignore", |c| c
                .cmd(ignore)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Tell the bot to ignore a channel, or being listening to one that was previously ignored.")
                .usage("<channel_resolvable>")
                .example("#general"))
            .command("prune", |c| c
                .cmd(prune)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Bulk delete messages.")
                .usage("<count> [filter]")
                .example("20 bot")))
        .group("Watchlist", |g| g
            .prefix("wl")
            .guild_only(true)
            .help_available(true)
            .command("add", |c| c
                .cmd(watchlist_add)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Add a user to the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1))
            .command("del", |c| c
                .cmd(watchlist_del)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Remove a user from the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1))
            .command("list", |c| c
                .cmd(watchlist_list)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("List users on the watchlist.")
                .usage("")))
        .group("Self Role Management", |g| g
            .help_available(true)
            .guild_only(true)
            .command("csr", |c| c
                .cmd(csr)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Create a self role from a discord role. Also optionally takes a category and/or aliases.")
                .usage("<role_resolvable> [/c category] [/a aliases as CSV]")
                .example("NSFW /c Opt-in /a porn, lewd")
                .min_args(1))
            .command("dsr", |c| c
                .cmd(dsr)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Delete a self role.")
                .usage("<role_resolvable>")
                .example("NSFW")
                .min_args(1)))
        .group("Config", |g| g
            .help_available(true)
            .guild_only(true)
            .prefix("config")
            .command("raw", |c| c
                .cmd(config_raw)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Lists current configuration as raw output."))
            .command("list", |c| c
                .cmd(config_list)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Lists current configuration."))
            .command("prefix", |c| c
                .cmd(config_prefix)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Set a new prefix")
                .usage("<prefix>")
                .example("!!"))
            .command("autorole", |c| c
                .cmd(config_autorole)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Change autorole settings. A role must be provided for add or remove.")
                .usage("<add|remove|enable|disable> <role_resolvable|_>")
                .example("add member"))
            .command("admin", |c| c
                .cmd(config_admin)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Add or remove roles from the bot's admin list.")
                .usage("<add|remove> <role_resolvable>")
                .example("add admin"))
            .command("mod", |c| c
                .cmd(config_mod)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Add or remove roles from the bot's admin list.")
                .usage("<add|remove> <role_resolvable>")
                .example("add staff"))
            .command("audit", |c| c
                .cmd(config_audit)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Change audit log settings. A channel must be provided for channel.")
                .usage("<enable|disable|channel> <channel_resolvable>")
                .example("channel #audit-logs"))
            .command("modlog", |c| c
                .cmd(config_modlog)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Change moderation log settings. A channel must be provided for channel.")
                .usage("<enable|disable|channel> <channel_resolvable>")
                .example("channel #mod-logs"))
            .command("welcome", |c| c
                .cmd(config_welcome)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Change welcome message settings. A channel must be provided for channel, while a message resolvable must be provided for message.")
                .usage("<enable|disable|channel|message> <channel_resolvable|message_resolvable>")
                .example("message Welcome to {guild}, {user}!"))
            .command("introduction", |c| c
                .cmd(config_introduction)
                .check(|ctx, message, _, _| {
                    let data = ctx.data.lock();
                    let db = data.get::<DB>().expect("Failed to get DB").lock();
                    let guild_id = message.guild_id.unwrap();
                    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
                    let member = guild_id.member(message.author.id.clone()).unwrap();
                    check_rank(guild_data.admin_roles, member.roles)
                })
                .desc("Change introduction message settings. A channel must be provided for channel, while a message resolvable must be provided for message. This is a premium only feature related to the Register command.")
                .usage("<enable|disable|channel|message> <channel_resolvable|message_resolvable>")
                .example("message Hey there {user}, mind introducting yourself?")))
        .group("Owner Only", |g| g
            .owners_only(true)
            .help_available(false)
            .command("log", |c| c
                .cmd(log)))
}

fn check_rank(roles: Vec<i64>, member: Vec<RoleId>) -> bool {
    for role in roles.iter() {
        if member.contains(&RoleId(*role as u64)) {
            return true;
        }
    }
    false
}

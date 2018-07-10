use serenity::framework::{StandardFramework, standard::help_commands};
use serenity::model::id::UserId;
use modules::commands::*;
use core::model::DB;
use std::collections::HashSet;

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
            .dynamic_prefix(|ctx, msg|
                if msg.is_private() {
                    Some(String::new())
                } else {
                    let mut data = ctx.data.lock();
                    let db = data.get::<DB>().unwrap().lock();
                    let settings = db.get_guild(msg.guild_id.unwrap().0 as i64).unwrap();
                    Some(settings.prefix)
                }
            ))
        .before(|ctx, message, command_name| {
            println!("Got command {} by user {}",
                command_name,
                message.author.name);
            let data = ctx.data.lock();
            let db = data.get::<DB>().unwrap().lock();
            let guild_id = message.guild_id.unwrap();
            let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
            !guild_data.ignored_channels.contains(&(message.channel_id.0 as i64))
        })
        .customised_help(help_commands::with_embeds, |c| c)
        .command("ping", |c| c
            .cmd(ping)
            .desc("Make sure the bot is alive.")
            .usage("")
            .help_available(true))
        .command("prefix", |c| c
            .cmd(prefix)
            .desc("Echoes the prefix of the current guild.")
            .usage("")
            .guild_only(true)
            .help_available(true)
            .known_as("pre"))
        .command("botinfo", |c| c
            .cmd(bot_info)
            .desc("Information about the bot.")
            .usage("")
            .help_available(true)
            .batch_known_as(vec!["bi", "binfo"]))
        .command("serverinfo", |c| c
            .cmd(server_info)
            .desc("Information about the current server (guild).")
            .usage("")
            .guild_only(true)
            .help_available(true)
            .batch_known_as(vec!["si", "sinfo"]))
        .command("userinfo", |c| c
            .cmd(user_info)
            .desc("Information about a user. Defaults to the author of the command.")
            .usage("[user_resolvable]")
            .example("@Adelyn")
            .guild_only(true)
            .help_available(true)
            .batch_known_as(vec!["ui", "uinfo"]))
        .command("roleinfo", |c| c
            .cmd(role_info)
            .desc("Information about a role.")
            .usage("<role_resolvable>")
            .example("@example role")
            .guild_only(true)
            .help_available(true)
            .batch_known_as(vec!["ri", "rinfo"])
            .min_args(1))
        .command("roll", |c| c
            .cmd(roll)
            .desc("Roll some dice. Defaults to 6-sided.")
            .usage("<Nd>[X]")
            .example("2d10")
            .help_available(true)
            .min_args(1))
        .command("now", |c| c
            .cmd(now)
            .desc("Current time. Optionally provide an amount of hours to offset by.")
            .usage("[hour]")
            .example("-5")
            .help_available(true)
            .known_as("time"))
        .command("cat", |c| c
            .cmd(cat)
            .desc("Random cat photo or gif.")
            .help_available(true))
        .command("dog", |c| c
            .cmd(dog)
            .desc("Random dog photo or gif.")
            .help_available(true))
        .command("joke", |c| c
            .cmd(dad_joke)
            .desc("Dad pun, now in discord.")
            .help_available(true))
        .command("urban", |c| c
            .cmd(urban)
            .desc("Look something up on UrbanDictionary.")
            .usage(r#"<"term"> [count]"#)
            .example(r#""boku no pico" 5"#)
            .help_available(true)
            .batch_known_as(vec!["ud", "urbandict"])
            .min_args(1))
        .command("e621", |c| c
            .cmd(e621)
            .desc("Random image from e621.net. Provide your own tags like you would on the website.")
            .usage("[tags]")
            .example("male/male dragon double_penetration")
            .help_available(true)
            .known_as("furry")
            .check(|_,msg,_,_| {
                if msg.channel_id.get().unwrap().is_nsfw() {
                    true
                } else {
                    msg.channel_id.say("Command only available in NSFW channels.").expect("Failed to send message");
                    false
                }}))
        .group("Notes", |g| g
            .prefix("note")
            .guild_only(true)
            .help_available(true)
            .command("add", |c| c
                .cmd(note_add)
                .desc("Add a note to a user.")
                .usage("<user_resolvable> <note>")
                .example("@Adelyn test note")
                .min_args(2))
            .command("del", |c| c
                .cmd(note_del)
                .desc("Delete a note from a user.")
                .usage("<user_resolvable> <index>")
                .example("@Adelyn 3")
                .min_args(2))
            .command("list", |c| c
                .cmd(note_list)
                .desc("List all notes for a user.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1)))
        .group("Watchlist", |g| g
            .prefix("wl")
            .guild_only(true)
            .help_available(true)
            .command("add", |c| c
                .cmd(watchlist_add)
                .desc("Add a user to the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1))
            .command("del", |c| c
                .cmd(watchlist_del)
                .desc("Remove a user from the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .min_args(1))
            .command("list", |c| c
                .cmd(watchlist_list)
                .desc("List users on the watchlist.")
                .usage("")))
        .command("modinfo", |c| c
            .cmd(mod_info)
            .desc("View some useful information on a user.")
            .usage("<user_resolvable>")
            .example("@Adelyn")
            .guild_only(true)
            .help_available(true)
            .batch_known_as(vec!["mi", "minfo"])
            .min_args(1))
        .group("Role Management", |g| g
            .help_available(true)
            .guild_only(true)
            .command("ar", |c| c
                .cmd(ar)
                .desc("Add role(s) to a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("addrole"))
            .command("rr", |c| c
                .cmd(rr)
                .desc("Remove role(s) from a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("removerole"))
            .command("rolecolour", |c| c
                .cmd(role_colour)
                .desc("Change the colour of a role.")
                .usage("<role_resolvable> <colour>")
                .example("418130449089691658 00ff00")
                .batch_known_as(vec!["rc", "rolecolor"])
                .min_args(2)))
        .group("Self Role Management", |g| g
            .help_available(true)
            .guild_only(true)
            .command("csr", |c| c
                .cmd(csr)
                .desc("Create a self role from a discord role. Also optionally takes a category and/or aliases.")
                .usage("<role_resolvable> [/c category] [/a aliases as CSV]")
                .example("NSFW /c Opt-in /a porn, lewd")
                .min_args(1))
            .command("dsr", |c| c
                .cmd(dsr)
                .desc("Delete a self role.")
                .usage("<role_resolvable>")
                .example("NSFW")
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
        .command("remind", |c| c
            .cmd(remind)
            .desc("Set a reminder.")
            .usage("<reminder text> </t time_resolvable>")
            .example("do the thing /t 1 day 10 min 25 s")
            .help_available(true))
        .command("ignore", |c| c
            .cmd(ignore)
            .desc("Tell the bot to ignore a channel, or being listening to one that was previously ignored.")
            .usage("<channel_resolvable>")
            .example("#general")
            .help_available(true)
            .guild_only(true))
        .group("Config", |g| g
            .help_available(true)
            .guild_only(true)
            .prefix("config")
            .command("list", |c| c
                .cmd(config_list)
                .desc("Lists current configuration."))
            .command("prefix", |c| c
                .cmd(config_prefix)
                .desc("Set a new prefix")
                .usage("<prefix>")
                .example("!!"))
            .command("autorole", |c| c
                .cmd(config_autorole)
                .desc("Change autorole settings. A role must be provided for add or remove.")
                .usage("<add|remove|enable|disable> <role_resolvable|_>")
                .example("add member"))
            .command("admin", |c| c
                .cmd(config_admin)
                .desc("Add or remove roles from the bot's admin list.")
                .usage("<add|remove> <role_resolvable>")
                .example("add admin"))
            .command("mod", |c| c
                .cmd(config_mod)
                .desc("Add or remove roles from the bot's admin list.")
                .usage("<add|remove> <role_resolvable>")
                .example("add staff"))
            .command("audit", |c| c
                .cmd(config_audit)
                .desc("Change audit log settings. A channel must be provided for channel.")
                .usage("<enable|disable|channel> <channel_resolvable>")
                .example("channel #audit-logs"))
            .command("modlog", |c| c
                .cmd(config_modlog)
                .desc("Change moderation log settings. A channel must be provided for channel.")
                .usage("<enable|disable|channel> <channel_resolvable>")
                .example("channel #mod-logs"))
            .command("welcome", |c| c
                .cmd(config_welcome)
                .desc("Change welcome message settings. A channel must be provided for channel, while a message resolvable must be provided for message.")
                .usage("<enable|disable|channel|message> <channel_resolvable|message_resolvable>")
                .example("message Welcome to {guild}, {user}!"))
            .command("introduction", |c| c
                .cmd(config_introduction)
                .desc("Change introduction message settings. A channel must be provided for channel, while a message resolvable must be provided for message. This is a premium only feature related to the Register command.")
                .usage("<enable|disable|channel|message> <channel_resolvable|message_resolvable>")
                .example("message Hey there {user}, mind introducting yourself?")))
        .command("prune", |c| c
            .cmd(prune)
            .desc("Bulk delete messages.")
            .usage("<count> [filter]")
            .example("20 bot")
            .guild_only(true)
            .help_available(true))
}

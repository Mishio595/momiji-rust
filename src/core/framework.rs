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
                    let settings = db.get_guild(msg.guild_id().unwrap().0 as i64).unwrap();
                    Some(settings.prefix)
                }
            ))
        .before(|_ctx, msg, command_name| {
            println!("Got command {} by user {}",
                command_name,
                msg.author.name);
            true
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
        .command("rolecolour", |c| c
            .cmd(role_colour)
            .desc("Change the colour of a role.")
            .usage("<role_resolvable> <colour>")
            .example("418130449089691658 00ff00")
            .guild_only(true)
            .help_available(true)
            .batch_known_as(vec!["rc", "rolecolor"])
            .min_args(2))
        .group("Self Role Management", |g| g
            .help_available(true)
            .guild_only(true)
            .command("csr", |c| c
                .cmd(csr)
                .min_args(1))
            .command("dsr", |c| c
                .cmd(dsr)
                .min_args(1)))
        .group("Self Roles", |g| g
            .help_available(true)
            .guild_only(true)
            .command("asr", |c| c
                .cmd(asr)
                .min_args(1)
                .known_as("role"))
            .command("rsr", |c| c
                .cmd(rsr)
                .min_args(1)
                .known_as("derole"))
            .command("lsr", |c| c
                .cmd(lsr)
                .known_as("roles")))
        .command("remind", |c| c
            .cmd(remind)
            .help_available(true))
}

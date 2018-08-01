use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::check_rank;
use modules::commands::*;
use serenity::framework::{
    StandardFramework,
    standard::{
        help_commands,
        HelpBehaviour
    }
};
use serenity::model::channel::Channel;
use serenity::model::id::{
    GuildId,
    UserId
};
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
            .prefix("m!")
            .dynamic_prefix(|_ctx, message| {
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
        .before(|_ctx, message, _command_name| {
            info!("{}", _command_name);
            if let false = message.is_private() {
                let guild_id = message.guild_id.unwrap_or(GuildId(0));
                if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                    return !guild_data.ignored_channels.contains(&(message.channel_id.0 as i64));
                }
            }
            true
        })
        .after(|_ctx, message, cmd_name, error| {
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
                    Channel::Private(_) => format!("{}", ch.id().0),
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
                check_error!(ERROR_LOG.send_message(|m| m
                    .embed(|e| e
                        .description(format!("{:?}", why))
                        .field("Message", format!("{}", message.id.0), true)
                        .field("Channel", format!("{}", message.channel_id.0), true)
                        .field("Command", cmd_name, true)
                        .field("Message Content", message.content_safe(), false)
                        .timestamp(now!())
                        .colour(*colours::RED)
                )));
            }
        })
        .customised_help(help_commands::plain, |c| c
            //.suggestion_text("Couldn't find that command, but I did find a similar one. `{}`")
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
            //.striked_commands_tip()
            //.striked_commands_tip_in_guild()
            //.striked_commands_tip_in_direct_message()
            .embed_success_colour(*colours::MAIN)
            .embed_error_colour(*colours::RED))
        .group("For Everyone", |g| g
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
                .example("2d10"))
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
                .desc("Set a reminder. The reminder is sent to whatever channel it originated in.")
                .usage("<reminder text> </t time_resolvable>")
                .example("do the thing /t 1 day 10 min 25 s"))
            // TODO write bucket, 1000 api calls per day
            .command("weather", |c| c
                .cmd(weather)
                .desc("Check on the current weather.")
                .usage("<city name>")
                .example("london"))
            .command("anime", |c| c
                .cmd(anime_search)
                .desc("Search for an anime using kitsu.io")
                .usage("<anime title>")
                .example("darling in the franxx"))
            .command("manga", |c| c
                .cmd(manga_search)
                .desc("Search for a manga using kitsu.io")
                .usage("<anime title>")
                .example("tsubasa"))
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
                .min_args(1))
            .command("tags", |c| c
                .cmd(tag_list)
                .desc("Alias to `tag list`"))
            .command("xp", |c| c
                .cmd(xp)
                .help_available(false)
                .desc("Check your current xp")))
        .group("Tags", |g| g
            .help_available(true)
            .guild_only(true)
            .prefix("tag")
            .default_cmd(tag_single)
            .command("show", |c| c
                .cmd(tag_single)
                .desc("View a tag.")
                .usage("<tag name>")
                .example("foobar"))
            .command("add", |c| c
                .cmd(tag_add)
                .desc("Create a new tag.")
                .usage("<tag name, quoted> <tag value>")
                .example(r#""my new tag" look, I made a tag!"#))
            .command("del", |c| c
                .cmd(tag_del)
                .desc("Delete a tag.")
                .usage("<tag name>")
                .example("foobar"))
            .command("edit", |c| c
                .cmd(tag_edit)
                .desc("Edit a tag. Only works if you are the author.")
                .usage("<tag name, quoted> <new value>")
                .example(r#""my edited tag" I had to edit this tag"#))
            .command("list", |c| c
                .cmd(tag_list)
                .desc("List all tags on the server.")))
        .group("NSFW", |g| g
            .help_available(true)
            .check(|_,message,_,_| {
                if let Ok(channel) = message.channel_id.get() {
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
                .cmd(e621)
                .desc("Random image from e621.net. Provide your own tags like you would on the website.")
                .usage("[tags]")
                .example("male/male dragon double_penetration")
                .known_as("furry")))
        .group("Self Roles", |g| g
            .help_available(true)
            .guild_only(true)
            .command("role", |c| c
                .cmd(asr)
                .desc("Add roles to yourself provided they are on the self role list.")
                .usage("<role_resolvables as CSV>")
                .example("red, green")
                .min_args(1)
                .batch_known_as(vec!["addselfrole", "asr"]))
            .command("derole", |c| c
                .cmd(rsr)
                .desc("Remove roles from yourself provided they are on the self role list.")
                .usage("<role_resolvables as CSV>")
                .example("red, green")
                .min_args(1)
                .batch_known_as(vec!["removeselfrole", "rsr"]))
            .command("roles", |c| c
                .cmd(lsr)
                .desc("List all the self roles for the current server.")
                .usage("")
                .batch_known_as(vec!["listselfroles", "lsr"])))
        .group("For Moderators", |g| g
            .guild_only(true)
            .help_available(true)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.mod_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("mute", |c| c
                .cmd(mute)
                .desc("Mute a user. Can provide an optional reason and time.")
                .usage("<user_resolvable> [/t time] [/r reason]")
                .example("@Adelyn /t 1day /r spam"))
            .command("unmute", |c| c
                .cmd(unmute)
                .desc("Unmute a user.")
                .usage("<user_resolvable>")
                .example("@Adelyn"))
            .command("modinfo", |c| c
                .cmd(mod_info)
                .desc("View some useful information on a user.")
                .usage("<user_resolvable>")
                .example("@Adelyn")
                .batch_known_as(vec!["mi", "minfo"])
                .min_args(1))
            .command("register", |c| c
                .cmd(register)
                .desc("A premium command that adds roles to a user (from the self roles list only), and depending on the settings for the command, will apply either a member role or a cooldown role with a timer. When the timer ends, cooldown is removed and member is added. In order for the switch to occur automatically, this command must be used. See the premium commands for more information on configuring this command.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn gamer, techie")
                .known_as("reg"))
            .command("addrole", |c| c
                .cmd(ar)
                .desc("Add role(s) to a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("ar"))
            .command("removerole", |c| c
                .cmd(rr)
                .desc("Remove role(s) from a user.")
                .usage("<user_resolvable> <role_resolvables as CSV>")
                .example("@Adelyn red, green")
                .min_args(2)
                .known_as("rr"))
            .command("rolecolour", |c| c
                .cmd(role_colour)
                .desc("Change the colour of a role.")
                .usage("<role_resolvable> <colour>")
                .example("418130449089691658 00ff00")
                .batch_known_as(vec!["rc", "rolecolor"])
                .min_args(2)))
        .group("Notes (Mod+)", |g| g
            .prefix("note")
            .guild_only(true)
            .help_available(true)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.mod_roles, member.roles);
                        }
                    }
                }
                false
            })
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
        .group("Watchlist (Mod+)", |g| g
            .prefixes(vec!["watchlist", "wl"])
            .guild_only(true)
            .help_available(true)
            .default_cmd(watchlist_list)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.mod_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("add", |c| c
                .cmd(watchlist_add)
                .desc("Add a user to the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn"))
            .command("del", |c| c
                .cmd(watchlist_del)
                .desc("Remove a user from the watchlist.")
                .usage("<user_resolvable>")
                .example("@Adelyn"))
            .command("list", |c| c
                .cmd(watchlist_list)
                .desc("List users on the watchlist.")
                .usage("")))
        .group("For Admins", |g| g
            .guild_only(true)
            .help_available(true)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.admin_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("setup", |c| c
                .cmd(setup_mute)
                .desc("Sets up mute for the server. This command requires the Manage Channels and Manage Roles permissions. It creates the Muted role if it doesn't exist, then iterates through every channel and category to disable Send Messages, Speak, and Add Reactions.")
                .usage(""))
            .command("ignore", |c| c
                .cmd(ignore)
                .desc("Tell the bot to ignore a channel, or being listening to one that was previously ignored.")
                .usage("<channel_resolvable>")
                .example("#general"))
            .command("prune", |c| c
                .cmd(prune)
                .desc("Bulk delete messages. Filter is one of bot, attachment, !pin, mention, or a user_resolvable.\n`bot` will prune only messages from bots.\n`attachment` will prune only messages with attachments.\n`!pin` will prune all but pinned messages.\n`mention` will prune only messages that mention a user or everyone.\nMentioning a user will prune only that user's messages.")
                .usage("<count> [filter]")
                .example("20 bot")))
        .group("Premium (Admin+)", |g| g
            .guild_only(true)
            .help_available(true)
            .prefixes(vec!["p", "premium", "prem"])
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.admin_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("register_member", |c| c
                .cmd(premium_reg_member)
                .batch_known_as(vec!["reg_m", "reg_member"]))
            .command("register_cooldown", |c| c
                .cmd(premium_reg_cooldown)
                .batch_known_as(vec!["reg_c", "reg_cooldown"]))
            .command("register_duration", |c| c
                .cmd(premium_reg_dur)
                .batch_known_as(vec!["reg_dur", "reg_duration"]))
            .command("register_roles", |c| c
                .cmd(premium_reg_restrict)
                .batch_known_as(vec!["reg_roles", "reg_restrict"])))
        .group("Tests (Admin+)", |g| g
            .guild_only(true)
            .help_available(true)
            .prefix("test")
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.admin_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("welcome", |c| c
                .cmd(test_welcome)
                .desc("Generates a welcome message to test your current setup.")))
        .group("Self Role Management (Admin+)", |g| g
            .help_available(true)
            .guild_only(true)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.admin_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("csr", |c| c
                .cmd(csr)
                .desc("Create a self role from a discord role. Also optionally takes a category and/or aliases.")
                .usage("<role_resolvable> [/c category] [/a aliases as CSV]")
                .example("NSFW /c Opt-in /a porn, lewd")
                .known_as("createselfrolesr"))
            .command("dsr", |c| c
                .cmd(dsr)
                .desc("Delete a self role.")
                .usage("<role_resolvable>")
                .example("NSFW")
                .known_as("deleteselfrolesr"))
            .command("esr", |c| c
                .cmd(esr)
                .desc("Edit a self role. Optionally takes a category and/or aliases. This operation is lazy and won't change anything you don't specify. Replace switch tells the bot to override aliases instead of append.")
                .usage("<role_resolvable> [/c category] [/a aliases as CSV] [/replace]")
                .example("NSFW /c Opt-in /a porn, lewd /replace")
                .known_as("editselfrolesr")))
        .group("Config (Admin+)", |g| g
            .help_available(true)
            .guild_only(true)
            .prefixes(vec!["config", "conf"])
            .default_cmd(config_list)
            .check(|ctx, message, _, _| {
                if let Some(guild_id) = message.guild_id {
                    if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
                        if let Ok(member) = guild_id.member(message.author.id.clone()) {
                            return check_rank(guild_data.admin_roles, member.roles);
                        }
                    }
                }
                false
            })
            .command("list", |c| c
                .cmd(config_list)
                .desc("Lists current configuration."))
            .command("raw", |c| c
                .cmd(config_raw)
                .desc("Lists current configuration as raw output."))
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
                .desc("Change welcome message settings.\nOption is one of enable, disable, channel, message, type and the respective values should be none, none, channel_resolvable, desired message.\nType designates if the message is plain or embed. Anything other than embed will result in plain.")
                .usage("<option> <value>")
                .example("message Welcome to {guild}, {user}!"))
            .command("introduction", |c| c
                .cmd(config_introduction)
                .desc("Change introduction message settings. This is exactly like welcome: `help config welcome` for more info. This is a premium only feature related to the Register command.")
                .usage("<option> <value>")
                .example("message Hey there {user}, mind introducting yourself?")))
        .group("Owner Only", |g| g
            .owners_only(true)
            .help_available(false)
            .command("op", |c| c
                .cmd(set_premium))
            .command("log", |c| c
                .cmd(log)))
}

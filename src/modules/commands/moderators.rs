use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::TC;
use core::utils::*;
use serenity::builder::CreateMessage;
use serenity::model::id::{
    RoleId,
    ChannelId,
    UserId
};
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct HackbanAdd;
impl Command for HackbanAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Adds a user to the hackban list. Users on this list will be banned on joining.".to_string()),
            usage: Some("<user_id> [reason]".to_string()),
            example: Some("242675474927583232 makes links for raiders".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        let user_id = UserId(args.single::<u64>()?);
        match hackbans.iter().find(|e| e.id as u64 == user_id.0) {
            Some(_) => { message.channel_id.say("User is already hackbanned.")?; },
            None => {
                let reason = args.single::<String>().ok();
                db.new_hackban(user_id.0 as i64, guild_id.0 as i64, reason.clone())?;
                message.channel_id.say(format!(
                    "Added {} to the hackban list{}",
                    user_id.0,
                    match reason {
                        Some(r) => format!(" with reason `{}`", r),
                        None => String::new(),
                    }
                ))?;
            }
        }
        Ok(())
    }
}

pub struct HackbanRemove;
impl Command for HackbanRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Removes a user from the hackban list.".to_string()),
            usage: Some("<user_id>".to_string()),
            example: Some("242675474927583232".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        let user_id = UserId(args.single::<u64>()?);
        match hackbans.iter().find(|e| e.id as u64 == user_id.0) {
            None => { message.channel_id.say("User isn't hackbanned.")?; },
            Some(_) => {
                db.del_hackban(user_id.0 as i64, guild_id.0 as i64)?;
                message.channel_id.say(format!(
                    "Removed {} from the hackban list",
                    user_id.0
                ))?;
            }
        }
        Ok(())
    }
}

pub struct HackbanList;
impl Command for HackbanList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Lets users on the hackban list along with their reasons, if provided.".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Hackbans")
                .description(
                    hackbans.iter().cloned().map(|e| format!(
                        "{}{}",
                        e.id,
                        format!(": `{}`", e.reason.unwrap_or(String::new()))
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
                )
                .colour(*colours::MAIN)
        ))?;
        Ok(())
    }
}

pub struct ModInfo;
impl Command for ModInfo {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("View some useful information on a user.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            aliases: vec!["mi", "minfo"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, _)) => {
                    let user = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    let cases = db.get_cases(user_id.0 as i64, guild_id.0 as i64)?;
                    let case_fmt = cases.iter().map(|c| format!("Type: {}\nModerator: {}\nTimestamp: {}", c.casetype, c.moderator, c.timestamp)).collect::<Vec<String>>().join("\n");
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Moderator info")
                            .field("Watchlist", { if user.watchlist { "Yes" } else { "No" } }, false)
                            .field("Cases", if case_fmt.is_empty() { "None" } else { case_fmt.as_str() }, false)
                    ))?;
                },
                None => { message.channel_id.say("I couldn't find that user.")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct Mute;
impl Command for Mute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Mute a user. Can provide an optional reason and time.".to_string()),
            usage: Some("<user_resolvable> [/t time] [/r reason]".to_string()),
            example: Some("@Adelyn /t 1day /r spam".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, ctx: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_lock) = message.guild() {
            let guild = {
                guild_lock.read().clone()
            };
            if let Some((_, mut member)) = parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
                let guild_data = db.get_guild(guild.id.0 as i64)?;
                if guild_data.mute_setup {
                    let switches = get_switches(args.rest().to_string());
                    let time = match switches.get("t") {
                        Some(s) => hrtime_to_seconds(s.clone()),
                        None => 0,
                    };
                    let reason = match switches.get("r") {
                        Some(s) => s.clone(),
                        None => String::new(),
                    };
                    if let Some(mute_role) = guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                        if member.roles.contains(&mute_role.id) {
                            message.channel_id.say("Member already muted.")?;
                        } else {
                            member.add_role(mute_role)?;
                            let user = {
                                member.user.read().clone()
                            };
                            let case = db.new_case(user.id.0 as i64, guild.id.0 as i64, "Mute".to_string(), Some(reason.clone()), message.author.id.0 as i64)?;
                            let mut fields = Vec::new();
                            fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                            fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                            if time != 0 {
                                let data = ctx.data.lock();
                                if let Some(tc_lock) = data.get::<TC>() {
                                    let tc = tc_lock.lock();
                                    tc.request(format!("UNMUTE||{}||{}||{}||{}||{}||{}",
                                        user.id.0,
                                        guild.id.0,
                                        mute_role.id.0,
                                        if guild_data.modlog && guild_data.modlog_channel > 0 {
                                            guild_data.modlog_channel
                                        } else { message.channel_id.0 as i64 },
                                        time,
                                        case.id), time as u64);
                                fields.push(("Duration", seconds_to_hrtime(time as usize), true));
                                } else {
                                    message.channel_id.say("Something went wrong with the timer.")?;
                                }
                            }
                            if !reason.is_empty() {
                                fields.push(("Reason", reason.to_string(), true));
                            }
                            let response = CreateMessage::default()
                                .embed(|e| e
                                    .title("Member Muted")
                                    .colour(*colours::BLUE)
                                    .fields(fields)
                                    .timestamp(now!()));

                            if guild_data.modlog && guild_data.modlog_channel > 0 {
                                let channel = ChannelId(guild_data.modlog_channel as u64);
                                channel.send_message(|_| response)?;
                            } else {
                                message.channel_id.send_message(|_| response)?;
                            }
                        }
                    } else { message.channel_id.say("No mute role")?; }
                } else {
                    message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
                }
            } else { message.channel_id.say("I couldn't find that user.")?; }
        } else { failed!(GUILD_FAIL); }
        Ok(())
    }
}

pub struct Unmute;
impl Command for Unmute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Unmute a user.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_lock) = message.guild() {
            let guild = {
                guild_lock.read().clone()
            };
            if let Some((_, mut member)) = parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
                let guild_data = db.get_guild(guild.id.0 as i64)?;
                if guild_data.mute_setup {
                    if let Some(mute_role) = guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                        let mut fields = Vec::new();
                        let user = {
                            member.user.read().clone()
                        };
                        fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                        fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                        let response = CreateMessage::default()
                            .embed(|e| e
                                .title("Member Unmuted")
                                .colour(*colours::BLUE)
                                .fields(fields)
                                .timestamp(now!()));

                        if member.roles.contains(&mute_role.id) {
                            member.remove_role(mute_role)?;
                            if guild_data.modlog && guild_data.modlog_channel > 0 {
                                let channel = ChannelId(guild_data.modlog_channel as u64);
                                channel.send_message(|_| response)?;
                            } else {
                                message.channel_id.send_message(|_| response)?;
                            }
                        } else {
                            message.channel_id.say("Member was not muted.")?;
                        }
                    } else { message.channel_id.say("No mute role")?; }
                } else {
                    message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
                }
            } else { message.channel_id.say("I couldn't find that user.")?; }
        } else { failed!(GUILD_FAIL); }
        Ok(())
    }
}

pub struct NoteAdd;
impl Command for NoteAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add a note to a user.".to_string()),
            usage: Some("<user_resolvable> <note>".to_string()),
            example: Some("@Adelyn test note".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user,_)) => {
                    let note = args.rest().to_string();
                    let data = db.new_note(user.0 as i64, guild_id.0 as i64, note, message.author.id.0 as i64)?;
                    message.channel_id.say(format!("Added note `{}`.", data.note))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct NoteRemove;
impl Command for NoteRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Delete a note from a user.".to_string()),
            usage: Some("<user_resolvable> <index>".to_string()),
            example: Some("@Adelyn 3".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user,_)) => {
                    let index = args.single::<i32>().unwrap_or(0);
                    let data = db.del_note(index, user.0 as i64, guild_id.0 as i64)?;
                    message.channel_id.say(format!("Deleted note `{}`.", data))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct NoteList;
impl Command for NoteList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List all notes for a user.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, member)) => {
                    let notes = db.get_notes(user_id.0 as i64, guild_id.0 as i64)?;
                    let notes_fmt = notes.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("\n\n");
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .colour(*colours::MAIN)
                            .title(format!("Notes for {}", member.display_name().into_owned()))
                            .description(notes_fmt)
                    ))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct Register;
impl Command for Register {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("A premium command that adds roles to a user (from the self roles list only), and depending on the settings for the command, will apply either a member role or a cooldown role with a timer. When the timer ends, cooldown is removed and member is added. In order for the switch to occur automatically, this command must be used. See the premium commands for more information on configuring this command.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            example: Some("@Adelyn gamer, techie".to_string()),
            aliases: vec!["reg"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, ctx: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let settings = db.get_premium(guild_id.0 as i64)?;
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            let roles = db.get_roles(guild_id.0 as i64)?;
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, mut member)) => {
                    let channel = if guild_data.modlog {
                        ChannelId(guild_data.modlog_channel as u64)
                    } else { message.channel_id };
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_add = Vec::new();
                    for r1 in list {
                        if let Some((r, _)) = parse_role(r1.clone(), guild_id) {
                            if settings.cooldown_restricted_roles.contains(&(r.0 as i64)) { continue; }
                            to_add.push(r);
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
                            if settings.cooldown_restricted_roles.contains(&(roles[i].id)) { continue; }
                            to_add.push(RoleId(roles[i].id as u64));
                        }
                    }
                    for (i, role_id) in to_add.clone().iter().enumerate() {
                        if member.roles.contains(role_id) {
                            to_add.remove(i);
                            continue;
                        }
                        if let Err(_) = member.add_role(*role_id) {
                            to_add.remove(i);
                        };
                    }
                    if let Some(role) = settings.register_cooldown_role {
                        member.add_role(RoleId(role as u64))?;
                        if let Some(member_role) = settings.register_member_role {
                            let data = ctx.data.lock();
                            if let Some(tc_lock) = data.get::<TC>() {
                                let tc = tc_lock.lock();
                                tc.request(format!("COOLDOWN||{}||{}||{}||{}",
                                    user_id.0,
                                    guild_id.0,
                                    member_role,
                                    role,
                                ), match settings.register_cooldown_duration {
                                    Some(dur) => dur as u64,
                                    None => DAY as u64,
                                });
                            }
                        }
                    } else if let Some(role) = settings.register_member_role {
                        member.add_role(RoleId(role as u64))?;
                    }
                    let desc = if !to_add.is_empty() {
                        to_add.iter().map(|r| match r.to_role_cached() {
                            Some(role) => role.name,
                            None => r.0.to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                    } else { String::new() };
                    channel.send_message(|m| m
                        .embed(|e| e
                            .title(format!(
                                "Registered {} with the following roles:",
                                member.user.read().tag()
                            ))
                            .description(desc)
                            .colour(member.colour().unwrap_or(*colours::MAIN))
                            .timestamp(now!())
                    ))?;
                    if guild_data.introduction && guild_data.introduction_channel>0 {
                        let channel = ChannelId(guild_data.introduction_channel as u64);
                        if guild_data.introduction_type == "embed" {
                            send_welcome_embed(guild_data.introduction_message, &member, channel)?;
                        } else {
                            channel.say(parse_welcome_items(guild_data.introduction_message, &member))?;
                        }
                    }
                },
                None => { message.channel_id.say("I couldn't find that user.")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct AddRole;
impl Command for AddRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add role(s) to a user.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            example: Some("@Adelyn red, green".to_string()),
            aliases: vec!["ar"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some((_, mut member)) = parse_user(args.single::<String>()?, guild_id) {
                let list = args.rest().split(",").map(|s| s.trim().to_string());
                let mut to_add = Vec::new();
                let mut failed = Vec::new();
                for r1 in list {
                    if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
                        to_add.push(s);
                    } else {
                        failed.push(format!("Could not locate {}", r1));
                    }
                }
                for (i, role_id) in to_add.clone().iter().enumerate() {
                    if member.roles.contains(role_id) {
                        to_add.remove(i);
                        failed.push(format!(
                            "You already have {}",
                            match role_id.to_role_cached() {
                                Some(role) => role.name,
                                None => role_id.0.to_string(),
                        }));
                    }
                    if let Err(_) = member.add_role(*role_id) {
                        to_add.remove(i);
                        failed.push(format!(
                            "Failed to add {}",
                            match role_id.to_role_cached() {
                                Some(role) => role.name,
                                None => role_id.0.to_string(),
                        }));
                    };
                }
                let mut fields = Vec::new();
                if !to_add.is_empty() {
                    fields.push(("Added Roles", to_add.iter()
                        .map(|r| match r.to_role_cached() {
                            Some(role) => role.name,
                            None => r.0.to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                        false));
                }
                if !failed.is_empty() {
                    fields.push(("Failed to Add", failed.join("\n"), false));
                }
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .title("Add Role Summary")
                        .fields(fields)
                        .colour(member.colour().unwrap_or(*colours::MAIN))
                ))?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct RemoveRole;
impl Command for RemoveRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Remove role(s) from a user.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            example: Some("@Adelyn red, green".to_string()),
            aliases: vec!["rr"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some((_, mut member)) = parse_user(args.single::<String>()?, guild_id) {
                let list = args.rest().split(",").map(|s| s.trim().to_string());
                let mut to_remove = Vec::new();
                let mut failed = Vec::new();
                for r1 in list {
                    if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
                        to_remove.push(s);
                    } else {
                        failed.push(format!("Could not locate {}", r1));
                    }
                }
                for (i, role_id) in to_remove.clone().iter().enumerate() {
                    if !member.roles.contains(role_id) {
                        to_remove.remove(i);
                        failed.push(format!(
                            "You don't have {}",
                            match role_id.to_role_cached() {
                                Some(role) => role.name,
                                None => role_id.0.to_string(),
                        }));
                    }
                    if let Err(_) = member.remove_role(*role_id) {
                        to_remove.remove(i);
                        failed.push(format!(
                            "Failed to remove {}",
                            match role_id.to_role_cached() {
                                Some(role) => role.name,
                                None => role_id.0.to_string(),
                        }));
                    };
                }
                let mut fields = Vec::new();
                if !to_remove.is_empty() {
                    fields.push(("Removed Roles", to_remove.iter()
                        .map(|r| match r.to_role_cached() {
                            Some(role) => role.name,
                            None => r.0.to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                        false));
                }
                if !failed.is_empty() {
                    fields.push(("Failed to Remove", failed.join("\n"), false));
                }
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .title("Remove Role Summary")
                        .fields(fields)
                        .colour(member.colour().unwrap_or(*colours::MAIN))
                ))?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct RoleColour;
impl Command for RoleColour {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Change the colour of a role.".to_string()),
            usage: Some("<role_resolvable> <colour>".to_string()),
            example: Some("418130449089691658 00ff00".to_string()),
            aliases: vec!["rc", "rolecolor"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_role(args.single_quoted::<String>().unwrap_or(String::new()), guild_id) {
                Some((_, mut role)) => {
                    let input = args.single::<String>()?;
                    let colour_as_hex = if input.starts_with("#") {
                        &input[1..]
                    } else { input.as_str() };
                    let colour = u64::from_str_radix(colour_as_hex, 16)?;
                    role.edit(|r| r.colour(colour))?;
                    message.channel_id.say(format!("Colour of `{}` changed to `#{:06X}`", role.name, colour))?;
                },
                None => { message.channel_id.say("I couldn't find that role")?; },
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct WatchlistAdd;
impl Command for WatchlistAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add a user to the watchlist.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, member)) => {
                    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    user_data.watchlist = true;
                    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data)?;
                    message.channel_id.say(format!("Set {} to watchlist status.", member.display_name().into_owned()))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct WatchlistRemove;
impl Command for WatchlistRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Remove a user from the watchlist.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, member)) => {
                    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    user_data.watchlist = false;
                    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data)?;
                    message.channel_id.say(format!("Unset {} from watchlist status.", member.display_name().into_owned()))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct WatchlistList;
impl Command for WatchlistList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List users on the watchlist.".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let users = db.get_users(guild_id.0 as i64)?;
            let user_map = users.iter()
                .filter(|e| e.watchlist)
                .map(|u| {
                    match UserId(u.id as u64).to_user() {
                        Ok(user) => user.tag(),
                        Err(_) => format!("<#{}>", u.id),
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Watchlist")
                    .description(user_map)
                    .colour(*colours::MAIN)
            ))?;
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

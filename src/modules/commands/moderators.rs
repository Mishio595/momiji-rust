use core::utils::*;
use core::model::TC;
use core::consts::{DAY, DB as db};
use core::colours;
use serenity::model::id::{RoleId, ChannelId, UserId};
use chrono::Utc;

// Rank 1

//TODO obtain data safely
command!(mod_info(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    let cases = db.get_cases(user_id.0 as i64, guild_id.0 as i64)?;
    let case_fmt = cases.iter().map(|c| format!("Type: {}\nModerator: {}\nTimestamp: {}", c.casetype, c.moderator, c.timestamp)).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Moderator info")
            .field("Watchlist", { if user.watchlist { "Yes" } else { "No" } }, false)
            .field("Cases", case_fmt, false)
    ))?;
});

command!(mute(ctx, message, args) {
    let data = ctx.data.lock();
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let (_, mut member) = parse_user(args.single::<String>().unwrap(), guild.id).unwrap();
    let temp = member.clone();
    let user = temp.user.read();
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
        let mute_role = guild.roles.values().find(|e| e.name.to_lowercase() == "muted").unwrap();
        if member.roles.contains(&mute_role.id) {
            message.channel_id.say("Member already muted.")?;
        } else {
            if let Ok(_) = member.add_role(mute_role) {
                let case = db.new_case(user.id.0 as i64, guild.id.0 as i64, "Mute".to_string(), message.author.id.0 as i64)?;
                let mut fields = Vec::new();
                fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                if time != 0 {
                    let tc = data.get::<TC>().unwrap().lock();
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
                }
                if !reason.is_empty() {
                    fields.push(("Reason", reason.to_string(), true));
                }
                if guild_data.modlog && guild_data.modlog_channel > 0 {
                    let channel = ChannelId(guild_data.modlog_channel as u64);
                    channel.send_message(|m| m
                        .embed(|e| e
                            .title("Member Muted")
                            .colour(*colours::BLUE)
                            .fields(fields)
                    ))?;
                } else {
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Member Muted")
                            .colour(*colours::BLUE)
                            .fields(fields)
                    ))?;
                }
            }
        }
    } else {
        message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
    }
});

command!(unmute(ctx, message, args) {
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let (_, mut member) = parse_user(args.single::<String>().unwrap(), guild.id).unwrap();
    let temp = member.clone();
    let user = temp.user.read();
    let guild_data = db.get_guild(guild.id.0 as i64)?;
    if guild_data.mute_setup {
        let mute_role = guild.roles.values().find(|e| e.name.to_lowercase() == "muted").unwrap();
        let mut fields = Vec::new();
        fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
        fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
        if member.roles.contains(&mute_role.id) {
            if let Ok(_) = member.remove_role(mute_role) {
                if guild_data.modlog && guild_data.modlog_channel > 0 {
                    let channel = ChannelId(guild_data.modlog_channel as u64);
                    channel.send_message(|m| m
                        .embed(|e| e
                            .title("Member Unmuted")
                            .colour(*colours::BLUE)
                            .fields(fields)
                    ))?;
                } else {
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Member Unmuted")
                            .colour(*colours::BLUE)
                            .fields(fields)
                    ))?;
                }
            }
        } else {
            message.channel_id.say("Member was not muted.")?;
        }
    } else {
        message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
    }
});

command!(note_add(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let note = String::from(args.rest());
    match db.new_note(user.0 as i64, message.guild_id.unwrap().0 as i64, note, message.author.id.0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Added note `{}`.", data.note))?; },
        Err(why) => { message.channel_id.say(format!("Failed to add note. Reason: {:?}", why))?; },
    }
});

command!(note_del(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let index = args.single::<i32>().unwrap_or(0);
    match db.del_note(index, user.0 as i64, message.guild_id.unwrap().0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Deleted note `{}`.", data))?; },
        Err(why) => { message.channel_id.say(format!("Failed to delete note. Reason: {:?}", why))?; },
    }
});

command!(note_list(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = user_id.get().unwrap();
    let notes = db.get_notes(user_id.0 as i64, message.guild_id.unwrap().0 as i64)?;
    let notes_fmt = notes.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(*colours::MAIN)
            .title(format!("Notes for {}", user.tag()))
            .description(notes_fmt)
    ))?;
});

command!(register(ctx, message, args) {
    let data = ctx.data.lock();
    let guild_id = message.guild_id.unwrap();
    if let Ok(settings) = db.get_premium(guild_id.0 as i64) {
        let tc = data.get::<TC>().expect("Failed to get TimerClient").lock();
        let guild_data = db.get_guild(guild_id.0 as i64)?;
        let roles = db.get_roles(guild_id.0 as i64)?;
        let (user_id, mut member) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
        let user = user_id.get().unwrap();
        let channel = if guild_data.modlog {
            ChannelId(guild_data.modlog_channel as u64)
        } else { message.channel_id };
        let list = args.rest().split(",").map(|s| s.trim().to_string());
        let mut to_add = Vec::new();
        for r1 in list {
            if let Some((r, _)) = parse_role(r1.clone(), guild_id) {
                if settings.cooldown_restricted_roles.contains(&(r.0 as i64)) { continue; }
                if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                    to_add.push(r);
                }
            } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
                to_add.push(RoleId(roles[i].id as u64));
            }
        }
        for (i, role_id) in to_add.clone().iter().enumerate() {
            if member.roles.contains(role_id) {
                to_add.remove(i);
            }
            if let Err(_) = member.add_role(*role_id) {
                to_add.remove(i);
            };
        }
        if let Some(role) = settings.register_cooldown_role {
            member.add_role(RoleId(role as u64))?;
            if let Some(member_role) = settings.register_member_role {
                tc.request(format!("COOLDOWN||{}||{}||{}||{}",
                    user.id.0,
                    guild_id.0,
                    member_role,
                    role,
                ), match settings.register_cooldown_duration {
                    Some(dur) => dur as u64,
                    None => DAY as u64,
                });
            }
        } else if let Some(role) = settings.register_member_role {
            member.add_role(RoleId(role as u64))?;
        }
        let desc = if !to_add.is_empty() {
            format!("{}", to_add.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n"))
        } else { String::new() };
        channel.send_message(|m| m
            .embed(|e| e
                .title(format!("Registered {} with the following roles:", user.tag()))
                .description(desc)
                .colour(member.colour().unwrap())
                .timestamp(now!())
        ))?;
    } else {
        message.channel_id.say("This guild does not have permissions to use this command.")?;
    }
});

command!(ar(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
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
            failed.push(format!("You already have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.add_role(*role_id) {
            to_add.remove(i);
            failed.push(format!("Failed to add {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_add.is_empty() {
        fields.push(("Added Roles", format!("{}", to_add.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Add", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Add Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap())
    ))?;
});

command!(rr(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
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
            failed.push(format!("You don't have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.remove_role(*role_id) {
            to_remove.remove(i);
            failed.push(format!("Failed to remove {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_remove.is_empty() {
        fields.push(("Removed Roles", format!("{}", to_remove.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Remove", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Remove Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap())
    ))?;
});

//TODO make not shit
command!(role_colour(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (_, mut role) = parse_role(args.single::<String>().unwrap(), guild_id).unwrap();
    let colour_as_hex = args.single::<String>().unwrap();
    let colour = u64::from_str_radix(colour_as_hex.as_str(), 16).unwrap();
    if let Ok(_) = role.edit(|r| r.colour(colour)) {
        message.channel_id.say("Colour changed successfully.")?;
    }
});

command!(watchlist_add(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user_id, _) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    user_data.watchlist = true;
    match db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data) {
        Ok(_) => { message.channel_id.say(format!("Set {} to watchlist status.", user_id.get().unwrap().tag()))?; },
        Err(_) => { message.channel_id.say("Failed to set watchlist status")?; },
    }
});

command!(watchlist_del(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    user_data.watchlist = false;
    match db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data) {
        Ok(_) => { message.channel_id.say(format!("Unset {} from watchlist status.", user_id.get().unwrap().tag()))?; },
        Err(_) => { message.channel_id.say("Failed to unset watchlist status")?; },
    }
});

command!(watchlist_list(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let users = db.get_users(guild_id.0 as i64).unwrap_or(Vec::new());
    let user_map = users.iter().filter(|e| e.watchlist).map(|u| UserId(u.id as u64).get().unwrap()).map(|u| u.tag()).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Watchlist")
            .description(user_map)
            .colour(*colours::MAIN)
    ))?;
});

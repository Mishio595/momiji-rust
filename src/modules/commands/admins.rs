use chrono::Utc;
use core::colours;
use core::consts::DB as db;
use core::utils::*;
use serenity::builder::GetMessages;
use serenity::model::Permissions;
use serenity::model::channel::{
    Message,
    PermissionOverwrite,
    PermissionOverwriteType
};
use serenity::model::id::*;
use serenity::prelude::*;
use std::str::FromStr;

// Rank 2

command!(config_raw(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
    message.channel_id.say(format!("{:?}", guild_data))?;
});

command!(config_list(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(*colours::MAIN)
            .description(format!("{}", guild_data))
    ))?;
});

command!(config_prefix(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let pre = args.single::<String>().unwrap();
    guild_data.prefix = pre;
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.say(format!("Set prefix to {}", guild.prefix))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to change prefix")?;
        },
    };
});

command!(config_autorole(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.autoroles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "enable" => {
            guild_data.autorole = true;
        },
        "disable" => {
            guild_data.autorole = false;
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Autorole Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.autorole) } else { val } ,
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_admin(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.admin_roles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(_) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Admin Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_mod(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.mod_roles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(_) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Mod Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_audit(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.audit = true;
        },
        "disable" => {
            guild_data.audit = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.audit_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "threshold" => {
            let th = val.parse::<i16>().unwrap();
            guild_data.audit_threshold = th;
            val = format!("{}", th);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Audit Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.audit) } else { val },
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_modlog(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.modlog = true;
        },
        "disable" => {
            guild_data.modlog = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.modlog_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Modlog Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.modlog) } else { val },
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_welcome(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.welcome = true;
        },
        "disable" => {
            guild_data.welcome = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.welcome_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "message" => {
            guild_data.welcome_message = val.to_string();
        },
        "type" => {
            guild_data.welcome_type = val.to_string();
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Welcome Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.welcome) } else { val },
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_introduction(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.introduction = true;
        },
        "disable" => {
            guild_data.introduction = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.introduction_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "message" => {
            guild_data.introduction_message = val.to_string();
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Introduction Summary")
                    .colour(*colours::MAIN)
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.introduction) } else { val },
                    ))
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

// TODO add hackban and ignore lists views
command!(hackban(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let user_id = match UserId::from_str(args.full()) {
        Ok(id) => id,
        Err(_) => {
            message.channel_id.say("Unable to resolve ID").expect("Failed to send message");
            panic!("Failed to resolve ID");
        },
    };
    if !guild_data.hackbans.contains(&(user_id.0 as i64)) {
        guild_data.hackbans.push(user_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(_) => {
                message.channel_id.say(format!("Added {} to the hackban list",
                    user_id.0
                ))?;
            },
            Err(_) =>{
                message.channel_id.say("Failed to add hackban")?;
            },
        };
    } else {
        guild_data.hackbans.retain(|e| *e != user_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(_) => {
                message.channel_id.say(format!("Removed {} from the hackban list",
                    user_id.0
                ))?;
            },
            Err(_) =>{
                message.channel_id.say("Failed to remove hackban")?;
            },
        };
    }
});

// TODO rewrite as group {add, remove, list}
command!(ignore(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let (channel_id, channel) = parse_channel(args.full().to_string(), guild_id).unwrap();
    if !guild_data.ignored_channels.contains(&(channel_id.0 as i64)) {
        guild_data.ignored_channels.push(channel_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(_) => {
                message.channel_id.say(format!("I will now ignore messages in {}",
                    channel.name
                ))?;
            },
            Err(_) =>{
                message.channel_id.say("Failed to add channel to ignore list")?;
            },
        };
    } else {
        guild_data.ignored_channels.retain(|e| *e != channel_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(_) => {
                message.channel_id.say(format!("I will no longer ignore messages in {}",
                    channel.name
                ))?;
            },
            Err(_) =>{
                message.channel_id.say("Failed to remove channel to ignore list")?;
            },
        };
    }
});

command!(csr(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let switches = get_switches(args.full().to_string());
    let rest = switches.get("rest").unwrap();
    let (role_id, _) = parse_role(rest.clone(), guild_id).expect("Failed to parse role");
    let category = match switches.get("c") {
        Some(s) => Some( s.clone()),
        None => None,
    };
    let aliases = match switches.get("a") {
        Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
        None => None,
    };
    match db.new_role(role_id.0 as i64, guild_id.0 as i64, category, aliases) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully added role {} to category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            ))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to add role: {:?}", why))?;
        },
    };
});

command!(dsr(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (role_id, _) = parse_role(args.single_quoted::<String>().unwrap(), guild_id).unwrap();
    match db.del_role(role_id.0 as i64, guild_id.0 as i64) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully deleted role {}", data))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to delete role: {:?}", why))?;
        },
    };
});

command!(esr(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let switches = get_switches(args.full().to_string());
    let rest = switches.get("rest").unwrap();
    let (role_id, _) = parse_role(rest.clone(), guild_id).expect("Failed to parse role");
    let category = match switches.get("c") {
        Some(s) => Some(s.clone()),
        None => None,
    };
    let aliases = match switches.get("a") {
        Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
        None => None,
    };
    let mut role = db.get_role(role_id.0 as i64, guild_id.0 as i64)?;
    if let Some(s) = category { role.category = s; }
    if let Some(mut a) = aliases {
        match switches.get("replace") {
            Some(_) => { role.aliases = a; },
            None => { role.aliases.append(&mut a); },
        }
    }
    match db.update_role(role_id.0 as i64, guild_id.0 as i64, role) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully update role {} in category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            ))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to edit role: {:?}", why))?;
        },
    };
});

command!(premium_reg_member(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    if let Ok(mut settings) = db.get_premium(guild_id.0 as i64) {
        if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
            settings.register_member_role = Some(role_id.0 as i64);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set member role to {}", role.name))?;
        }
    }
});

command!(premium_reg_cooldown(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    if let Ok(mut settings) = db.get_premium(guild_id.0 as i64) {
        if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
            settings.register_cooldown_role = Some(role_id.0 as i64);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set cooldown role to {}", role.name))?;
        }
    }
});

command!(premium_reg_dur(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    if let Ok(mut settings) = db.get_premium(guild_id.0 as i64) {
        if let Ok(dur) = args.full().parse::<String>() {
            let dur = hrtime_to_seconds(dur);
            settings.register_cooldown_duration = Some(dur as i32);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set duration of cooldown to {}", seconds_to_hrtime(dur as usize)))?;
        }
    }
});

command!(premium_reg_restrict(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let op = args.single::<String>().unwrap();
    let mut sec = "";
    let mut val = String::new();
    if let Ok(mut settings) = db.get_premium(guild_id.0 as i64) {
        match op.as_str() {
            "add" => {
                if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
                    settings.cooldown_restricted_roles.push(role_id.0 as i64);
                    sec = "Added";
                    val = role.name;
                }
            },
            "del" => {
                if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
                    settings.cooldown_restricted_roles.push(role_id.0 as i64);
                    sec = "Removed";
                    val = role.name;
                }
            },
            "set" => {
                let list = args.rest().split(",").map(|s| s.trim().to_string());
                let mut roles = Vec::new();
                let mut role_names = Vec::new();
                for role in list {
                    if let Some((role_id, role)) = parse_role(role, guild_id) {
                        roles.push(role_id.0 as i64);
                        role_names.push(role.name);
                    }
                }
                settings.cooldown_restricted_roles = roles;
                sec = "Set to";
                val = role_names.join(", ");
            },
            _ => {},
        }
        db.update_premium(guild_id.0 as i64, settings)?;
        message.channel_id.say(format!("Successfully modified restricted roles. {} {}", sec, val))?;
    }
});

command!(prune(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
    let mut count = args.single::<usize>().unwrap();
    let fsel = args.single::<String>().unwrap_or(String::new());
    let mut filter = get_filter(fsel, guild_id);
    let mut deletions = message.channel_id.messages(|_| re_retriever(100)).unwrap();
    let mut next_deletions;
    let mut num_del = 0;
    message.delete().expect("Failed to delete message");
    if count<1000 {
        while count>0 {
            deletions.retain(|m| filter(m));
            let mut len = deletions.len();
            if len>count {
                deletions.truncate(count);
                len = count;
            }
            count -= len;
            if count>0 {
                next_deletions = match message.channel_id.messages(|_| be_retriever(deletions.first().unwrap().id, 100)) {
                    Ok(msgs) => Some(msgs),
                    Err(_) => None,
                }
            } else {
                next_deletions = None;
            }
            match message.channel_id.delete_messages(deletions) {
                Ok(_) => {
                    num_del += len;
                    deletions = match next_deletions {
                        Some(s) => s,
                        None => { break; },
                    }
                },
                Err(why) => {
                    error!("{:?}", why);
                    break;
                },
            }
        }
        if guild_data.modlog {
            let channel_lock = message.channel_id.get().unwrap().guild().unwrap();
            let channel = channel_lock.read();
            ChannelId(guild_data.modlog_channel as u64).send_message(|m| m
                .embed(|e| e
                    .title("Messages Pruned")
                    .description(format!("**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {} ({})",
                        num_del,
                        message.author.mention(),
                        message.author.tag(),
                        channel.mention(),
                        channel.name))
                    .timestamp(now!())
            ))?;
        } else {
            message.channel_id.say(format!("Pruned {} message!", num_del))?;
        }
    }
});

command!(test_welcome(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
    let member = message.member().unwrap();
    if guild_data.welcome {
        let channel = ChannelId(guild_data.welcome_channel as u64);
        if guild_data.welcome_type.as_str() == "embed" {
            send_welcome_embed(guild_data.welcome_message, &member, channel)?;
        } else {
            channel.say(parse_welcome_items(guild_data.welcome_message, &member))?;
        }
    }
});

command!(setup_mute(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
        Some(role) => role.clone(),
        None => {
            message.channel_id.say("Role `Muted` created")?;
            guild.create_role(|r| r.name("Muted")).unwrap()
        },
    };
    let allow = Permissions::empty();
    let deny = Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS | Permissions::SPEAK;
    let overwrite = PermissionOverwrite {
        allow,
        deny,
        kind: PermissionOverwriteType::Role(mute_role.id),
    };
    for channel in guild.channels.values() {
        let mut channel = channel.read();
        channel.create_permission(&overwrite)?;
    }
    guild_data.mute_setup = true;
    db.update_guild(guild.id.0 as i64, guild_data)?;
    message.channel_id.say(format!("Setup permissions for {} channels.", guild.channels.len()))?;
});

// Helper functions for commands::prune
fn re_retriever(limit: u64) -> GetMessages {
    GetMessages::default()
        .limit(limit)
}

fn be_retriever(id: MessageId, limit: u64) -> GetMessages {
    GetMessages::default()
        .before(id)
        .limit(limit)
}

fn get_filter(input: String, guild_id: GuildId) -> Box<FnMut(&Message) -> bool> {
    match input.as_str() {
        "bot" => Box::new(|m| m.author.bot),
        "mention" => Box::new(|m| !m.mentions.is_empty() && m.mention_everyone),
        "attachment" => Box::new(|m| !m.attachments.is_empty()),
        "!pin" => Box::new(|m| !m.pinned),
        _ => {
            match parse_user(input.to_string(), guild_id) {
                Some((user_id, _)) => {
                    Box::new(move |m| m.author.id == user_id)
                },
                None => {
                    Box::new(|_| true)
                },
            }
        },
    }
}

use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use serenity::builder::GetMessages;
use serenity::CACHE;
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

command!(config_raw(_ctx, message, _args) {
    if let Some(guild_id) = message.guild_id {
        let guild_data = db.get_guild(guild_id.0 as i64)?;
        message.channel_id.say(format!("{:?}", guild_data))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_list(_ctx, message, _args) {
    if let Some(guild_id) = message.guild_id {
        let guild_data = db.get_guild(guild_id.0 as i64)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(*colours::MAIN)
                .description(format!("{}", guild_data))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_prefix(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let pre = args.single::<String>()?;
        guild_data.prefix = pre;
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("Set prefix to {}", guild.prefix))?;
            },
            Err(_) => {
                message.channel_id.say("Failed to change prefix")?;
            },
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(config_autorole(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "add" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.autoroles.push(role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            "remove" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            "enable" => {
                guild_data.autorole = true;
            },
            "disable" => {
                guild_data.autorole = false;
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`, `enable`, `disable`. For more information see `help config autorole`")?; },
        }
        let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Autorole Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.autorole) } else { val } ,
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_admin(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "add" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.admin_roles.push(role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            "remove" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config admin`")?; },
        }
        db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Admin Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_mod(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "add" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.mod_roles.push(role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            "remove" => {
                match parse_role(val.to_string(), guild_id) {
                    Some((role_id, role)) => {
                        guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
                        val = format!("{} ({})", role.name, role_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that role.")?; }
                }
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config mod`")?; },
        }
        db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Mod Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_audit(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "enable" => {
                guild_data.audit = true;
            },
            "disable" => {
                guild_data.audit = false;
            },
            "channel" => {
                match parse_channel(val.to_string(), guild_id) {
                    Some((channel_id, channel)) => {
                        guild_data.audit_channel = channel_id.0 as i64;
                        val = format!("{} ({})", channel.name, channel_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that channel.")?; }
                }
            },
            "threshold" => {
                match val.parse::<i16>() {
                    Ok(th) => {
                        guild_data.audit_threshold = th;
                        val = format!("{}", th);
                    },
                    Err(_) => { message.channel_id.say("Please input a number as the threshold")?; }
                }
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `threshold`. For more information see `help config audit`")?; },
        }
        let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Audit Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.audit) } else { val },
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_modlog(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "enable" => {
                guild_data.modlog = true;
            },
            "disable" => {
                guild_data.modlog = false;
            },
            "channel" => {
                match parse_channel(val.to_string(), guild_id) {
                    Some((channel_id, channel)) => {
                        guild_data.modlog_channel = channel_id.0 as i64;
                        val = format!("{} ({})", channel.name, channel_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that channel.")?; }
                }
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`. For more information see `help config modlog`")?; },
        }
        let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Modlog Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.modlog) } else { val },
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_welcome(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "enable" => {
                guild_data.welcome = true;
            },
            "disable" => {
                guild_data.welcome = false;
            },
            "channel" => {
                match parse_channel(val.to_string(), guild_id) {
                    Some((channel_id, channel)) => {
                        guild_data.welcome_channel = channel_id.0 as i64;
                        val = format!("{} ({})", channel.name, channel_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that channel.")?; }
                }
            },
            "message" => {
                guild_data.welcome_message = val.to_string();
            },
            "type" => {
                guild_data.welcome_type = val.to_string();
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config welcome`")?; },
        }
        let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Welcome Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.welcome) } else { val },
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(config_introduction(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut guild_data = db.get_guild(guild_id.0 as i64)?;
        let op = args.single::<String>().unwrap_or(String::new());
        let mut val = args.rest().to_string();
        match op.to_lowercase().as_str() {
            "enable" => {
                guild_data.introduction = true;
            },
            "disable" => {
                guild_data.introduction = false;
            },
            "channel" => {
                match parse_channel(val.to_string(), guild_id) {
                    Some((channel_id, channel)) => {
                        guild_data.introduction_channel = channel_id.0 as i64;
                        val = format!("{} ({})", channel.name, channel_id.0);
                    },
                    None => { message.channel_id.say("I couldn't find that channel.")?; }
                }
            },
            "message" => {
                guild_data.introduction_message = val.to_string();
            },
            "type" => {
                guild_data.welcome_type = val.to_string();
            },
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config introduction`")?; },
        }
        let guild = db.update_guild(guild_id.0 as i64, guild_data)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Config Introduction Summary")
                .colour(*colours::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.introduction) } else { val },
                ))
        ))?;
    } else { failed!(GUILDID_FAIL); }
});

// TODO add hackban and ignore lists views
command!(hackban(_ctx, message, args) {
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
command!(ignore(_ctx, message, args) {
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

command!(csr(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let switches = get_switches(args.full().to_string());
        let backup = String::new();
        let rest = switches.get("rest").unwrap_or(&backup);
        if let Some((role_id, _)) = parse_role(rest.clone(), guild_id) {
            let category = match switches.get("c") {
                Some(s) => Some( s.clone()),
                None => None,
            };
            let aliases = match switches.get("a") {
                Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
                None => None,
            };
            let data = db.new_role(role_id.0 as i64, guild_id.0 as i64, category, aliases)?;
            message.channel_id.say(format!("Successfully added role {} to category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            ))?;
        } else { message.channel_id.say("I couldn't find that role.")?; }
    } else { failed!(GUILDID_FAIL); }
});

command!(dsr(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        if let Some((role_id, _)) = parse_role(args.full().to_string(), guild_id) {
            let data = db.del_role(role_id.0 as i64, guild_id.0 as i64)?;
            message.channel_id.say(format!("Successfully deleted role {}", data))?;
        } else { message.channel_id.say("I couldn't find that role.")?; }
    } else { failed!(GUILDID_FAIL); }
});

command!(esr(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let switches = get_switches(args.full().to_string());
        let backup = String::new();
        let rest = switches.get("rest").unwrap_or(&backup);
        if let Some((role_id, _)) = parse_role(rest.clone(), guild_id) {
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
            let data = db.update_role(role_id.0 as i64, guild_id.0 as i64, role)?;
            message.channel_id.say(format!("Successfully update role {} in category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            ))?;
        } else { message.channel_id.say("I couldn't find that role.")?; }
    } else { failed!(GUILDID_FAIL); }
});

command!(premium_reg_member(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut settings = db.get_premium(guild_id.0 as i64)?;
        if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
            settings.register_member_role = Some(role_id.0 as i64);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set member role to {}", role.name))?;
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(premium_reg_cooldown(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut settings = db.get_premium(guild_id.0 as i64)?;
        if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
            settings.register_cooldown_role = Some(role_id.0 as i64);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set cooldown role to {}", role.name))?;
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(premium_reg_dur(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let mut settings = db.get_premium(guild_id.0 as i64)?;
        if let Ok(dur) = args.full().parse::<String>() {
            let dur = hrtime_to_seconds(dur);
            settings.register_cooldown_duration = Some(dur as i32);
            db.update_premium(guild_id.0 as i64, settings)?;
            message.channel_id.say(format!("Set duration of cooldown to {}", seconds_to_hrtime(dur as usize)))?;
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(premium_reg_restrict(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let op = args.single::<String>().unwrap_or(String::new());
        let mut sec = "";
        let mut val = String::new();
        let mut settings = db.get_premium(guild_id.0 as i64)?;
        match op.as_str() {
            "add" => {
                if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
                    settings.cooldown_restricted_roles.push(role_id.0 as i64);
                    sec = "Added";
                    val = role.name;
                }
            },
            "remove" => {
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
            _ => { message.channel_id.say("I didn't understand that option. Valid options are: `add`, `remove`, `set`. For more information see `help p reg_roles`")?; },
        }
        db.update_premium(guild_id.0 as i64, settings)?;
        message.channel_id.say(format!("Successfully modified restricted roles. {} {}", sec, val))?;
    } else { failed!(GUILDID_FAIL); }
});

command!(prune(_ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        let guild_data = db.get_guild(guild_id.0 as i64)?;
        let mut count = args.single::<usize>().unwrap_or(0);
        let fsel = args.single::<String>().unwrap_or(String::new());
        let mut filter = get_filter(fsel, guild_id);
        let mut deletions = message.channel_id.messages(|_| re_retriever(100))?;
        let mut next_deletions;
        let mut num_del = 0;
        message.delete()?;
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
                    next_deletions = message.channel_id.messages(|_| be_retriever(deletions[0].id, 100)).ok();
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
                let channel = {
                    let cache = CACHE.read();
                    cache.guild_channel(message.channel_id)
                };
                ChannelId(guild_data.modlog_channel as u64).send_message(|m| m
                    .embed(|e| e
                        .title("Messages Pruned")
                        .description(format!("**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {}",
                            num_del,
                            message.author.mention(),
                            message.author.tag(),
                            match channel {
                                Some(ch) => {
                                    let ch = ch.read();
                                    format!(
                                        "{} ({})",
                                        ch.mention(),
                                        ch.name)
                                },
                                None => format!("{}", message.channel_id.0),
                            }))
                        .timestamp(now!())
                ))?;
            } else {
                message.channel_id.say(format!("Pruned {} message!", num_del))?;
            }
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(test_welcome(_ctx, message, _args) {
    if let Some(guild_id) = message.guild_id {
        if let Some(member) = message.member() {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            if guild_data.welcome {
                let channel = ChannelId(guild_data.welcome_channel as u64);
                if guild_data.welcome_type.as_str() == "embed" {
                    send_welcome_embed(guild_data.welcome_message, &member, channel)?;
                } else {
                    channel.say(parse_welcome_items(guild_data.welcome_message, &member))?;
                }
            }
        } else { failed!(MEMBER_FAIL); }
    } else { failed!(GUILDID_FAIL); }
});

command!(setup_mute(_ctx, message, _args) {
    if let Some(guild_id) = message.guild_id {
        let guild = {
            let cache = CACHE.read();
            cache.guild(guild_id)
        };
        if let Some(guild_lock) = guild {
            let guild = guild_lock.read().clone();
            let mut guild_data = db.get_guild(guild_id.0 as i64)?;
            let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                Some(role) => role.clone(),
                None => {
                    message.channel_id.say("Role `Muted` created")?;
                    guild.create_role(|r| r.name("Muted"))?
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
        }
    } else { failed!(GUILDID_FAIL); }
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

use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::TC;
use core::utils::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::id::{
    ChannelId,
    RoleId
};
use serenity::model::guild::Member;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct Register;
impl Command for Register {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("A premium command that adds roles to a user (from the self roles list only), and depending on the settings for the command, will apply either a member role or a cooldown role with a timer. When the timer ends, cooldown is removed and member is added. In order for the switch to occur automatically, this command must be used. See the premium commands for more information on configuring this command.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            example: Some("@Adelyn gamer, techie".to_string()),
            aliases: vec!["reg"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_ROLES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, ctx: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let settings = db.get_premium(guild_id.0 as i64).map_err(|_| "Premium is required to use this command.")?;
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
                    let mut to_add = filter_roles(to_add, guild_id.member(&message.author)?);
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
                            let tc_lock = data.get::<TC>().ok_or("Failed to obtain timer client.")?;
                            let tc = tc_lock.lock();
                            let dur = match settings.register_cooldown_duration {
                                Some(dur) => dur,
                                None => DAY as i32,
                            };
                            let data = format!("COOLDOWN||{}||{}||{}||{}",
                                user_id.0,
                                guild_id.0,
                                member_role,
                                role);
                            let start_time = Utc::now().timestamp();
                            let end_time = start_time + dur as i64;
                            check_error!(db.new_timer(start_time, end_time, data));
                            tc.request();
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
            required_permissions: Permissions::MANAGE_ROLES,
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
                let mut to_add = filter_roles(to_add, guild_id.member(&message.author)?);
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
            required_permissions: Permissions::MANAGE_ROLES,
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
                let mut to_remove = filter_roles(to_remove, guild_id.member(&message.author)?);
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
            required_permissions: Permissions::MANAGE_ROLES,
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

fn filter_roles(roles: Vec<RoleId>, member: Member) -> Vec<RoleId> {
    let highest = match member.highest_role_info() {
        Some((_,h)) => h,
        None => -1,
    };
    roles.into_iter()
        .filter_map(|r| {
            let role = r.to_role_cached()?;
            match role.position >= highest {
                true => None,
                false => Some(r),
            }
        })
        .collect()
}

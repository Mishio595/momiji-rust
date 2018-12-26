use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use fuzzy_match::fuzzy_match;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::guild::Role;
use serenity::model::id::RoleId;
use serenity::prelude::Context;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct AddSelfRole;
impl Command for AddSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add roles to yourself provided they are on the self role list.".to_string()),
            usage: Some("<role_resolvables as CSV>".to_string()),
            example: Some("red, green".to_string()),
            aliases: vec!["addselfrole", "asr"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some(mut member) = message.member() {
                let roles = db.get_roles(guild_id.0 as i64)?;
                let (restricted_roles, has_cooldown) = match db.get_premium(guild_id.0 as i64) {
                    Ok(data) => {
                        let has_cooldown = if let Some(cooldown_role) = data.register_cooldown_role {
                            member.roles.contains(&RoleId(cooldown_role as u64))
                        } else {
                            false
                        };
                        (data.cooldown_restricted_roles, has_cooldown)
                    },
                    Err(_) => {
                        (Vec::new(), false)
                    },
                };
                if !roles.is_empty() {
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_add = Vec::new();
                    let mut failed = Vec::new();
                    let role_names = roles.iter().filter_map(|r| match RoleId(r.id as u64).to_role_cached() {
                        Some(role) => Some(role.clone()),
                        None => None,
                    }).collect::<Vec<Role>>();
                    for r1 in list {
                        if let Some((r, r2)) = parse_role(r1.clone(), guild_id) {
                            if has_cooldown && restricted_roles.contains(&(r.0 as i64)) {
                                failed.push(format!("{} is not available on cooldown", r2.name));
                                continue;
                            }
                            if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                to_add.push(r);
                            } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1.to_lowercase())) {
                            if has_cooldown && restricted_roles.contains(&(roles[i].id)) {
                                failed.push(format!("{} is not available on cooldown", match RoleId(roles[i].id as u64).to_role_cached() {
                                    Some(role) => role.name,
                                    None => roles[i].id.to_string(),
                                }));
                                continue;
                            }
                            to_add.push(RoleId(roles[i].id as u64));
                        } else {
                            failed.push(format!("Failed to find match \"{}\". {}", r1,
                                if let Some(i) = fuzzy_match(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                    let ref val = role_names[i];
                                    format!("Closest match: {}", val.name.clone())
                                } else { String::new() }
                            ));
                        }
                    }
                    for (i, role_id) in to_add.clone().iter().enumerate() {
                        if member.roles.contains(role_id) {
                            to_add.remove(i);
                            failed.push(format!("You already have {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        }
                        if let Err(_) = member.add_role(*role_id) {
                            to_add.remove(i);
                            failed.push(format!("Failed to add {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        };
                    }
                    let mut fields = Vec::new();
                    if !to_add.is_empty() {
                        fields.push(("Added Roles", to_add.iter().filter_map(|r| match r.to_role_cached() {
                            Some(r) => Some(r.name.clone()),
                            None => None,
                        }).collect::<Vec<String>>().join("\n").to_string(), false));
                    }
                    if !failed.is_empty() {
                        fields.push(("Failed to Add", failed.join("\n"), false));
                    }
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Add Self Role Summary")
                            .fields(fields)
                            .colour(member.colour().unwrap_or(*colours::GREEN))
                    ))?;
                } else {
                    message.channel_id.say("There are no self roles.")?;
                }
            } else { failed!(MEMBER_FAIL); }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct RemoveSelfRole;
impl Command for RemoveSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Remove roles from yourself provided they are on the self role list.".to_string()),
            usage: Some("<role_resolvables as CSV>".to_string()),
            example: Some("red, green".to_string()),
            aliases: vec!["removeselfrole", "rsr"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some(mut member) = message.member() {
                let roles = db.get_roles(guild_id.0 as i64)?;
                if !roles.is_empty() {
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_remove = Vec::new();
                    let mut failed = Vec::new();
                    let role_names = roles.iter().filter_map(|r| match RoleId(r.id as u64).to_role_cached() {
                        Some(role) => Some(role.clone()),
                        None => None,
                    }).collect::<Vec<Role>>();
                    for r1 in list {
                        if let Some((r, r2)) = parse_role(r1.clone(), guild_id) {
                            if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                to_remove.push(r);
                            } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1.to_lowercase())) {
                            to_remove.push(RoleId(roles[i].id as u64));
                        } else {
                            failed.push(format!("Failed to find match \"{}\". {}", r1,
                                if let Some(i) = fuzzy_match(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                    let ref val = role_names[i];
                                    format!("Closest match: {}", val.name.clone())
                                } else { String::new() }
                            ));
                        }
                    }
                    for (i, role_id) in to_remove.clone().iter().enumerate() {
                        if !member.roles.contains(role_id) {
                            to_remove.remove(i);
                            failed.push(format!("You don't have {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        }
                        if let Err(_) = member.remove_role(*role_id) {
                            to_remove.remove(i);
                            failed.push(format!("Failed to remove {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        };
                    }
                    let mut fields = Vec::new();
                    if !to_remove.is_empty() {
                        fields.push(("Removed Roles", to_remove.iter().filter_map(|r| match r.to_role_cached() {
                            Some(r) => Some(r.name.clone()),
                            None => None,
                        }).collect::<Vec<String>>().join("\n").to_string(), false));
                    }
                    if !failed.is_empty() {
                        fields.push(("Failed to Remove", failed.join("\n"), false));
                    }
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Remove Self Role Summary")
                            .fields(fields)
                            .colour(member.colour().unwrap_or(*colours::RED))
                    ))?;
                } else {
                    message.channel_id.say("There are no self roles.")?;
                }
            } else { failed!(MEMBER_FAIL); }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct ListSelfRoles;
impl Command for ListSelfRoles {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List all the self roles for the current server. Optionally, you can view a single category.".to_string()),
            usage: Some("[category]".to_string()),
            aliases: vec!["listselfroles", "lsr"].iter().map(|e| e.to_string()).collect(),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let mut roles = db.get_roles(guild_id.0 as i64)?;
            if !roles.is_empty() {
                if args.is_empty() {
                    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
                    for role in roles.iter() {
                        match RoleId(role.id as u64).to_role_cached() {
                            Some(r) => {
                                map.entry(role.category.clone()).or_insert(Vec::new()).push(r.name);
                            },
                            None => {
                                // Clean up roles that don't exist
                                db.del_role(role.id, guild_id.0 as i64)?;
                            },
                        }
                    }
                    let mut fields = Vec::new();
                    for (key, val) in map.iter_mut() {
                        val.sort();
                        fields.push((key, val.join("\n"), true));
                    }
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Self Roles")
                            .fields(fields)
                            .colour(*colours::MAIN)
                    ))?;
                } else {
                    let category = args.full().to_string();
                    roles.retain(|e| *e.category.to_lowercase() == category.to_lowercase());
                    if !roles.is_empty() {
                        let roles_out = roles
                            .iter()
                            .map(|e| match RoleId(e.id as u64).to_role_cached() {
                                Some(r) => r.name,
                                None => e.id.to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        message.channel_id.send_message(|m| m
                            .embed(|e| e
                                .title(category)
                                .description(roles_out)
                                .colour(*colours::MAIN)
                        ))?;
                    } else {
                        message.channel_id.say(format!("The category `{}` does not exist.", category))?;
                    }
                }
            } else {
                message.channel_id.say("There are no self roles.")?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}
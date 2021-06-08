use fuzzy_match::fuzzy_match;
use momiji::core::consts::*;
use momiji::core::timers::TimerClient;
use momiji::db::DatabaseConnection;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::debug;
use twilight_cache_inmemory::InMemoryCache;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_http::Client as HttpClient;
use twilight_http::request::AuditLogReason;
use twilight_model::{
    channel::Message,
    id::RoleId,
    guild::Role,
};
use std::{collections::BTreeMap, error::Error};
use std::sync::Arc;

pub struct AddSelfRole;
#[async_trait]
impl Command for AddSelfRole {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Add roles to yourself provided they are on the self role list.".to_string()),
            usage: Some("<role_resolvables as CSV>".to_string()),
            examples: vec!["red, green".to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Some(member) = message.member {
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
                    let role_names: Vec<Arc<Role>> = roles.iter().filter_map(|r| match cache.role(RoleId(r.id as u64)) {
                        Some(role) => Some(role.clone()),
                        None => None,
                    }).collect();
                    for r1 in list {
                        if let Some((r, r2)) = parse_role(r1.clone(), guild_id, &cache) {
                            if has_cooldown && restricted_roles.contains(&(r.0 as i64)) {
                                failed.push(format!("{} is not available on cooldown", r2.name));
                                continue;
                            }
                            if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                to_add.push(r);
                            } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1.to_lowercase())) {
                            if has_cooldown && restricted_roles.contains(&(roles[i].id)) {
                                let rid = RoleId(roles[i].id as u64);
                                failed.push(format!("{} is not available on cooldown", cache
                                    .guild_roles(guild_id)
                                    .and_then(|set| set.get(&rid).cloned())
                                    .unwrap_or(rid)
                                ));
                                continue;
                            }
                            to_add.push(RoleId(roles[i].id as u64));
                        } else {
                            failed.push(format!("Failed to find match \"{}\". {}", r1,
                                if let Some(i) = fuzzy_match::<usize, Vec<(&str, usize)>>(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                    format!("Closest match: {}", role_names[i].name.clone())
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
                        if let Err(_) = http.add_guild_member_role(guild_id, message.author.id, *role_id).reason("Self role")?.await {
                            to_add.remove(i);
                            failed.push(format!("Failed to add {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        };
                    }

                    
                    let mut embed = EmbedBuilder::new()
                        .title("Add Self Role Summary")
                        .color(colors::GREEN);

                    if !to_add.is_empty() {
                        let value = to_add.iter().filter_map(|r| match cache.role(*r) {
                            Some(r) => Some(r.name.clone()),
                            None => None,
                        }).collect::<Vec<String>>().join("\n").to_string();

                        embed = embed.field(EmbedFieldBuilder::new("Added Roles", value));
                    }
                    if !failed.is_empty() {
                        embed = embed.field(EmbedFieldBuilder::new("Failed to Add", failed.join("\n")));
                    }

                    http.create_message(message.channel_id).reply(message.id).embed(embed.build()?)?.await?;
                } else {
                    http.create_message(message.channel_id).reply(message.id).content("There are no self roles.")?.await?;
                }
            } else { debug!("{}", MEMBER_FAIL); }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct RemoveSelfRole;
#[async_trait]
impl Command for RemoveSelfRole {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Remove roles from yourself provided they are on the self role list.".to_string()),
            usage: Some("<role_resolvables as CSV>".to_string()),
            examples: vec!["red, green".to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Some(member) = message.member {
                let roles = db.get_roles(guild_id.0 as i64)?;
                if !roles.is_empty() {
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_remove = Vec::new();
                    let mut failed = Vec::new();
                    let role_names = roles.iter().filter_map(|r| match cache.role(RoleId(r.id as u64)) {
                        Some(role) => Some(role.clone()),
                        None => None,
                    }).collect::<Vec<Arc<Role>>>();
                    for r1 in list {
                        if let Some((r, r2)) = parse_role(r1.clone(), guild_id, &cache) {
                            if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                to_remove.push(r);
                            } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1.to_lowercase())) {
                            to_remove.push(RoleId(roles[i].id as u64));
                        } else {
                            failed.push(format!("Failed to find match \"{}\". {}", r1,
                                if let Some(i) = fuzzy_match::<usize, Vec<(&str, usize)>>(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                    format!("Closest match: {}", role_names[i].name.clone())
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
                        if let Err(_) = http.remove_guild_member_role(guild_id, message.author.id, *role_id).reason("Self role")?.await {
                            to_remove.remove(i);
                            failed.push(format!("Failed to remove {}", match role_names.iter().find(|r| &r.id == role_id) {
                                Some(s) => s.name.clone(),
                                None => role_id.0.to_string(),
                            }));
                        };
                    }
                    let mut embed = EmbedBuilder::new()
                        .title("Remove Self Role Summary")
                        .color(colors::RED);

                    if !to_remove.is_empty() {
                        let value = to_remove.iter().filter_map(|r| match cache.role(*r) {
                            Some(r) => Some(r.name.clone()),
                            None => None,
                        }).collect::<Vec<String>>().join("\n").to_string();

                        embed = embed.field(EmbedFieldBuilder::new("Removed Roles", value));
                    }
                    if !failed.is_empty() {
                        embed = embed.field(EmbedFieldBuilder::new("Failed to remove", failed.join("\n")));
                    }

                    http.create_message(message.channel_id).reply(message.id).embed(embed.build()?)?.await?;
                } else {
                    http.create_message(message.channel_id).reply(message.id).content("There are no self roles.")?.await?;
                }
            } else { debug!("{}", MEMBER_FAIL); }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct ListSelfRoles;
#[async_trait]
impl Command for ListSelfRoles {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("List all the self roles for the current server. Optionally, you can view a single category.".to_string()),
            usage: Some("[category]".to_string()),
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut roles = db.get_roles(guild_id.0 as i64)?;
            if !roles.is_empty() {
                if args.is_empty() {
                    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
                    for role in roles.iter() {
                        match cache.role(RoleId(role.id as u64)) {
                            Some(r) => {
                                map.entry(role.category.clone()).or_insert(Vec::new()).push(r.name.clone());
                            },
                            None => {
                                // Clean up roles that don't exist
                                db.del_role(role.id, guild_id.0 as i64)?;
                            },
                        }
                    }

                    let mut embed = EmbedBuilder::new()
                        .title("Self Roles")
                        .color(colors::MAIN);

                    for (key, val) in map.iter_mut() {
                        val.sort();
                        embed = embed.field(EmbedFieldBuilder::new(key, val.join("\n")));
                    }
                    http.create_message(message.channel_id).reply(message.id).embed(embed.build()?)?.await?;
                } else {
                    let category = args.full().to_string();
                    roles.retain(|e| *e.category.to_lowercase() == category.to_lowercase());
                    if !roles.is_empty() {
                        let mut roles = roles
                            .iter()
                            .map(|e| match cache.role(RoleId(e.id as u64)) {
                                Some(r) => r.name.clone(),
                                None => e.id.to_string(),
                            })
                            .collect::<Vec<String>>();
                        roles.sort();

                        let embed = EmbedBuilder::new()
                            .title(category)
                            .description(roles.join("\n"))
                            .color(colors::MAIN)
                            .build()?;

                        http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
                    } else {
                        http.create_message(message.channel_id).reply(message.id).content(format!("The category `{}` does not exist.", category))?.await?;
                    }
                }
            } else {
                http.create_message(message.channel_id).reply(message.id).content("There are no self roles.")?.await?;
            }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

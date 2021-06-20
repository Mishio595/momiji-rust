use chrono::Utc;
use momiji::Context;
use momiji::core::consts::*;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
// use tracing::{event, Level};
use twilight_model::channel::Message;
use twilight_model::guild::{Member, Role};
use twilight_model::guild::Permissions;
use twilight_model::id::{ChannelId, RoleId};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_mention::Mention;
use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error;
pub struct Register;
#[async_trait]
impl Command for Register {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("A command that adds roles to a user (from the self roles list only), and depending on the settings for the command, will apply either a member role or a cooldown role with a timer. When the timer ends, cooldown is removed and member is added. In order for the switch to occur automatically, this command must be used. See the premium commands for more information on configuring this command.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            examples: vec!["@Adelyn gamer, techie".to_string()],
            required_permissions: Permissions::MANAGE_ROLES,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = ctx.db.clone();
        let http = ctx.http.clone();
        if let Some(guild_id) = message.guild_id {
            let guild_data = db.get_guild(guild_id.0 as i64)?;
            let roles = db.get_roles(guild_id.0 as i64)?;
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id, ctx.clone()).await {
                Some((user_id, member)) => {
                    let channel_id = if guild_data.modlog {
                        ChannelId(guild_data.modlog_channel as u64)
                    } else { message.channel_id };
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_add = Vec::new();
                    for r1 in list {
                        if let Some((r, _)) = parse_role(r1.clone(), guild_id, ctx.clone()) {
                            if guild_data.cooldown_restricted_roles.contains(&(r.0 as i64)) { continue; }
                            to_add.push(r);
                        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
                            if guild_data.cooldown_restricted_roles.contains(&(roles[i].id)) { continue; }
                            to_add.push(RoleId(roles[i].id as u64));
                        }
                    }
                    // let author = message.member.ok_or_else(|| "Member unavailable")?;
                    // TODO implement hierarchy filter
                    // let mut to_add = filter_roles(to_add, author);
                    for (i, role_id) in to_add.clone().iter().enumerate() {
                        if member.roles.contains(role_id) {
                            to_add.remove(i);
                            continue;
                        }
                        if let Err(_) = http.add_guild_member_role(guild_id, user_id, *role_id).await {
                            to_add.remove(i);
                        };
                    }
                    if let Some(role) = guild_data.register_cooldown_role {
                        http.add_guild_member_role(guild_id, user_id, RoleId(role as u64)).await?;
                        if let Some(member_role) = guild_data.register_member_role {
                            let dur = match guild_data.register_cooldown_duration {
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
                            db.new_timer(start_time, end_time, data)?;
                            ctx.tc.request();
                        }
                    } else if let Some(role) = guild_data.register_member_role {
                        http.add_guild_member_role(guild_id, user_id, RoleId(role as u64)).await?;
                    }
                    let desc = if !to_add.is_empty() {
                        to_add.iter().map(|r| match ctx.cache.role(*r) {
                            Some(role) => role.name.clone(),
                            None => r.0.to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                    } else { String::new() };
                    let embed = EmbedBuilder::new()
                        .title(format!(
                            "Registered {}#{} with the following roles:",
                            member.user.name,
                            member.user.discriminator,
                        ))
                        .description(desc)
                        .color(colors::MAIN)
                        .timestamp(Utc::now().to_rfc3339())
                        .build()?;
                    http.create_message(channel_id).embed(embed)?.await?;
                    if guild_data.introduction && guild_data.introduction_channel>0 {
                        let channel = ChannelId(guild_data.introduction_channel as u64);
                        if guild_data.introduction_type == "embed" {
                            let embed = build_welcome_embed(guild_data.introduction_message, &member, ctx.clone())?.build()?;
                            http.create_message(channel).embed(embed)?.await?;
                        } else {
                            http.create_message(channel).content(parse_welcome_items(guild_data.introduction_message, &member, ctx.clone()))?.await?;
                        }
                    }
                },
                None => { http.create_message(message.channel_id).reply(message.id).content("I couldn't find that user.")?.await?; }
            }
        }
        Ok(())
    }
}

pub struct AddRole;
#[async_trait]
impl Command for AddRole {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Add role(s) to a user.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            examples: vec!["@Adelyn red, green".to_string()],
            required_permissions: Permissions::MANAGE_ROLES,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Some((_, member)) = parse_user(args.single::<String>()?, guild_id, ctx.clone()).await {
                if let Some(author) = ctx.http.guild_member(guild_id, message.author.id).await? {
                    let author_highest_role = get_highest_role(Arc::new(author), ctx.clone()).await?;
                    let target_highest_role = get_highest_role(member.clone(), ctx.clone()).await?;
                    if target_highest_role > author_highest_role {
                        ctx.http.create_message(message.channel_id).reply(message.id)
                            .content("Cannot modify roles of someone higher on the role hierachy.")?
                            .await?;

                        return Ok(())
                    }
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_add = Vec::new();
                    let mut failed = Vec::new();
                    for r1 in list {
                        if let Some((_, role)) = parse_role(r1.clone(), guild_id, ctx.clone()) {
                            to_add.push(role);
                        } else {
                            failed.push(format!("Could not locate {}", r1));
                        }
                    }
                    let mut to_add = filter_roles(to_add, author_highest_role, member.clone(), ctx.clone());
                    for (i, role) in to_add.clone().iter().enumerate() {
                        if member.roles.contains(&role.id) {
                            to_add.remove(i);
                            failed.push(format!("You already have {}", role.name));
                        } else {
                            if let Err(_) = ctx.http.add_guild_member_role(guild_id, member.user.id, role.id).await {
                                to_add.remove(i);
                                failed.push(format!("Failed to add {}", role.name));
                            }
                        }
                    }
                    let mut embed = EmbedBuilder::new()
                        .title("Add Role Summary")
                        .description(format!("Adding roles to {}", member.mention().to_string()))
                        .color(colors::GREEN);
                        
                    if !to_add.is_empty() {
                        let roles = to_add.into_iter()
                            .map(|r| r.name.clone())
                            .collect::<Vec<String>>()
                            .join("\n");
                        let field = EmbedFieldBuilder::new("Added Roles", roles).build();
                        
                        embed = embed.field(field);
                    }
                    if !failed.is_empty() {
                        let field = EmbedFieldBuilder::new("Failed to add", failed.join("\n")).build();

                        embed = embed.field(field);
                    }
                    ctx.http.create_message(message.channel_id).reply(message.id)
                        .embed(embed.build()?)?
                        .await?;
                }
            }
        }
        Ok(())
    }
}

pub struct RemoveRole;
#[async_trait]
impl Command for RemoveRole {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Remove role(s) from a user.".to_string()),
            usage: Some("<user_resolvable> <role_resolvables as CSV>".to_string()),
            examples: vec!["@Adelyn red, green".to_string()],
            required_permissions: Permissions::MANAGE_ROLES,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Some((_, member)) = parse_user(args.single::<String>()?, guild_id, ctx.clone()).await {
                if let Some(author) = ctx.http.guild_member(guild_id, message.author.id).await? {
                    let author_highest_role = get_highest_role(Arc::new(author), ctx.clone()).await?;
                    let target_highest_role = get_highest_role(member.clone(), ctx.clone()).await?;
                    if target_highest_role > author_highest_role {
                        ctx.http.create_message(message.channel_id).reply(message.id)
                            .content("Cannot modify roles of someone higher on the role hierachy.")?
                            .await?;

                        return Ok(())
                    }
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut to_remove = Vec::new();
                    let mut failed = Vec::new();
                    for r1 in list {
                        if let Some((_, role)) = parse_role(r1.clone(), guild_id, ctx.clone()) {
                            to_remove.push(role);
                        } else {
                            failed.push(format!("Could not locate {}", r1));
                        }
                    }
                    let mut to_remove = filter_roles(to_remove, author_highest_role, member.clone(), ctx.clone());
                    for (i, role) in to_remove.clone().iter().enumerate() {
                        if !member.roles.contains(&role.id) {
                            to_remove.remove(i);
                            failed.push(format!("You don't have {}", role.name));
                        } else {
                            if let Err(_) = ctx.http.remove_guild_member_role(guild_id, member.user.id, role.id).await {
                                to_remove.remove(i);
                                failed.push(format!("Failed to remove {}", role.name));
                            }
                        }
                    }
                    let mut embed = EmbedBuilder::new()
                        .title("Remove Role Summary")
                        .description(format!("Removing roles from {}", member.mention().to_string()))
                        .color(colors::RED);
                        
                    if !to_remove.is_empty() {
                        let roles = to_remove.into_iter()
                            .map(|r| r.name.clone())
                            .collect::<Vec<String>>()
                            .join("\n");
                        let field = EmbedFieldBuilder::new("Removed Roles", roles).build();
                        
                        embed = embed.field(field);
                    }
                    if !failed.is_empty() {
                        let field = EmbedFieldBuilder::new("Failed to remove", failed.join("\n")).build();

                        embed = embed.field(field);
                    }
                    ctx.http.create_message(message.channel_id).reply(message.id)
                        .embed(embed.build()?)?
                        .await?;
                }
            }
        }
        Ok(())
    }
}

// pub struct RoleColour;
// #[async_trait]
// impl Command for RoleColour {
//     fn options(&self) -> Arc<Options> {
//         let options = Options {
//             description: Some("Change the colour of a role.".to_string()),
//             usage: Some("<role_resolvable> <colour>".to_string()),
//             examples: vec!["418130449089691658 00ff00".to_string()],
//             required_permissions: Permissions::MANAGE_ROLES,
//             guild_only: true,
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             match parse_role(args.single_quoted::<String>().unwrap_or(String::new()), guild_id, &ctx.cache) {
//                 Some((_, mut role)) => {
//                     let input = args.single::<String>()?;
//                     let colour_as_hex = if input.starts_with("#") {
//                         &input[1..]
//                     } else { input.as_str() };
//                     let colour = u64::from_str_radix(colour_as_hex, 16)?;
//                     role.edit(|r| r.colour(colour))?;
//                     message.channel_id.say(format!("Colour of `{}` changed to `#{:06X}`", role.name, colour))?;
//                 },
//                 None => { message.channel_id.say("I couldn't find that role")?; },
//             }
//         }
//         Ok(())
//     }
// }

fn filter_roles(roles: Vec<Arc<Role>>, highest: i64, member: Arc<Member>, ctx: Context) -> Vec<Arc<Role>> {
    roles.into_iter()
        .filter_map(|role| {
            match role.position >= highest {
                true => None,
                false => Some(role),
            }
        })
        .collect()
}

async fn get_highest_role(member: Arc<Member>, ctx: Context) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let roles = ctx.http.roles(member.guild_id).await?;
    let roles = {
        let mut map = HashMap::new();
        for role in roles.iter() {
            map.insert(role.id, role.clone());
        }

        map
    };

    let mut pos = -1;
    for role in member.roles.iter() {
        if let Some(role) = roles.get(role) {
            if role.position > pos { pos = role.position }
        }
    }

    Ok(pos)
}

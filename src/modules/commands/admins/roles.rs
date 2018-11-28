use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct CreateSelfRole;
impl Command for CreateSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Create a self role from a discord role. Also optionally takes a category and/or aliases.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV]".to_string()),
            example: Some("NSFW /c Opt-in /a porn, lewd".to_string()),
            aliases: vec!["createselfrole".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args
                .full()
                .to_string());
            let backup = String::new();
            let rest = switches
                .get("rest")
                .unwrap_or(&backup);
            if let Some((role_id, role)) = parse_role(rest.clone(), guild_id) {
                let category = switches
                    .get("c")
                    .cloned();
                let aliases: Option<Vec<String>> = switches
                    .get("a")
                    .map(|s| s
                        .split(",")
                        .map(|c| c
                            .trim()
                            .to_string()
                            .to_lowercase())
                    .collect());
                let data = db.new_role(
                    role_id.0 as i64,
                    guild_id.0 as i64,
                    category,
                    aliases)?;
                message.channel_id.say(format!(
                    "Successfully added role {} to category {} {}"
                    ,role.name
                    ,data.category
                    ,if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct DeleteSelfRole;
impl Command for DeleteSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Delete a self role.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            example: Some("NSFW".to_string()),
            aliases: vec!["deleteselfrole".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id) {
                db.del_role(role_id.0 as i64, guild_id.0 as i64)?;
                message.channel_id.say(format!("Successfully deleted role {}", role.name))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct EditSelfRole;
impl Command for EditSelfRole {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Edit a self role. Optionally takes a category and/or aliases. This operation is lazy and won't change anything you don't specify. Replace switch tells the bot to override aliases instead of append.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV] [/replace]".to_string()),
            example: Some("NSFW /c Opt-in /a porn, lewd /replace".to_string()),
            aliases: vec!["editselfrole".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args.full().to_string());
            let backup = String::new();
            let rest = switches.get("rest").unwrap_or(&backup);
            if let Some((role_id, d_role)) = parse_role(rest.clone(), guild_id) {
                let category = switches
                    .get("c")
                    .cloned();
                let aliases: Option<Vec<String>> = switches
                    .get("a")
                    .map(|s| s
                        .split(",")
                        .map(|c| c
                            .trim()
                            .to_string()
                            .to_lowercase())
                    .collect());
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
                    d_role.name,
                    data.category,
                    if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?;
            } else { message.channel_id.say("I couldn't find that role.")?; }
        } else {
            failed!(GUILDID_FAIL);
        }
        Ok(())
    }
}
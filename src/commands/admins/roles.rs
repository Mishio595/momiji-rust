use momiji::Context;
use momiji::core::consts::*;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::debug;
use twilight_model::{
    channel::Message,
    guild::Permissions,
};
use std::error::Error;
use std::sync::Arc;

pub struct CreateSelfRole;
#[async_trait]
impl Command for CreateSelfRole {
    fn options(&self) -> Arc<Options> {
        let default = Options::default();
        let options = Options {
            description: Some("Create a self role from a discord role. Also optionally takes a category and/or aliases.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV]".to_string()),
            examples: vec!["NSFW /c Opt-in /a porn, lewd".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args
                .full()
                .to_string());
            let backup = String::new();
            let rest = switches
                .get("rest")
                .unwrap_or(&backup);
            if let Some((role_id, role)) = parse_role(rest.clone(), guild_id, &ctx.cache) {
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
                let data = ctx.db.new_role(
                    role_id.0 as i64,
                    guild_id.0 as i64,
                    category,
                    aliases)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!(
                    "Successfully added role {} to category {} {}"
                    ,role.name
                    ,data.category
                    ,if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?.await?;
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?; }
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct DeleteSelfRole;
#[async_trait]
impl Command for DeleteSelfRole {
    fn options(&self) -> Arc<Options> {
        let default = Options::default();
        let options = Options {
            description: Some("Delete a self role.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            examples: vec!["NSFW".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Some((role_id, role)) = parse_role(args.full().to_string(), guild_id, &ctx.cache) {
                ctx.db.del_role(role_id.0 as i64, guild_id.0 as i64)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully deleted role {}", role.name))?.await?;
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?; }
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct EditSelfRole;
#[async_trait]
impl Command for EditSelfRole {
    fn options(&self) -> Arc<Options> {
        let default = Options::default();
        let options = Options {
            description: Some("Edit a self role. Optionally takes a category and/or aliases. This operation is lazy and won't change anything you don't specify. Replace switch tells the bot to override aliases instead of append.".to_string()),
            usage: Some("<role_resolvable> [/c category] [/a aliases as CSV] [/replace]".to_string()),
            examples: vec!["NSFW /c Opt-in /a porn, lewd /replace".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let switches = get_switches(args.full().to_string());
            let backup = String::new();
            let rest = switches.get("rest").unwrap_or(&backup);
            if let Some((role_id, d_role)) = parse_role(rest.clone(), guild_id, &ctx.cache) {
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
                let mut role = ctx.db.get_role(role_id.0 as i64, guild_id.0 as i64)?;
                if let Some(s) = category { role.category = s; }
                if let Some(mut a) = aliases {
                    match switches.get("replace") {
                        Some(_) => { role.aliases = a; },
                        None => { role.aliases.append(&mut a); },
                    }
                }
                let data = ctx.db.update_role(role_id.0 as i64, guild_id.0 as i64, role)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully update role {} in category {} {}",
                    d_role.name,
                    data.category,
                    if !data.aliases.is_empty() {
                        format!("with aliases {}", data.aliases.join(","))
                    } else {
                        String::new()
                    }
                ))?.await?;
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?; }
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}
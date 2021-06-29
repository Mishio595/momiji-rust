use momiji::Context;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::{event, Level};
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;
use std::error::Error;
use std::sync::Arc;
pub struct RegisterMember;
#[async_trait]
impl Command for RegisterMember {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Set the member role used by register. This role is automatically either after cooldown, if cooldown is set, or right away.".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            examples: vec!["member".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = ctx.db.get_guild(guild_id.0 as i64)?;
            event!(Level::DEBUG, "Obtained settings");
            event!(Level::DEBUG, "Args: {:?}", args.rest());
            if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id, ctx.clone()) {
                settings.register_member_role = Some(role_id.0 as i64);
                ctx.db.update_guild(guild_id.0 as i64, settings)?;
                event!(Level::DEBUG, "Updated settings");
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Set member role to {}", role.name))?.await?;
            }
        }
        
        Ok(())
    }
}

pub struct RegisterCooldown;
#[async_trait]
impl Command for RegisterCooldown {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Set the cooldown role used by register. This is applied automatically before member and removed after cooldown_duration".to_string()),
            usage: Some("<role_resolvable>".to_string()),
            examples: vec!["cooldown".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = ctx.db.get_guild(guild_id.0 as i64)?;
            if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id, ctx.clone()) {
                settings.register_cooldown_role = Some(role_id.0 as i64);
                ctx.db.update_guild(guild_id.0 as i64, settings)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Set cooldown role to {}", role.name))?.await?;
            }
        }
        
        Ok(())
    }
}

pub struct RegisterDuration;
#[async_trait]
impl Command for RegisterDuration {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Set the duration cooldown is applied for. Default value is 24 hours.".to_string()),
            usage: Some("<time_resolvable>".to_string()),
            examples: vec!["24h".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut settings = ctx.db.get_guild(guild_id.0 as i64)?;
            if let Ok(dur) = args.rest().parse::<String>() {
                let dur = hrtime_to_seconds(dur);
                settings.register_cooldown_duration = Some(dur as i32);
                ctx.db.update_guild(guild_id.0 as i64, settings)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Set duration of cooldown to {}", seconds_to_hrtime(dur as usize)))?.await?;
            }
        }
        
        Ok(())
    }
}

pub struct RegisterRestrictions;
#[async_trait]
impl Command for RegisterRestrictions {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Manage the roles people on cooldown cannot self-assign. These are also ignored in register command usage. Valid options: `add`, `remove`, `set`".to_string()),
            usage: Some("<option> [values]".to_string()),
            examples: vec!["set selfies, nsfw".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let op = args.single::<String>().unwrap_or(String::new());
            let mut sec = "";
            let mut val = String::new();
            let mut settings = ctx.db.get_guild(guild_id.0 as i64)?;
            match op.as_str() {
                "add" => {
                    if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id, ctx.clone()) {
                        settings.cooldown_restricted_roles.push(role_id.0 as i64);
                        sec = "Added";
                        val = role.name.clone();
                    }
                },
                "remove" => {
                    if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id, ctx.clone()) {
                        settings.cooldown_restricted_roles.push(role_id.0 as i64);
                        sec = "Removed";
                        val = role.name.clone();
                    }
                },
                "set" => {
                    let list = args.rest().split(",").map(|s| s.trim().to_string());
                    let mut roles = Vec::new();
                    let mut role_names = Vec::new();
                    for role in list {
                        if let Some((role_id, role)) = parse_role(role, guild_id, ctx.clone()) {
                            roles.push(role_id.0 as i64);
                            role_names.push(role.name.clone());
                        }
                    }
                    settings.cooldown_restricted_roles = roles;
                    sec = "Set to";
                    val = role_names.join(", ");
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `add`, `remove`, `set`. For more information see `help p reg_roles`")?.await?;
                    return Ok(())
                },
            }
            ctx.db.update_guild(guild_id.0 as i64, settings)?;
            ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully modified restricted roles. {} {}", sec, val))?.await?;
        }

        Ok(())
    }
}
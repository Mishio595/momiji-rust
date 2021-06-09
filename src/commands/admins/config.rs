use momiji::Context;
use momiji::core::consts::*;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::debug;
use twilight_embed_builder::EmbedBuilder;
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;
use std::sync::Arc;
use std::error::Error;

pub struct ConfigRaw;
#[async_trait]
impl Command for ConfigRaw {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Lists current configuration as raw output.".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            ctx.http.create_message(message.channel_id).reply(message.id).content(format!("{:?}", guild_data))?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigList;
#[async_trait]
impl Command for ConfigList {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Lists current configuration.".to_string()),
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let guild_data = ctx.db.get_guild(guild_id.0 as i64)?;

            let embed = EmbedBuilder::new()
                .color(colors::MAIN)
                .description(format!("{}", guild_data))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }

        Ok(())
    }
}

pub struct ConfigPrefix;
#[async_trait]
impl Command for ConfigPrefix {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Set a new prefix.".to_string()),
            usage: Some("<prefix>".to_string()),
            examples: vec!["!!".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let pre = args.single::<String>()?;
            guild_data.prefix = pre;
            match ctx.db.update_guild(guild_id.0 as i64, guild_data) {
                Ok(guild) => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Set prefix to {}", guild.prefix))?.await?;
                },
                Err(_) => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("Failed to change prefix")?.await?;
                },
            }
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAutorole;
#[async_trait]
impl Command for ConfigAutorole {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change autorole settings. A role must be provided for add or remove.".to_string()),
            usage: Some("<add|remove|enable|disable> <role_resolvable|_>".to_string()),
            examples: vec!["add member".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.autoroles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "enable" => {
                    guild_data.autorole = true;
                },
                "disable" => {
                    guild_data.autorole = false;
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `add`, `remove`, `enable`, `disable`. For more information see `help config autorole`")?.await?;
                    return Ok(())
                },
            }
            let guild = ctx.db.update_guild(guild_id.0 as i64, guild_data)?;

            let embed = EmbedBuilder::new()
                .title("Config Autorole Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { guild.autorole.to_string() } else { val } ,
                ))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAdmin;
#[async_trait]
impl Command for ConfigAdmin {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Add or remove roles from the bot's admin list.".to_string()),
            usage: Some("<add|remove> <role_resolvable>".to_string()),
            examples: vec!["add admin".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.admin_roles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config admin`")?.await?;
                    return Ok(())
                },
            }
            ctx.db.update_guild(guild_id.0 as i64, guild_data)?;

            let embed = EmbedBuilder::new()
                .title("Config Admin Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
                .build()?;
            
            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigMod;
#[async_trait]
impl Command for ConfigMod {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Add or remove roles from the bot's admin list.".to_string()),
            usage: Some("<add|remove> <role_resolvable>".to_string()),
            examples: vec!["add staff".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let mut val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "add" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.mod_roles.push(role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "remove" => {
                    match parse_role(val.to_string(), guild_id, &ctx.cache) {
                        Some((role_id, role)) => {
                            guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
                            val = format!("{} ({})", role.name, role_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that role.")?.await?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `add`, `remove`. For more information see `help config mod`")?.await?;
                    return Ok(())
                },
            }
            ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Mod Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigAudit;
#[async_trait]
impl Command for ConfigAudit {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change audit log settings. A channel must be provided for channel.".to_string()),
            usage: Some("<enable|disable|channel> <channel_resolvable>".to_string()),
            examples: vec!["channel #audit-logs".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
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
                    match parse_channel(val.to_string(), guild_id, &ctx.cache) {
                        Some((channel_id, channel)) => {
                            guild_data.audit_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name(), channel_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that channel.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "threshold" => {
                    match val.parse::<i16>() {
                        Ok(th) => {
                            guild_data.audit_threshold = th;
                            val = th.to_string();
                        },
                        Err(_) => { ctx.http.create_message(message.channel_id).reply(message.id).content("Please input a number as the threshold")?.await?; }
                    }
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `threshold`. For more information see `help config audit`")?.await?;
                    return Ok(())
                },
            }
            let guild = ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Audit Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { format!("{}", guild.audit) } else { val },
                ))
                .build()?;
            
            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigModlog;
#[async_trait]
impl Command for ConfigModlog {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change moderation log settings. A channel must be provided for channel.".to_string()),
            usage: Some("<enable|disable|channel> <channel_resolvable>".to_string()),
            examples: vec!["channel #mod-logs".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
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
                    match parse_channel(val.to_string(), guild_id, &ctx.cache) {
                        Some((channel_id, channel)) => {
                            guild_data.modlog_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name(), channel_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that channel.")?.await?;
                            return Ok(())
                        },
                    }
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`. For more information see `help config modlog`")?.await?;
                    return Ok(())
                },
            }
            let guild = ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Modlog Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { guild.modlog.to_string() } else { val },
                ))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigWelcome;
#[async_trait]
impl Command for ConfigWelcome {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change welcome message settings.\nOption is one of enable, disable, channel, message, type and the respective values should be none, none, channel_resolvable, desired message.\nType designates if the message is plain or embed. Anything other than embed will result in plain.".to_string()),
            usage: Some("<option> <value>".to_string()),
            examples: vec!["message Welcome to {guild}, {user}!".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
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
                    match parse_channel(val.to_string(), guild_id, &ctx.cache) {
                        Some((channel_id, channel)) => {
                            guild_data.welcome_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name(), channel_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that channel.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "message" => {
                    guild_data.welcome_message = val.to_string();
                },
                "type" => {
                    guild_data.welcome_type = val.to_string();
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config welcome`")?.await?;
                    return Ok(())
                },
            }
            let guild = ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Welcome Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { guild.welcome.to_string() } else { val },
                ))
                .build()?;
            
            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigIntroduction;
#[async_trait]
impl Command for ConfigIntroduction {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change introduction message settings. This is exactly like welcome: `help config welcome` for more info. This is a premium only feature related to the Register command.".to_string()),
            usage: Some("<option> <value>".to_string()),
            examples: vec!["message Hey there {user}, mind introducting yourself?".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
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
                    match parse_channel(val.to_string(), guild_id, &ctx.cache) {
                        Some((channel_id, channel)) => {
                            guild_data.introduction_channel = channel_id.0 as i64;
                            val = format!("{} ({})", channel.name(), channel_id.0);
                        },
                        None => {
                            ctx.http.create_message(message.channel_id).reply(message.id).content("I couldn't find that channel.")?.await?;
                            return Ok(())
                        },
                    }
                },
                "message" => {
                    guild_data.introduction_message = val.to_string();
                },
                "type" => {
                    guild_data.introduction_type = val.to_string();
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`, `channel`, `message`, `type`. For more information see `help config introduction`")?.await?;
                    return Ok(())
                },
            }
            let guild = ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Introduction Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    if val.is_empty() { guild.introduction.to_string() } else { val },
                ))
                .build()?;
            
            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigCommands;
#[async_trait]
impl Command for ConfigCommands {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change which commands are disabled. A command name must be provided.".to_string()),
            usage: Some("<enable|disable> <command_name>".to_string()),
            examples: vec!["disable e621".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.commands.retain(|e| *e != val);
                },
                "disable" => {
                    if !val.starts_with("conf") {
                        guild_data.commands.push(val.clone());
                    } else {
                        ctx.http.create_message(message.channel_id).reply(message.id).content("Config commands cannot be disabled.")?.await?;
                        return Ok(());
                    }
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config command`")?.await?;
                    return Ok(())
                },
            }
            ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Command Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

pub struct ConfigLogs;
#[async_trait]
impl Command for ConfigLogs {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Change which log messages are disabled. A log type must be provided.".to_string()),
            usage: Some("<enable|disable|types> [type]".to_string()),
            examples: vec!["disable message_edit".to_string()],
            required_permissions: Permissions::MANAGE_GUILD,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let mut guild_data = ctx.db.get_guild(guild_id.0 as i64)?;
            let op = args.single::<String>().unwrap_or(String::new());
            let val = args.rest().to_string();
            match op.to_lowercase().as_str() {
                "enable" => {
                    guild_data.logging.retain(|e| *e != val);
                },
                "disable" => {
                    if LOG_TYPES.contains(&val.as_str()) {
                        guild_data.logging.push(val.clone());
                    } else {
                        ctx.http.create_message(message.channel_id).reply(message.id).content("Invalid log type. See `config log types` for valid types.")?.await?;
                        return Ok(());
                    }
                },
                "types" => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content(LOG_TYPES.iter()
                        .map(|e| format!("`{}`", e))
                        .collect::<Vec<String>>()
                        .join(", "))?
                        .await?;
                    return Ok(());
                },
                _ => {
                    ctx.http.create_message(message.channel_id).reply(message.id).content("I didn't understand that option. Valid options are: `enable`, `disable`. For more information see `help config log`")?.await?;
                    return Ok(())
                },
            }
            ctx.db.update_guild(guild_id.0 as i64, guild_data)?;
            
            let embed = EmbedBuilder::new()
                .title("Config Log Summary")
                .color(colors::MAIN)
                .description(format!("**Operation:** {}\n**Value:** {}",
                    op,
                    val,
                ))
                .build()?;

            ctx.http.create_message(message.channel_id).reply(message.id).embed(embed)?.await?;
        } else {
            debug!("{}", GUILDID_FAIL);
        }
        Ok(())
    }
}

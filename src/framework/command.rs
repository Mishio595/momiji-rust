use crate::Context;
use crate::core::consts::colors;
use std::collections::HashMap;
use std::{fmt, fmt::{Debug, Formatter}};
use std::error::Error as StdError;
use std::sync::Arc;
use super::args::Args;
use tracing::{event, Level};
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

pub(crate) type InternalCommand = Arc<dyn Command>;

#[derive(Clone, Debug)]
pub struct Error(pub String);

// TODO: Have separate `From<(&)String>` and `From<&str>` impls via specialization
impl<D: fmt::Display> From<D> for Error {
    fn from(d: D) -> Self {
        Error(d.to_string())
    }
}

#[derive(Clone)]
pub struct Help(pub HashMap<String, Arc<Module>>, pub Arc<HelpOptions>);

impl Debug for Help {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Help")
            .field("options", &self.1)
            .finish()
    }
}

impl Help {
    fn individual_help(&self, input: &String, cmd: Arc<dyn Command>, module: Arc<Module>) -> Option<EmbedBuilder> {
        let options = &self.1;

        let cmd_options = cmd.options();
        let mut aliases = Vec::new();
        let mut name = input.clone();
        
        for (k, v) in module.commands.iter() {
            match v {
                CommandOrAlias::Alias(alias) => {
                    if alias == &input.to_lowercase() {
                        aliases.push(k);
                    }
                },
                CommandOrAlias::Command(_) => {
                    if k == &input.to_lowercase() {
                        name = k.clone();
                    }
                },
            }
        }

        if !cmd_options.help_available { return None }

        let mut embed = EmbedBuilder::new()
            .title(name.clone())
            .color(colors::MAIN);

        if let Some(description) = &cmd_options.description {
            embed = embed.field(EmbedFieldBuilder::new(options.description_label.clone(), description).build());
        }

        let restrictions = format!("Guild Only: {}\nOwner Only: {}",
            cmd_options.guild_only,
            cmd_options.owner_only);
        embed = embed.field(EmbedFieldBuilder::new(options.restrictions_label.clone(), restrictions).inline().build());

        if !cmd_options.required_permissions.is_empty() {
            embed = embed.field(EmbedFieldBuilder::new(options.required_permissions_label.clone(), format!("{:?}", cmd_options.required_permissions)).inline().build());
        }

        if let Some(usage) = &cmd_options.usage {
            embed = embed.field(EmbedFieldBuilder::new(options.usage_label.clone(), usage).build());
        }

        
        if !aliases.is_empty() {
            let aliases = aliases.iter().map(|e| e.as_str()).collect::<Vec<&str>>().join(", ");
            embed = embed.field(EmbedFieldBuilder::new(options.aliases_label.clone(), aliases).build());
        }

        if !cmd_options.examples.is_empty() {
            let examples = format!("{} {}",
                name.clone(),
                cmd_options.examples.iter()
                    .map(|e| e.as_str()).collect::<Vec<&str>>()
                    .join(format!("\n{}", name).as_str())
            );
            embed = embed.field(EmbedFieldBuilder::new(options.examples_label.clone(), examples).build());
        }

        Some(embed)
    }
}

#[async_trait]
impl Command for Help {
    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let modules = &self.0;
        let options = &self.1;

        if let Ok(input) = args.single::<String>() {
            let mut found = false;

            for (module_name, module) in modules.iter() {
                // TODO add module prefix catching
                if let Some(ref prefix) = module.prefix {
                    if prefix == &input.to_lowercase() {
                        if let Ok(subcmd) = args.single::<String>() {
                            if let Some(cmd) = super::command_crawl(subcmd, module) {
                                if let Some(embed) = self.individual_help(&input, cmd, module.clone()) {
                                    ctx.http.create_message(message.channel_id)
                                        .reply(message.id)
                                        .embed(embed.build()?)?
                                        .await?;

                                    found = true;
                                }
                            }
                        } else {
                            //module matches, no subcommand
                            let mut commands: Vec<&str> = module.commands.iter()
                                .filter_map(|(name, cmd)| { match cmd {
                                    CommandOrAlias::Command(_) => Some(name.as_str()),
                                    CommandOrAlias::Alias(_) => None
                                }}).collect();
                            commands.sort();

                            let embed = EmbedBuilder::new()
                                .title(module_name.clone())
                                .color(colors::MAIN)
                                .field(EmbedFieldBuilder::new("Sub-commands", commands.join("\n ")).build())
                                .build()?;

                            ctx.http.create_message(message.channel_id).reply(message.id)
                                .embed(embed)?
                                .await?;

                            found = true;
                        }
                    }
                }
                if let Some(cmd) = super::command_crawl(input.clone(), module) {
                    if let Some(embed) = self.individual_help(&input, cmd, module.clone()) {
                        ctx.http.create_message(message.channel_id)
                            .reply(message.id)
                            .embed(embed.build()?)?
                            .await?;

                        found = true;
                    }
                }
            }

            if !found {
                ctx.http.create_message(message.channel_id)
                    .reply(message.id)
                    .content(format!("**Error**: Command `{}` not found.", input))?
                    .await?;
            }
        } else {
            let mut embed = EmbedBuilder::new()
                .description(options.individual_command_tip.clone())
                .color(colors::MAIN);

            for (name, module) in modules.iter() {
                let name = if let Some(ref prefix) = module.prefix {
                    format!("{} (prefix: `{}`)", name, prefix)
                } else { name.clone() };

                let mut commands: Vec<&str> = module.commands.iter()
                    .filter_map(|(k, v)|  match v {
                        CommandOrAlias::Command(cmd) => { if cmd.options().help_available {
                            Some(k.as_str()) } else { None }
                        }
                        CommandOrAlias::Alias(_) => { None }
                    }).collect();
                commands.sort();
                
                if commands.is_empty() { continue; }

                let field = EmbedFieldBuilder::new(name, format!("`{}`", commands.join("`, `")));

                embed = embed.field(field);
            }

            ctx.http.create_message(message.channel_id)
                .reply(message.id)
                .embed(embed.build()?)?
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct HelpOptions {
    pub suggestion_text: String,
    pub no_help_available_text: String,
    pub usage_label: String,
    pub examples_label: String,
    pub description_label: String,
    pub aliases_label: String,
    pub guild_only_text: String,
    pub available_text: String,
    pub command_not_found_text: String,
    pub individual_command_tip: String,
    pub striked_commands_tip_in_dm: Option<String>,
    pub striked_commands_tip_in_guild: Option<String>,
    pub group_prefix: String,
    pub restrictions_label: String,
    pub required_permissions_label: String,
    // pub lacking_role: HelpBehaviour,
    // pub lacking_permissions: HelpBehaviour,
    // pub wrong_channel: HelpBehaviour,
    // pub embed_error_colour: Colour,
    // pub embed_success_colour: Colour,
    pub max_levenshtein_distance: usize,
}

impl Default for HelpOptions {
    fn default() -> HelpOptions {
        HelpOptions {
            suggestion_text: "Did you mean `{}`?".to_string(),
            no_help_available_text: "**Error**: No help available.".to_string(),
            usage_label: "Usage".to_string(),
            examples_label: "Examples".to_string(),
            aliases_label: "Aliases".to_string(),
            description_label: "Description".to_string(),
            guild_only_text: "Only in guilds".to_string(),
            restrictions_label: "Restrictions".to_string(),
            required_permissions_label: "Required Permissions".to_string(),
            available_text: "Available".to_string(),
            command_not_found_text: "**Error**: Command `{}` not found.".to_string(),
            individual_command_tip: "To get help with an individual command, pass its \
                 name as an argument to this command.".to_string(),
            group_prefix: "Prefix".to_string(),
            striked_commands_tip_in_dm: Some(String::new()),
            striked_commands_tip_in_guild: Some(String::new()),
            // lacking_role: HelpBehaviour::Strike,
            // lacking_permissions: HelpBehaviour::Strike,
            // wrong_channel: HelpBehaviour::Strike,
            // embed_error_colour: Colour::DARK_RED,
            // embed_success_colour: Colour::ROSEWATER,
            max_levenshtein_distance: 0,
        }
    }
}

pub enum CommandOrAlias {
    Alias(String),
    Command(InternalCommand),
}

impl Debug for CommandOrAlias {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CommandOrAlias::Alias(ref s) => f.debug_tuple("CommandOrAlias::Alias").field(&s).finish(),
            CommandOrAlias::Command(ref arc) => f.debug_tuple("CommandOrAlias::Command").field(&arc.options()).finish(),
        }
    }
}

//TODO Complete module implementaion
#[derive(Debug)]
pub struct ModuleBuilder {
    pub prefix: Option<String>,
    pub commands: HashMap<String, CommandOrAlias>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub guild_only: bool,
    pub owners_only: bool,
    // pub help: Option<Arc<Help>>,
    pub default_command: Option<CommandOrAlias>,
    pub description: Option<String>,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self {
            prefix: None,
            commands: HashMap::new(),
            required_permissions: Permissions::empty(),
            guild_only: false,
            help_available: true,
            owners_only: false,
            // help: None,
            default_command: None,
            description: None,
        }
    }
}

impl ModuleBuilder {
    pub fn build(self) -> Module {
        Module {
            prefix: self.prefix,
            commands: self.commands,
            required_permissions: self.required_permissions,
            guild_only: self.guild_only,
            help_available: self.help_available,
            owners_only: self.owners_only,
            // help: self.help,
            default_command: self.default_command,
            description: self.description,
        }
    }

    pub fn prefix<S: ToString>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.to_string());

        self
    }

    pub fn add_command<S: ToString>(mut self, name: S, command: CommandOrAlias) -> Self {
        self.commands.insert(name.to_string(), command);

        self
    }

    pub fn required_permissions(mut self, p: Permissions) -> Self {
        self.required_permissions = p;

        self
    }

    pub fn guild_only(mut self, b: bool) -> Self {
        self.guild_only = b;

        self
    }

    pub fn help_available(mut self, b: bool) -> Self {
        self.help_available = b;

        self
    }

    pub fn owners_only(mut self, b: bool) -> Self {
        self.owners_only = b;

        self
    }

    pub fn default_command(mut self, c: CommandOrAlias) -> Self {
        self.default_command = Some(c);

        self
    }

    pub fn description(mut self, d: String) -> Self {
        self.description = Some(d);

        self
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Module {
    pub prefix: Option<String>,
    pub commands: HashMap<String, CommandOrAlias>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub guild_only: bool,
    pub owners_only: bool,
    // pub help: Option<Arc<Help>>,
    pub default_command: Option<CommandOrAlias>,
    pub description: Option<String>,
}

impl Module {
    pub(crate) fn builder() -> ModuleBuilder {
        ModuleBuilder::default()
    }
}

#[derive(Debug)]
pub struct Options {
    pub description: Option<String>,
    pub usage: Option<String>,
    pub examples: Vec<String>,
    pub required_permissions: Permissions,
    pub guild_only: bool,
    pub owner_only: bool,
    pub help_available: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            description: None,
            usage: None,
            examples: Vec::new(),
            required_permissions: Permissions::empty(),
            guild_only: false,
            owner_only: false,
            help_available: true,
        }
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_OPTIONS: Arc<Options> = Arc::new(Options::default());
}

#[async_trait]
pub trait Command: Send + Sync + 'static {
    async fn run(&self, message: Message, args: Args, ctx: Context)
        -> Result<(), Box<dyn StdError + Send + Sync>>;

    fn options(&self) -> Arc<Options> {
        Arc::clone(&DEFAULT_OPTIONS)
    }

    fn before(&self, _: Message, _: Args, _: Context) -> bool { true }

    fn after(&self, _: Message, _: Args, _: Context, _: Box<dyn StdError + Send + Sync>) {}
}

#[async_trait]
impl Command for Arc<dyn Command> {
    async fn run(&self, m: Message, a: Args, ctx: Context)
        -> Result<(), Box<dyn StdError + Send + Sync>> {
            (**self).run(m, a, ctx).await
        }
    
    fn options(&self) -> Arc<Options> {
        (**self).options()
    }

    fn before(&self, m: Message, a: Args, ctx: Context) -> bool {
        (**self).before(m, a, ctx)
    }

    fn after(&self, m: Message, a: Args, ctx: Context, e: Box<dyn StdError + Send + Sync>) {
        (**self).after(m, a, ctx, e)
    }
}

impl std::fmt::Debug for dyn Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Command {:?}", self.options()))
    }
}
use crate::core::timers::TimerClient;
use crate::db::DatabaseConnection;
use std::collections::HashMap;
use std::{fmt, fmt::{Debug, Formatter}};
use std::error::Error as StdError;
use std::sync::Arc;
use super::args::Args;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;

pub(crate) type InternalCommand = Arc<dyn Command>;
pub type HelpFunction = fn(&Message, &HelpOptions, HashMap<String, Arc<Module>>, &Args)
                   -> Result<(), Error>;

pub struct Help(pub HelpFunction, pub Arc<HelpOptions>);

#[derive(Clone, Debug)]
pub struct Error(pub String);

// TODO: Have separate `From<(&)String>` and `From<&str>` impls via specialization
impl<D: fmt::Display> From<D> for Error {
    fn from(d: D) -> Self {
        Error(d.to_string())
    }
}

impl Debug for Help {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Help")
            .field("options", &self.1)
            .finish()
    }
}

impl HelpCommand for Help {
    fn execute(&self, m: &Message, ho: &HelpOptions,hm: HashMap<String, Arc<Module>>, a: &Args) -> Result<(), Error> {
        (self.0)(m, ho, hm, a)
    }
}

#[derive(Debug)]
pub struct HelpOptions {
    pub suggestion_text: String,
    pub no_help_available_text: String,
    pub usage_label: String,
    pub usage_sample_label: String,
    pub ungrouped_label: String,
    pub description_label: String,
    pub grouped_label: String,
    pub aliases_label: String,
    pub guild_only_text: String,
    pub dm_only_text: String,
    pub dm_and_guild_text: String,
    pub available_text: String,
    pub command_not_found_text: String,
    pub individual_command_tip: String,
    pub striked_commands_tip_in_dm: Option<String>,
    pub striked_commands_tip_in_guild: Option<String>,
    pub group_prefix: String,
    // pub lacking_role: HelpBehaviour,
    // pub lacking_permissions: HelpBehaviour,
    // pub wrong_channel: HelpBehaviour,
    // pub embed_error_colour: Colour,
    // pub embed_success_colour: Colour,
    pub max_levenshtein_distance: usize,
}

pub trait HelpCommand: Send + Sync + 'static {
    fn execute(&self, _: &Message, _: &HelpOptions, _: HashMap<String, Arc<Module>>, _: &Args) -> Result<(), Error>;

    fn options(&self) -> Arc<Options> {
        Arc::clone(&DEFAULT_OPTIONS)
    }
}

impl HelpCommand for Arc<dyn HelpCommand> {
    fn execute(&self, m: &Message, ho: &HelpOptions, hm: HashMap<String, Arc<Module>>, a: &Args) -> Result<(), Error> {
        (**self).execute(m, ho, hm, a)
    }
}

impl Default for HelpOptions {
    fn default() -> HelpOptions {
        HelpOptions {
            suggestion_text: "Did you mean `{}`?".to_string(),
            no_help_available_text: "**Error**: No help available.".to_string(),
            usage_label: "Usage".to_string(),
            usage_sample_label: "Sample usage".to_string(),
            ungrouped_label: "Ungrouped".to_string(),
            grouped_label: "Group".to_string(),
            aliases_label: "Aliases".to_string(),
            description_label: "Description".to_string(),
            guild_only_text: "Only in guilds".to_string(),
            dm_only_text: "Only in DM".to_string(),
            dm_and_guild_text: "In DM and guilds".to_string(),
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
        }
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_OPTIONS: Arc<Options> = Arc::new(Options::default());
}

#[async_trait]
pub trait Command: Send + Sync + 'static {
    async fn run(&self, message: Message, args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, timers: TimerClient)
        -> Result<(), Box<dyn StdError + Send + Sync>>;

    fn options(&self) -> Arc<Options> {
        Arc::clone(&DEFAULT_OPTIONS)
    }

    fn before(&self, _: Message, _: Args, _: HttpClient, _: InMemoryCache, _: DatabaseConnection) -> bool { true }

    fn after(&self, _: Message, _: Args, _: HttpClient, _: InMemoryCache, _: DatabaseConnection, _: Box<dyn StdError + Send + Sync>) {}
}

#[async_trait]
impl Command for Arc<dyn Command> {
    async fn run(&self, m: Message, a: Args, h: HttpClient , c: InMemoryCache , d: DatabaseConnection, t: TimerClient)
        -> Result<(), Box<dyn StdError + Send + Sync>> {
            (**self).run(m, a, h, c, d, t).await
        }
    
    fn options(&self) -> Arc<Options> {
        (**self).options()
    }

    fn before(&self, m: Message, a: Args, h: HttpClient , c: InMemoryCache , d: DatabaseConnection) -> bool {
        (**self).before(m, a, h, c, d)
    }

    fn after(&self, m: Message, a: Args, h: HttpClient , c: InMemoryCache , d: DatabaseConnection, e: Box<dyn StdError + Send + Sync>) {
        (**self).after(m, a, h, c, d, e)
    }
}

impl std::fmt::Debug for dyn Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Command {:?}", self.options()))
    }
}
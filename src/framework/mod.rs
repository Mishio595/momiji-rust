pub mod args;
pub mod command;
pub mod parser;

use crate::Context;
use self::command::{Help, HelpOptions};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use self::args::Args;
use self::command::{
    CommandOrAlias::*,
    Command as CommandTrait,
    Module,
    ModuleBuilder
};
use tracing::{event, Level};
use twilight_model::{channel::Message, guild::Permissions};

#[derive(Debug)]
pub enum DispatchError {
    InsufficientPermissions(Permissions),
    InvalidChannelType,
    OwnerOnly,
    FailedCheck,
}

impl Error for DispatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DispatchError::*;

        match *self {
            InsufficientPermissions(ref p) => write!(f, "Insufficient Permissions: {:?}", p),
            InvalidChannelType => write!(f, "Invalid Channel Type"),
            OwnerOnly => write!(f, "Must be bot owner to execute"),
            FailedCheck => write!(f, "Failed Check:"),
        }
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    case_sensitive: bool,
    delimiters: Vec<String>,
    dynamic_prefix: Arc<dyn Fn(&Message, Context) -> Option<String> + Send + Sync>,
    ignore_bots: bool,
    on_dm: bool,
    on_mention: bool,
    prefix: String,
    before: Arc<dyn Fn(&Message, &str, Context) -> bool + Send + Sync>,
    after: Arc<dyn Fn(&Message, &str, Context,  Box<dyn Error>) + Send + Sync>,
}

#[derive(Clone)]
#[non_exhaustive]
pub struct ConfigBuilder {
    case_sensitive: bool,
    delimiters: Vec<String>,
    dynamic_prefix: Arc<dyn Fn(&Message, Context) -> Option<String> + Send + Sync>,
    ignore_bots: bool,
    on_dm: bool,
    on_mention: bool,
    prefix: String,
    before: Arc<dyn Fn(&Message, &str, Context) -> bool + Send + Sync>,
    after: Arc<dyn Fn(&Message, &str, Context,  Box<dyn Error>) + Send + Sync>,
}

impl Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder { 
            case_sensitive: false,
            delimiters: vec![",", " "].iter().map(|e| e.to_string()).collect(),
            dynamic_prefix: Arc::new(|_,_| None),
            ignore_bots: true,
            on_dm: true,
            on_mention: true,
            prefix: "m!".to_string(),
            before: Arc::new(|_,_,_| true),
            after: Arc::new(|_,_,_,_| {}),
        }
    }
}

impl ConfigBuilder {
    pub fn build(&self) -> Config {
        self.into()
    }
    
    pub fn case_sensitive(&mut self, b: bool) -> &mut Self {
        self.ignore_bots = b;

        self
    }

    pub fn delimiters<S: ToString>(&mut self, d: Vec<S>) -> &mut Self {
        self.delimiters = d.iter().map(|e| e.to_string()).collect();

        self
    }

    pub fn dynamic_prefix<F>(&mut self, f: F) -> &mut Self
        where F: Fn(&Message, Context) -> Option<String> + Send + Sync + 'static {
        self.dynamic_prefix = Arc::new(f);

        self
    }

    pub fn ignore_bots(&mut self, b: bool) -> &mut Self {
        self.ignore_bots = b;

        self
    }

    pub fn on_dm(&mut self, b: bool) -> &mut Self {
        self.on_dm = b;

        self
    }

    pub fn on_mention(&mut self, b: bool) -> &mut Self {
        self.on_mention = b;

        self
    }

    pub fn prefix<S: ToString>(&mut self, p: S) -> &mut Self {
        self.prefix = p.to_string();

        self
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn new() -> Self {
        ConfigBuilder::default().build()
    }
}

impl From<&ConfigBuilder> for Config {
    fn from(builder: &ConfigBuilder) -> Self {
        Self {
            case_sensitive: builder.case_sensitive,
            delimiters: builder.delimiters.clone(),
            dynamic_prefix: builder.dynamic_prefix.clone(),
            ignore_bots: builder.ignore_bots,
            on_dm: builder.on_dm,
            on_mention: builder.on_mention,
            prefix: builder.prefix.clone(),
            before: builder.before.clone(),
            after: builder.after.clone(),
        }
    }
}

pub struct FrameworkBuilder {
    config: Config,
    modules: HashMap<String, Arc<Module>>,
    help_options: HelpOptions,
}

impl FrameworkBuilder {
    pub fn build(mut self) -> Framework {
        let help_command = Help(self.modules.clone(), Arc::new(self.help_options));
        let help_module = Module::builder()
            .add_command("help", Command(Arc::new(help_command)))
            .build();
        self.modules.insert("Help Command".to_string(), Arc::new(help_module));

        Framework {
            config: self.config,
            modules: self.modules,
        }
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;

        self
    }

    pub fn raw_add_module<S: ToString>(mut self, name: S, module: Module) -> Self {
        self.modules.insert(name.to_string(), Arc::new(module));

        self
    }

    pub fn add_module<F, S>(self, name: S, builder: F) -> Self
            where F: FnOnce(ModuleBuilder) -> ModuleBuilder,
            S: ToString {
        self.raw_add_module(name, builder(Module::builder()).build())
    }
}

impl Default for FrameworkBuilder {
    fn default() -> Self {
        Self {
            config: Config::new(),
            modules: HashMap::new(),
            help_options: HelpOptions::default(),
        }
    }
}

#[non_exhaustive]
pub struct Framework {
    //TODO add before/after checks for all commands. Dispatch errors?
    config: Config,
    modules: HashMap<String, Arc<Module>>,
}

impl Framework {
    pub fn builder() -> FrameworkBuilder {
        FrameworkBuilder::default()
    }

    pub async fn handle_command(&self, message: Message, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if message.content.is_empty() { return Ok(()) }

        let prefix = match (*self.config.dynamic_prefix)(&message, ctx.clone()) {
            Some(p) => p,
            None => self.config.prefix.clone(),
        };
        //TODO add before and after hooks, add dispatch error hook
        if let Some((command, args)) = ctx.parser.parse_with_prefix(prefix.as_str(), message.content.as_str(), &self.config.delimiters[..]) {
            if let Some((c, args)) = get_command(&self.modules, command, args) {
                self.execute_command_with_hooks(c, message, args, ctx).await?;
            }
        } else if self.config.on_mention {
            event!(Level::DEBUG, "Trying self mention");
            let mention = format!("<@!{}>", ctx.user.id.0);
            if let Some((command, args)) = ctx.parser.parse_with_prefix(mention.as_str(), message.content.as_str(), &self.config.delimiters[..]) {
                if let Some((c, args)) = get_command(&self.modules, command, args) {
                    self.execute_command_with_hooks(c, message, args, ctx).await?;
                }
            }
        }

        Ok(())
    }

    async fn execute_command_with_hooks(&self, comm: Arc<dyn CommandTrait>, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let options = comm.options();
    
        if options.guild_only && message.guild_id.is_none() {
            return Err(Box::new(DispatchError::InvalidChannelType));
        }
    
        if !ctx.owners.contains_key(&message.author.id) {
            if let Some(m) = &message.member {
                let p = m.roles.iter().fold(Permissions::empty(), |p, r| {
                    ctx.cache.role(*r)
                        .and_then(|r| Some((*r).permissions))
                        .unwrap_or(Permissions::empty()) | p
                });
        
                if !p.contains(Permissions::ADMINISTRATOR) && !p.contains(options.required_permissions) {
                    let mut mp = options.required_permissions.clone();
                    mp.remove(p);
                    return Err(Box::new(DispatchError::InsufficientPermissions(mp)));
                }
            }
        
            if options.owner_only {
                return Err(Box::new(DispatchError::OwnerOnly));
            }
        }
    
        if !(*comm).before(message.clone(), args.clone(), ctx.clone()) {
            return Err(Box::new(DispatchError::FailedCheck));
        }
    
        if let Err(err) = (*comm).run(message.clone(), args.clone(), ctx.clone()).await {
            (*comm).after(message.clone(), args.clone(), ctx.clone(), err);
        }
    
        Ok(())
    }
}

pub(crate) fn get_command(modules: &HashMap<String, Arc<Module>>, input: String, mut args: Args) -> Option<(Arc<dyn CommandTrait>, Args)> {
    for module in modules.values() {
        if let Some(module_prefix) = &module.prefix {
            if module_prefix == &input {
                match args.single::<String>() {
                    Ok(sub_comm) => {
                        return command_crawl(sub_comm.clone(), module)
                            .or_else(|| {
                                args.restore();
                                module.default_command.as_ref().and_then(|c_or_a| match c_or_a {
                                    Command(c) => Some(c.clone()),
                                    Alias(a) => command_crawl(a.clone(), module),
                                })
                            })
                            .zip(Some(args))
                    },
                    _ => {
                        return module.default_command.as_ref().and_then(|c_or_a| match c_or_a {
                            Command(c) => Some(c.clone()),
                            Alias(a) => command_crawl(a.clone(), module),
                        }).zip(Some(args))
                    },
                }
            }
        } else {
            let comm = command_crawl(input.clone(), module);
            if comm.is_some() { return comm.zip(Some(args)) }
        }
    }

    None
}

pub(crate) fn command_crawl(comm: String, module: &Module) -> Option<Arc<dyn CommandTrait>> {
    match module.commands.get(&comm) {
        None => None,
        Some(Command(c)) => Some(c.clone()),
        Some(Alias(a)) => command_crawl(a.clone(), module),
    }
}
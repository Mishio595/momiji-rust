pub mod args;
pub mod command;
pub mod parser;

use crate::Context;
use std::collections::{HashMap, HashSet};
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
use twilight_model::{channel::Message, guild::Permissions};
use twilight_model::id::UserId;

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
    owners: HashSet<UserId>,
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
    owners: HashSet<UserId>,
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
            owners: HashSet::new(),
            prefix: "!".to_string(),
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

    pub fn owners(&mut self, owners: HashSet<UserId>) -> &mut Self {
        self.owners = owners;

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
            owners: builder.owners.clone(),
            prefix: builder.prefix.clone(),
            before: builder.before.clone(),
            after: builder.after.clone(),
        }
    }
}

pub struct FrameworkBuilder {
    config: Config,
    modules: HashMap<String, Module>,
}

impl FrameworkBuilder {
    pub fn build(self) -> Framework {
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
        self.modules.insert(name.to_string(), module);

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
        }
    }
}

#[non_exhaustive]
pub struct Framework {
    //TODO add before/after checks for all commands. Dispatch errors?
    //TODO help command?
    config: Config,
    modules: HashMap<String, Module>,
}

impl Framework {
    pub fn builder() -> FrameworkBuilder {
        FrameworkBuilder::default()
    }

    pub async fn handle_command(&self, message: Message, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = match (*self.config.dynamic_prefix)(&message, ctx.clone()) {
            Some(p) => p,
            None => self.config.prefix.clone(),
        };
        //TODO add before and after hooks, add dispatch error hook
        if let Some((command, args)) = ctx.parser.parse_with_prefix(prefix.as_str(), message.content.as_str(), &self.config.delimiters[..]) {
            let res= self.get_command(command, args);
            if let Some((c, args)) = res {
                self.execute_command_with_hooks(c, message, args, ctx).await?;
            }
        };

        Ok(())
    }

    fn get_command(&self, c: String, mut a: Args) -> Option<(Arc<dyn CommandTrait>, Args)> {
        for (_, module) in self.modules.iter() {
            if let Some(module_prefix) = &module.prefix {
                if module_prefix == &c {
                    match a.single::<String>() {
                        Ok(sub_comm) => {
                            return command_crawl(sub_comm.clone(), module)
                                .or_else(|| {
                                    a.restore();
                                    module.default_command.as_ref().and_then(|c_or_a| match c_or_a {
                                        Command(c) => Some(c.clone()),
                                        Alias(a) => command_crawl(a.clone(), module),
                                    })
                                })
                                .zip(Some(a))
                        },
                        _ => {
                            return module.default_command.as_ref().and_then(|c_or_a| match c_or_a {
                                Command(c) => Some(c.clone()),
                                Alias(a) => command_crawl(a.clone(), module),
                            }).zip(Some(a))
                        },
                    }
                }
            } else {
                let comm = command_crawl(c.clone(), module);
                if comm.is_some() { return comm.zip(Some(a)) }
            }
        }

        None
    }

    async fn execute_command_with_hooks(&self, comm: Arc<dyn CommandTrait>, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let options = comm.options();
    
        if options.guild_only && message.guild_id.is_none() {
            return Err(Box::new(DispatchError::InvalidChannelType));
        }
    
        if !self.config.owners.contains(&message.author.id) {
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

fn command_crawl(comm: String, module: &Module) -> Option<Arc<dyn CommandTrait>> {
    match module.commands.get(&comm) {
        None => None,
        Some(Command(c)) => Some(c.clone()),
        Some(Alias(a)) => command_crawl(a.clone(), module),
    }
}
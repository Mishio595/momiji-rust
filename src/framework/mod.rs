use std::collections::HashMap;
use std::collections::HashSet;
use serenity::framework::Framework;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::model::id::*;
use threadpool::ThreadPool;

pub mod command;
mod config;

use self::config::Config;
use self::command::Command;

pub struct MomijiFramework {
    config: Config,
    commands: HashMap<String, Command>,
}

impl Framework for MomijiFramework {
    // Required by Serenity
    // Called when discord sends a MESSAGE_CREATE event
    // TODO Expand logic
    fn dispatch(&mut self, ctx: Context, message: Message, threadpool: &ThreadPool) {
        let start = &message.content.split_whitespace().next().unwrap_or_else(|| "");
        let mut name = "";
        if !start.is_empty() {
            name = &start[*&self.config.prefix.len()..]; // Fix index out of bounds error
        }
        let default = Command::default();
        let command = &self.commands.get(name).unwrap_or(&default);
        if !&command.name.is_empty() {
            let res = &command.execute(&message, &"".to_string());
        }
    }
}

impl MomijiFramework {
    // Create a new framework for commands
    // Returns the framework to be consumed by other method
    pub fn new() -> MomijiFramework {
        let commands: HashMap<String, Command> = HashMap::new();
        let config = Config {
            use_mention: false,
            prefix: "!".to_string(),
            owners: HashSet::new(),
        };

        MomijiFramework {
            config,
            commands,
        }
    }
    
    // Add a command to the framework
    pub fn command(mut self, name: String, cmd: Command) -> MomijiFramework {
        self.commands.insert(name, cmd);
        self
    }

    // Set the framework to accept a mention in place of the prefix
    pub fn use_mention(mut self, state: bool) -> MomijiFramework {
        self.config.use_mention = state;
        self
    }

    // Set the owners of the bot. This allows you to set any number in a vector
    pub fn owners(mut self, owners: HashSet<UserId>) -> MomijiFramework {
        self.config.owners = owners;
        self
    }
}

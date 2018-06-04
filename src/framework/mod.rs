//! Provides MomijiFramework which is the core of command handling in Momiji. Each struct uses
//! simple builder syntax to create a more complex set of commands and options.

// Bunches of dependencies
// Make sure you do `extern crate serenity;` in your main.rs
use std::collections::HashMap;
use serenity::framework::Framework;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::prelude::*;
use threadpool::ThreadPool;
use super::db::Database;

// We use these internally
pub mod command;
mod config;

// Bring some shit from our mods into scope for easy access
use self::config::Config;
use self::command::Command;

pub struct MomijiFramework {
    config: Config,
    commands: HashMap<String, Command>,
    database: Database,
}

/// Implement serenity's framework trait and the required dispatch method
/// This is where the logic happens
impl Framework for MomijiFramework {
    /// Required by Serenity
    /// Called when discord sends a MESSAGE_CREATE event
    // TODO Expand logic
    fn dispatch(&mut self, _ctx: Context, message: Message, _threadpool: &ThreadPool) {
        // Find the first word or "" if nothing
        let start = &message.content.split_whitespace().next().unwrap_or_else(|| "");
        let mut name = "";
        if !start.is_empty() {
            // Strip the prefix
            name = &start[self.config.pre.len()..];
        }
        let default = Command::default();
        let command = self.commands.get(name).unwrap_or(&default);
        if !command.name.is_empty() {
            if get_rank(&message.author) >= command.rank {
                let _ = command.execute(&message, "");
            }
        }
    }
}

// More methods here
impl MomijiFramework {
    /// Create a new framework for commands
    /// Returns the framework to be consumed by other method
    pub fn new() -> MomijiFramework {
        let commands: HashMap<String, Command> = HashMap::new();
        let config = Config::new();
        let database = Database::connect();

        MomijiFramework {
            config,
            commands,
            database,
        }
    }

    /// Add a command to the framework. See Command::new() for more details.
    pub fn command<S>(mut self, name: S, f: (fn(&Message, String) -> bool), rank: u8, server: bool) -> MomijiFramework
    where S: Into<String> + Copy
    {
        let mut cmd = Command::new(name, f);
        cmd.set_rank(rank);
        cmd.server_only(server);
        self.commands.insert(name.into(), cmd);
        self
    }

    /// Build a config for the framework using a function or closure that consumes and returns self
    /// with each step
    pub fn configure<T>(mut self, config: T) -> MomijiFramework
    where T: FnOnce(Config) -> Config {
        let c = Config::new();
        let built = config(c);
        self.config = built;
        self
    }
}

fn get_rank(member: &User) -> u8 {
    0
}

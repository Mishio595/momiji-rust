//! `Framework::Command` is a builder-style struct for creating Discord commands within the
//! framework. This generally is not invoked directly and you will instead create commands with the
//! parent builder method `Framework.command`

use serenity::model::channel::Message;
use std::default::Default;

/// The basic outline of a command, this is mostly private, except for the name as nothing else is
/// accessed directly.
pub struct Command {
    pub name: String,
    action: (fn(&Message, String) -> bool),
    pub rank: u8,
    pub server: bool,
    pub description: String,
    pub usage: String,
}

impl Default for Command {
    /// Default is implemented for an "empty command"
    /// Used when unable to get the command from the containing hashmap
    /// Unique in that the command name is empty and the action does nothing
    fn default() -> Command {
        Command {
            name: String::new(),
            description: String::new(),
            usage: String::new(),
            action: |_,_| false,
            rank: 0,
            server: false,
        }
    }
}

impl Command {
    /// Build a new Command given a name and action
    /// Use other builder functions to configure
    pub fn new<S>(name: S, action: (fn(&Message, String) -> bool)) -> Command
    where S: Into<String>,
    {
        Command {
            name: name.into(),
            action,
            ..Command::default()
        }
    }

    /// Run the command!
    /// Pass in the message struct and any additional arguments as a string
    pub fn execute<S: Into<String>>(&self, m: &Message, str: S) -> bool {
        (self.action)(m, str.into())
    }

    /// Change the state of restrictions
    /// Will expand to be more flexible in the future
    pub fn set_rank(&mut self, level: u8) {
        self.rank = level;
    }

    /// Restrict the command to servers only
    pub fn server_only(&mut self, state: bool) {
        self.server = state;
    }
}

//! A collection of options for `momiji::framework` that allow you to easily set default options
//! using an easy builder syntax

use std::collections::HashSet;
use serenity::model::id::UserId;

pub struct Config {
    pub mention: bool,
    pub pre: String,
    pub own: HashSet<UserId>,
}

impl Config {
    /// Build a new Config
    /// While the Config struct is public, this method can be used for a quick default structure
    pub fn new() -> Config {
        Config {
            mention: false,
            pre: String::from("m!"),
            own: HashSet::new(),
        }
    }

    /// Tells the framework whether or not to consider a mention of the bot user as a valid prefix
    pub fn use_mention(mut self, s: bool) -> Config {
        self.mention = s;
        self
    }

    /// Set the default prefix of the bot given a String or string literal
    pub fn prefix<S: Into<String>>(mut self, p: S) -> Config {
        self.pre = p.into();
        self
    }

    /// Sets the owners of the bot. While the actual bot owner can easily be referenced, this
    /// allows for several owners to be set. Impacts Commands that use ranks
    pub fn owners(mut self, o: HashSet<UserId>) -> Config {
        self.own = o;
        self
    }
}

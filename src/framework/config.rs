use std::collections::HashSet;
use serenity::model::id::UserId;

pub struct Config {
    pub use_mention: bool,
    pub prefix: String,
    pub owners: HashSet<UserId>,
}


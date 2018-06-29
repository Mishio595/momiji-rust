use std::str::FromStr;
use serenity::model::id::*;
use regex;
use regex::Regex;

pub fn parse_role(input: String) -> Option<RoleId> {
    let re = Regex::new(r"(?:<@)?&?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
        Some(s) => Some(RoleId::from_str(&s[1]).unwrap()),
        None => { None },
    }
}

pub fn parse_user(input: String) -> Option<UserId> {
    let re = Regex::new(r"(?:<@)?!?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
        Some(s) => Some(UserId::from_str(&s[1]).unwrap()),
        None => { None },
    }
}

pub fn parse_channel(input: String) -> Option<ChannelId> {
    let re = Regex::new(r"(?:<#)?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
        Some(s) => Some(ChannelId::from_str(&s[1]).unwrap()),
        None => { None },
    }
}

pub fn parse_guild(input: String) -> Option<GuildId> {
    let re = Regex::new(r"(\d{17,})").unwrap();
    match re.captures(input.as_str()) {
        Some(s) => Some(GuildId(s[1].parse::<u64>().unwrap())),
        None => { None },
    }
}

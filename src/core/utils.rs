use core::consts::*;
use serenity::model::id::*;
use serenity::model::guild::{Guild, Role, Member};
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::misc::Mentionable;
use serenity::CACHE;
use serenity::Error;
use serenity::prelude::RwLock;
use regex::Regex;
use std::sync::Arc;
use std::str::FromStr;
use std::collections::HashMap;

lazy_static! {
    static ref ROLE_MATCH: Regex    = Regex::new(r"(?:<@)?&?(\d{17,})>*?").unwrap();
    static ref USER_MATCH: Regex    = Regex::new(r"(?:<@)?!?(\d{17,})>*?").unwrap();
    static ref CHANNEL_MATCH: Regex = Regex::new(r"(?:<#)?(\d{17,})>*?").unwrap();
    static ref GUILD_MATCH: Regex   = Regex::new(r"\d{17,}").unwrap();
    static ref EMBED_ITEM: Regex    = Regex::new(r"\$[^\$]*").unwrap();
    static ref EMEBED_PARTS: Regex  = Regex::new(r"\$?(?P<field>\S+):(?P<value>.*)").unwrap();
    static ref PLAIN_PARTS: Regex   = Regex::new(r"\{.*\}").unwrap();
    static ref SWITCH_REST: Regex   = Regex::new(r"^[^/]+").unwrap();
    static ref SWITCH_PARTS: Regex  = Regex::new(r"/\s*(\S+)([^/]*)").unwrap();
    static ref TIME: Regex          = Regex::new(r"(\d+)\s*?(\w)").unwrap();
}

/// Attempts to parse a role ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached roles
/// This method is case insensitive
/// # Panics
/// This method will panic if `guild` is not a valid, cached GuildId
pub fn parse_role(input: String, guild: GuildId) -> Option<(RoleId, Role)> {
    let cache = CACHE.read();
    match ROLE_MATCH.captures(input.as_str()) {
        Some(s) => {
            let id = RoleId::from_str(&s[1]).unwrap();
            let guild = cache.guilds.get(&guild).unwrap().read();
            let role = guild.roles.get(&id).unwrap();
            Some((id, role.clone()))
        },
        None => {
            for (id, role) in cache.guilds.get(&guild).unwrap().read().roles.iter() {
                if role.name.to_lowercase() == input.to_lowercase() {
                    return Some((*id, role.clone()));
                }
            }
            None
        },
    }
}

/// Attempts to parse a user ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached users
/// This method is case insensitive
/// # Panics
/// This method will panic if `guild` is not a valid, cached GuildId
pub fn parse_user(input: String, guild: GuildId) -> Option<(UserId, Member)> {
    let cache = CACHE.read();
    match USER_MATCH.captures(input.as_str()) {
        Some(s) => {
            let id = UserId::from_str(&s[1]).unwrap();
            let member = guild.member(id).unwrap();
            Some((id, member.clone()))
        },
        None => {
            for (id, member) in cache.guilds.get(&guild).unwrap().read().members.iter() {
                let user = member.user.read();
                if user.name.to_lowercase() == input.to_lowercase() || user.tag().to_lowercase() == input.to_lowercase() || member.display_name().to_lowercase() == input.to_lowercase() {
                    return Some((*id, member.clone()));
                }
            }
            None
        },
    }
}

/// Attempts to parse a channel ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached GuildChannels
/// This method is case insensitive
/// # Panics
/// This method will panic if `guild` is not a valid, cached GuildId
pub fn parse_channel(input: String, guild: GuildId) -> Option<(ChannelId, GuildChannel)> {
    let cache = CACHE.read();
    match CHANNEL_MATCH.captures(input.as_str()) {
        Some(s) => {
            let id = ChannelId::from_str(&s[1]).unwrap();
            let ch_lock = id.get().unwrap().guild().unwrap();
            let ch = ch_lock.read();
            Some((id, ch.clone()))
        },
        None => {
            for (id, ch_lock) in cache.guilds.get(&guild).unwrap().read().channels.iter() {
                let ch = ch_lock.read();
                if ch.name.to_lowercase() == input.to_lowercase() {
                    return Some((*id, ch.clone()));
                }
            }
            None
        },
    }
}

/// Attempts to parse a guild ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached guild
/// This method is case insensitive
pub fn parse_guild(input: String) -> Option<(GuildId, Arc<RwLock<Guild>>)> {
    let cache = CACHE.read();
    match GUILD_MATCH.captures(input.as_str()) {
        Some(s) => {
            let id = GuildId(s[0].parse::<u64>().unwrap());
            let g_lock = id.find().unwrap();
            Some((id, g_lock))
        },
        None => {
            for (id, g_lock) in cache.guilds.iter() {
                if g_lock.read().name.to_lowercase() == input.to_lowercase() {
                    return Some((*id, Arc::clone(g_lock)));
                }
            }
            None
        },
    }
}

/// This is used for checking if a member has any roles that match the guild's configured mod_roles
/// or admin_roles
pub fn check_rank(roles: Vec<i64>, member: Vec<RoleId>) -> bool {
    for role in roles.iter() {
        if member.contains(&RoleId(*role as u64)) {
            return true;
        }
    }
    false
}

/// Parses a string for flags preceded by `/`
/// The HashMap returned correlates to `/{key} {value}` where value may be an empty string.
/// Additionally, the map will contain the key "rest" which contains anything in the string prior
/// to any unescaped `/` appearing. If no unescaped `/` are present, this will also be the full
/// string.
pub fn get_switches(input: String) -> HashMap<String, String> {
    let input = input.replace(r"\/", "∰"); // use an uncommon substitute because the regex crate doesn't support lookaround, we'll sub back after the regex does its thing
    let mut map: HashMap<String, String> = HashMap::new();
    if let Some(s) = SWITCH_REST.captures(input.as_str()) {
        map.insert("rest".to_string(), s[0].replace("∰", "/").trim().to_string());
    };
    for s in SWITCH_PARTS.captures_iter(input.as_str()) {
        map.insert(s[1].to_string(), s[2].replace("∰", "/").trim().to_string());
    }
    map
}

/// Converts a human-readable time to seconds
/// Example inputs
/// `3 days 2 hours 23 seconds`
/// `7w2d4h`
pub fn hrtime_to_seconds(time: String) -> i64 {
    let mut secs: usize = 0;
    for s in TIME.captures_iter(time.as_str()) {
        let count = s[1].parse::<usize>().unwrap();
        match &s[2] {
            "w" => { secs += count*WEEK },
            "d" => { secs += count*DAY },
            "h" => { secs += count*HOUR },
            "m" => { secs += count*MIN },
            "s" => { secs += count },
            _ => {},
        }
    }
    secs as i64
}

/// Converts a time in seconds to a human readable string
pub fn seconds_to_hrtime(mut secs: usize) -> String {
    let mut time = [0,0,0,0,0];
    let word = ["week", "day", "hour", "min", "sec"];
    while secs>0 {
        if secs>=WEEK {
            time[0] += 1;
            secs -= WEEK;
        } else if secs>=DAY {
            time[1] += 1;
            secs -= DAY;
        } else if secs>=HOUR {
            time[2] += 1;
            secs -= HOUR;
        } else if secs>=MIN {
            time[3] += 1;
            secs -= MIN;
        } else {
            time[4] += secs;
            secs -= secs;
        }
    }
    let mut parts = Vec::new();
    for i in 0..5 {
        if time[i]>1 {
            parts.push(format!("{} {}s", time[i], word[i]));
        } else if time[i]>0 {
            parts.push(format!("{} {}", time[i], word[i]));
        }
    }
    parts.join(", ")
}

pub fn parse_welcome_items<S: Into<String>>(input: S, member: &Member) -> String {
    let input = input.into();
    let mut ret = input.clone();
    let user = member.user.read();
    for word in PLAIN_PARTS.captures_iter(input.as_str()) {
        match word[0].to_lowercase().as_str() {
            "{user}" => {
                ret = input.replace(&word[0], user.mention().as_str());
            },
            "{usertag}" => {
                ret = input.replace(&word[0], user.tag().as_str());
            },
            "{username}" => {
                ret = input.replace(&word[0], user.name.as_str());
            },
            "{guild}" => {
                let guild = member.guild_id.get().unwrap();
                ret = input.replace(&word[0], guild.name.as_str());
            },
            _ => {},
        }
    }
    ret
}

pub fn send_welcome_embed(input: String, member: &Member, channel: ChannelId) -> Result<Message, Error> {
    let user = member.user.read();
    let guild = member.guild_id.get().unwrap();
    channel.send_message(|m| { m .embed(|mut e| {
        for item in EMBED_ITEM.captures_iter(input.as_str()) {
            let caps = EMEBED_PARTS.captures(&item[0]).unwrap();
            match caps["field"].to_lowercase().as_str() {
                "title" => {
                    e = e.title(parse_welcome_items(&caps["value"], member));
                },
                "description" => {
                    e = e.description(parse_welcome_items(&caps["value"], member));
                },
                "thumbnail" => {
                    match caps["value"].to_lowercase().as_str() {
                        "user" => {
                            e = e.thumbnail(user.face());
                        },
                        "member" => {
                            e = e.thumbnail(user.face());
                        },
                        "guild" => {
                            if let Some(ref s) = guild.icon {
                                e = e.thumbnail(s);
                            }
                        },
                        _ => {},
                    }
                },
                "color" => {
                    e = e.colour(u64::from_str_radix(&caps["value"].replace("#",""), 16).unwrap_or(0));
                },
                "colour" => {
                    e = e.colour(u64::from_str_radix(&caps["value"].replace("#",""), 16).unwrap_or(0));
                },
                _ => {},
            }
        }
        e
    })})
}

use core::consts::*;
use regex::Regex;
use serenity::CACHE;
use serenity::Error;
use serenity::model::channel::{
    GuildChannel,
    Message
};
use serenity::model::guild::{
    Guild,
    Role,
    Member
};
use serenity::model::id::*;
use serenity::model::misc::Mentionable;
use serenity::prelude::RwLock;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

lazy_static! {
    static ref CHANNEL_MATCH: Regex = Regex::new(r"(?:<#)?(\d{17,})>*?").expect("Failed to create Regex");
    static ref EMBED_ITEM: Regex    = Regex::new(r"\$[^\$]*").expect("Failed to create Regex");
    static ref EMBED_PARTS: Regex   = Regex::new(r"\$?(?P<field>\S+):(?P<value>.*)").expect("Failed to create Regex");
    static ref GUILD_MATCH: Regex   = Regex::new(r"\d{17,}").expect("Failed to create Regex");
    static ref PLAIN_PARTS: Regex   = Regex::new(r"\{.*?\}").expect("Failed to create Regex");
    static ref ROLE_MATCH: Regex    = Regex::new(r"(?:<@)?&?(\d{17,})>*?").expect("Failed to create Regex");
    static ref SWITCH_PARTS: Regex  = Regex::new(r"/\s*(\S+)([^/]*)").expect("Failed to create Regex");
    static ref SWITCH_REST: Regex   = Regex::new(r"^[^/]+").expect("Failed to create Regex");
    static ref TIME: Regex          = Regex::new(r"(\d+)\s*?(\w)").expect("Failed to create Regex");
    static ref USER_MATCH: Regex    = Regex::new(r"(?:<@)?!?(\d{17,})>*?").expect("Failed to create Regex");
}

/// Attempts to parse a role ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached roles
/// This method is case insensitive
pub fn parse_role(input: String, guild_id: GuildId) -> Option<(RoleId, Role)> {
    match ROLE_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = RoleId::from_str(&s[1]) {
                let guild_lock = CACHE.read().guild(&guild_id);
                if let Some(guild_lock) = guild_lock {
                    let guild = guild_lock.read();
                    if let Some(role) = guild.roles.get(&id) {
                        return Some((id, role.clone()));
                    }
                }
            }
            None
        },
        None => {
            let guild_lock = CACHE.read().guild(&guild_id);
            if let Some(guild_lock) = guild_lock {
                let guild = guild_lock.read();
                for (id, role) in guild.roles.iter() {
                    if role.name.to_lowercase() == input.to_lowercase() {
                        return Some((*id, role.clone()));
                    }
                }
            }
            None
        },
    }
}

/// Attempts to parse a user ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached users
/// This method is case insensitive
pub fn parse_user(input: String, guild_id: GuildId) -> Option<(UserId, Member)> {
    match USER_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = UserId::from_str(&s[1]) {
                let member = CACHE.read().member(&guild_id, &id);
                if let Some(member) = member {
                    return Some((id, member.clone()));
                }
            }
            None
        },
        None => {
            let guild_lock = CACHE.read().guild(&guild_id);
            if let Some(guild_lock) = guild_lock {
                let guild = guild_lock.read();
                for (id, member) in guild.members.iter() {
                    let user = member.user.read();
                    if user.name.to_lowercase() == input.to_lowercase()
                    || user.tag().to_lowercase() == input.to_lowercase()
                    || member.display_name().to_lowercase() == input.to_lowercase() {
                        return Some((*id, member.clone()));
                    }
                }
            }
            None
        },
    }
}

/// Attempts to parse a channel ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached GuildChannels
/// This method is case insensitive
pub fn parse_channel(input: String, guild_id: GuildId) -> Option<(ChannelId, GuildChannel)> {
    match CHANNEL_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = ChannelId::from_str(&s[1]) {
                let ch_lock = CACHE.read().guild_channel(&id);
                if let Some(ch_lock) = ch_lock {
                    let ch = ch_lock.read();
                    return Some((id, ch.clone()));
                }
            }
            None
        },
        None => {
            let guild_lock = CACHE.read().guild(&guild_id);
            if let Some(guild_lock) = guild_lock {
                let guild = guild_lock.read();
                for (id, ch_lock) in guild.channels.iter() {
                    let ch = ch_lock.read();
                    if ch.name.to_lowercase() == input.to_lowercase() {
                        return Some((*id, ch.clone()));
                    }
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
    match GUILD_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = s[0].parse::<u64>() {
                let id = GuildId(id);
                if let Some(g_lock) = id.to_guild_cached() {
                    return Some((id, g_lock));
                }
            }
            None
        },
        None => {
            let guilds = &CACHE.read().guilds;
            for (id, g_lock) in guilds.iter() {
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
pub fn check_rank<T: AsRef<Vec<RoleId>>>(roles: Vec<i64>, member: T) -> bool {
    for role in roles.iter() {
        if member.as_ref().contains(&RoleId(*role as u64)) {
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
    TIME.captures_iter(time.as_str())
        .fold(0, |acc, s| {
            match s[1].parse::<i64>() {
                Err(_) => acc,
                Ok(c) => {
                    match &s[2] {
                        "w" => acc + (c * WEEK as i64),
                        "d" => acc + (c * DAY as i64),
                        "h" => acc + (c * HOUR as i64),
                        "m" => acc + (c * MIN as i64),
                        "s" => acc + c,
                        _ => acc,
                    }
                },
            }
        })
}

/// Converts a time in seconds to a human readable string
pub fn seconds_to_hrtime(secs: usize) -> String {
    let word = ["week", "day", "hour", "min", "sec"];
    fn make_parts(t: usize, steps: &[usize], mut accum: Vec<usize>) -> Vec<usize> {
        match steps.split_first() {
            None => accum,
            Some((s, steps)) => {
                accum.push(t / *s);
                make_parts(t % *s, steps, accum)
            },
        }
    }
    
    make_parts(secs, &[WEEK, DAY, HOUR, MIN, 1], Vec::new())
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if s > &0 {
                if s > &1 {
                    Some(format!("{} {}s", s, word[i]))
                } else {
                    Some(format!("{} {}", s, word[i]))
                }
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(", ")
}

pub fn parse_welcome_items<S: Into<String>>(input: S, member: &Member) -> String {
    let input = input.into();
    let mut ret = input.clone();
    let user = member.user.read();
    for word in PLAIN_PARTS.captures_iter(input.as_str()) {
        match word[0].to_lowercase().as_str() {
            "{user}" => {
                ret = ret.replace(&word[0], user.mention().as_str());
            },
            "{usertag}" => {
                ret = ret.replace(&word[0], user.tag().as_str());
            },
            "{username}" => {
                ret = ret.replace(&word[0], user.name.as_str());
            },
            "{guild}" => {
                if let Ok(guild) = member.guild_id.to_partial_guild() {
                    ret = ret.replace(&word[0], guild.name.as_str());
                }
            },
            "{membercount}" => {
                if let Some(guild) = member.guild_id.to_guild_cached() {
                    ret = ret.replace(&word[0], guild.read().member_count.to_string().as_str());
                }
            },
            _ => {},
        }
    }
    ret
}

pub fn send_welcome_embed(input: String, member: &Member, channel: ChannelId) -> Result<Message, Error> {
    let user = member.user.read();
    channel.send_message(|m| { m .embed(|mut e| {
        for item in EMBED_ITEM.captures_iter(input.as_str()) {
            if let Some(caps) = EMBED_PARTS.captures(&item[0]) {
                match caps["field"].to_lowercase().as_str() {
                    "title" => {
                        e = e.title(parse_welcome_items(&caps["value"], member));
                    },
                    "description" => {
                        e = e.description(parse_welcome_items(&caps["value"], member));
                    },
                    "thumbnail" => {
                        match caps["value"].to_lowercase().trim() {
                            "user" => {
                                e = e.thumbnail(user.face());
                            },
                            "member" => {
                                e = e.thumbnail(user.face());
                            },
                            "guild" => {
                                if let Ok(guild) = member.guild_id.to_partial_guild() {
                                    if let Some(ref s) = guild.icon_url() {
                                        e = e.thumbnail(s);
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    "color" | "colour" => {
                        e = e.colour(u64::from_str_radix(&caps["value"].trim().replace("#",""), 16).unwrap_or(0));
                    },
                    _ => {},
                }
            }
        }
        e
    })})
}

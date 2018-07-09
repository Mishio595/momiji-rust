use std::str::FromStr;
use std::collections::HashMap;
use serenity::model::id::*;
use serenity::model::guild::{Role, Member};
use serenity::model::channel::GuildChannel;
use serenity::CACHE;
use regex::Regex;
use core::consts::*;

pub fn parse_role(input: String, guild: GuildId) -> Option<(RoleId, Role)> {
    let cache = CACHE.read();
    let re = Regex::new(r"(?:<@)?&?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
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

pub fn parse_user(input: String, guild: GuildId) -> Option<(UserId, Member)> {
    let cache = CACHE.read();
    let re = Regex::new(r"(?:<@)?!?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
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

pub fn parse_channel(input: String, guild: GuildId) -> Option<(ChannelId, GuildChannel)> {
    let cache = CACHE.read();
    let re = Regex::new(r"(?:<#)?(\d{17,})>*?").unwrap();
    match re.captures(input.as_str()) {
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

pub fn parse_guild(input: String) -> Option<GuildId> {
    let cache = CACHE.read();
    let re = Regex::new(r"(\d{17,})").unwrap();
    match re.captures(input.as_str()) {
        Some(s) => Some(GuildId(s[1].parse::<u64>().unwrap())),
        None => {
            for (id, g_lock) in cache.guilds.iter() {
                if g_lock.read().name.to_lowercase() == input.to_lowercase() {
                    return Some(*id);
                }
            }
            None
        },
    }
}

pub fn get_switches(input: String) -> HashMap<String, String> {
    let re_rest = Regex::new(r"^[^/]+").unwrap();
    let re = Regex::new(r"/\s*?(\S+)([^/]+)").unwrap();
    let mut map: HashMap<String, String> = HashMap::new();
    if let Some(s) = re_rest.captures(input.as_str()) {
        map.insert("rest".to_string(), s[0].trim().to_string());
    };
    for s in re.captures_iter(input.as_str()) {
        map.insert(s[1].to_string(), s[2].trim().to_string());
    }
    map
}

pub fn hrtime_to_seconds(mut time: String) -> i64 {
    let re = Regex::new(r"(\d+)\s*?(\w)").unwrap();
    let mut secs: usize = 0;
    for s in re.captures_iter(time.as_str()) {
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

use crate::Context;
use super::consts::*;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use twilight_cache_inmemory::{model::CachedMember, model::CachedGuild};
use twilight_embed_builder::{EmbedBuilder, ImageSource};
use twilight_model::channel::GuildChannel;
use twilight_model::guild::{Member, Role, Permissions};
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};
use twilight_model::user::User;
use twilight_mention::Mention;

lazy_static::lazy_static! {
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
pub fn parse_role(input: String, guild_id: GuildId, ctx: Context) -> Option<(RoleId, Arc<Role>)> {
    match ROLE_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = s[1].parse::<u64>() {
                let rid = RoleId(id);
                if let Some(role) = ctx.cache.role(rid) {
                    return Some((rid, role.clone()));
                // } else {
                //     let roles = ctx.http.roles(guild_id).await.unwrap_or(Vec::new());
                //     for r in roles.iter() {
                //         if r.id == rid {
                //             return Some((rid, Arc::new(r.clone())))
                //         }
                //     }
                }
            }
            None
        },
        None => {
            if let Some(roles) = ctx.cache.guild_roles(guild_id) {
                for id in roles.iter() {
                    let r = ctx.cache.role(*id)
                        .and_then(|role| {
                            if role.name.to_lowercase() == input.to_lowercase() { Some((*id, role.clone())) }
                            else { None }
                        });
                    
                    if r.is_some() { return r; }
                }
            }
            None
        },
    }
}

/// Attempts to parse a user ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached users
/// This method is case insensitive
pub async fn parse_user(input: String, guild_id: GuildId, ctx: Context) -> Option<(UserId, Arc<Member>)> {
    match USER_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = s[1].parse::<u64>() {
                let uid = UserId(id);
                if let Some(member) = ctx.cache.member(guild_id, uid) {
                    let member = cached_member_to_member(member);
                    return Some((uid, Arc::new(member)))
                } else {
                    if let Ok(Some(member)) = ctx.http.guild_member(guild_id, uid).await {
                        return Some((uid, Arc::new(member)))
                    }
                }
            }
            None
        },
        None => {
            if let Some(members) = ctx.cache.guild_members(guild_id) {
                for user_id in members.iter() {
                    return ctx.cache.member(guild_id, *user_id)
                        .and_then(|m| {
                            if m.user.name.to_lowercase() == input.to_lowercase()
                                || member_tag(m.clone()) == input.to_lowercase()
                                || display_name(m.clone()).to_lowercase() == input.to_lowercase() {
                                    Some((*user_id, Arc::new(cached_member_to_member(m))))
                                } else { None }
                        });
                }
            }
            None
        },
    }
}

/// Attempts to parse a channel ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached GuildChannels
/// This method is case insensitive
pub fn parse_channel(input: String, guild_id: GuildId, ctx: Context) -> Option<(ChannelId, Arc<GuildChannel>)> {
    match CHANNEL_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = s[1].parse::<u64>() {
                let channel_id = ChannelId(id);
                return ctx.cache.guild_channel(channel_id)
                    .and_then(|channel| {
                        Some((channel_id, channel))
                    });
            }

            None
        },
        None => {
            if let Some(channels) = ctx.cache.guild_channels(guild_id) {
                for channel_id in channels.iter() {
                    return ctx.cache.guild_channel(*channel_id)
                        .and_then(|channel| {
                            if channel.name().to_lowercase() == input.to_lowercase() {
                                Some((*channel_id, channel.clone()))
                            } else {
                                None
                            }
                        });
                }
            }
            None
        },
    }
}

/// Attempts to parse a guild ID out of a string
/// If the string does not contain a valid snowflake, attempt to match as name to cached guild
/// This method is case insensitive
pub fn parse_guild(input: String, ctx: Context) -> Option<(GuildId, Arc<CachedGuild>)> {
    match GUILD_MATCH.captures(input.as_str()) {
        Some(s) => {
            if let Ok(id) = s[0].parse::<u64>() {
                let id = GuildId(id);
                if let Some(guild) = ctx.cache.guild(id) {
                    return Some((id, guild));
                }
            }
            None
        },
        None => {
            // TODO need a way to iterate over cached guilds
            // for (id, g_lock) in guilds.iter() {
            //     if g_lock.read().name.to_lowercase() == input.to_lowercase() {
            //         return Some((*id, Arc::clone(g_lock)));
            //     }
            // }
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

pub fn parse_welcome_items<S: Into<String>>(input: S, member: &Member, ctx: Context) -> String {
    let input = input.into();
    let mut ret = input.clone();
    for word in PLAIN_PARTS.captures_iter(input.as_str()) {
        match word[0].to_lowercase().as_str() {
            "{user}" => {
                ret = ret.replace(&word[0], member.user.mention().to_string().as_str());
            },
            "{usertag}" => {
                ret = ret.replace(&word[0], format!("{}#{}", member.user.name, member.user.discriminator).as_str());
            },
            "{username}" => {
                ret = ret.replace(&word[0], member.user.name.as_str());
            },
            "{guild}" => {
                if let Some(guild) = ctx.cache.guild(member.guild_id) {
                    ret = ret.replace(&word[0], guild.name.as_str());
                }
            },
            "{membercount}" => {
                if let Some(guild) = ctx.cache.guild(member.guild_id) {
                    ret = ret.replace(&word[0], guild.member_count.unwrap_or(0).to_string().as_str());
                }
            },
            _ => {},
        }
    }
    ret
}

pub fn build_welcome_embed(input: String, member: &Member, ctx: Context) -> Result<EmbedBuilder, Box<dyn Error+ Send + Sync>> {
    let mut embed = EmbedBuilder::new();
    for item in EMBED_ITEM.captures_iter(input.as_str()) {
        if let Some(caps) = EMBED_PARTS.captures(&item[0]) {
            match caps["field"].to_lowercase().as_str() {
                "title" => {
                    embed = embed.title(parse_welcome_items(&caps["value"], &member, ctx.clone()));
                },
                "description" => {
                    embed = embed.description(parse_welcome_items(&caps["value"], &member, ctx.clone()));
                },
                "thumbnail" => {
                    match caps["value"].to_lowercase().trim() {
                        "user" | "member" => {
                            embed = embed.thumbnail(ImageSource::url(user_avatar_url(&member.user))?);
                        },
                        "guild" => {
                            if let Some(guild) = ctx.cache.guild((&member).guild_id.clone()) {
                                if let Some(ref s) = guild.icon {
                                    embed = embed.thumbnail(ImageSource::url(guild_icon_url(guild.id, s.clone()))?);
                                }
                            }
                        },
                        _ => {},
                    }
                },
                "color" | "colour" => {
                    embed = embed.color(u64::from_str_radix(&caps["value"].trim().replace("#",""), 16).unwrap_or(0) as u32);
                },
                _ => {},
            }
        }
    }

    Ok(embed)
}

pub fn get_permissions_for_member(m: Arc<CachedMember>, ctx: Context) -> Permissions {
    m.roles.iter().fold(Permissions::empty(), |p, role_id| {
        p | ctx.cache.role(*role_id).and_then(|role| {
            Some(role.permissions)
        }).unwrap_or(Permissions::empty())
    })
}

fn cached_member_to_member(m: Arc<CachedMember>) -> Member {
    Member {
        deaf: m.deaf,
        guild_id: m.guild_id,
        hoisted_role: None,
        joined_at: m.joined_at.clone(),
        mute: m.mute,
        nick: m.nick.clone(),
        pending: m.pending,
        premium_since: m.premium_since.clone(),
        roles: m.roles.clone(),
        user: (*m.user).clone(),
    }
}

pub(crate) fn display_name(m: Arc<CachedMember>) -> String {
    format!("{}", m.nick.clone().unwrap_or(m.user.name.clone()))
}

pub(crate) fn member_tag(m: Arc<CachedMember>) -> String {
    format!("{}#{}", m.user.name, m.user.discriminator)
}

pub(crate) fn user_tag(user: Arc<User>) -> String {
    format!("{}#{}", user.name, user.discriminator)
}

pub(crate) fn user_avatar_url(user: &User) -> String {
    match user.avatar {
        Some(ref hash) => format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.id.0, hash),
        None => format!("https://cdn.discordapp.com/embed/avatars/{}.png", user.discriminator.parse::<usize>().unwrap() % 5)
    }
}

pub(crate) fn guild_icon_url(id: GuildId, hash: String) -> String {
    format!("https://cdn.discordapp.com/icons/{}/{}.png", id.0, hash)
}
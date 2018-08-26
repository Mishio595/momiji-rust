use chrono::{DateTime, TimeZone, Utc};
use serenity::model::id::{UserId, RoleId};
use std::fmt::{Display, Formatter, Result as FmtResult};
use super::schema::*;

// QUERYABLES

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id)]
pub struct Guild {
    pub id: i64,
    pub admin_roles: Vec<i64>,
    pub audit: bool,
    pub audit_channel: i64,
    pub audit_threshold: i16,
    pub autorole: bool,
    pub autoroles: Vec<i64>,
    pub ignored_channels: Vec<i64>,
    pub ignore_level: i16,
    pub introduction: bool,
    pub introduction_channel: i64,
    pub introduction_message: String,
    pub introduction_type: String,
    pub mod_roles: Vec<i64>,
    pub modlog: bool,
    pub modlog_channel: i64,
    pub mute_setup: bool,
    pub prefix: String,
    pub welcome: bool,
    pub welcome_channel: i64,
    pub welcome_message: String,
    pub welcome_type: String,
    pub commands: Vec<String>,
    pub logging: Vec<String>,
}

// Deprecated fields: nickname, roles
#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id, guild_id)]
pub struct User<Tz: TimeZone> {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<i64>,
    pub watchlist: bool,
    pub xp: i64,
    pub last_message: DateTime<Tz>,
    pub registered: Option<DateTime<Tz>>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id, user_id, guild_id)]
pub struct Note<Tz: TimeZone> {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub note: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id, guild_id)]
pub struct Role {
    pub id: i64,
    pub guild_id: i64,
    pub category: String,
    pub aliases: Vec<String>,
    pub required_roles: Vec<i64>,
    pub forbidden_roles: Vec<i64>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id)]
pub struct Timer {
    pub id: i32,
    pub starttime: i64,
    pub endtime: i64,
    pub data: String,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(id, user_id, guild_id)]
pub struct Case<Tz: TimeZone> {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub casetype: String,
    pub reason: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(guild_id, name)]
pub struct Tag {
    pub author: i64,
    pub guild_id: i64,
    pub name: String,
    pub data: String,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[table_name="premium"]
pub struct PremiumSettings {
    pub id: i64,
    pub tier: i32,
    pub register_member_role: Option<i64>,
    pub register_cooldown_role: Option<i64>,
    pub register_cooldown_duration: Option<i32>,
    pub cooldown_restricted_roles: Vec<i64>,
}

// This one would be the same for insertable or queryable, so it has both
#[derive(Queryable, Identifiable, AsChangeset, Insertable, Clone, Debug)]
#[primary_key(id, guild_id)]
pub struct Hackban {
    pub id: i64,
    pub guild_id: i64,
    pub reason: Option<String>,
}

// END QUERYABLES
// INSERTABLES

#[derive(Insertable)]
#[table_name="guilds"]
pub struct NewGuild {
    pub id: i64,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub id: i64,
    pub guild_id: i64,
}

#[derive(Insertable)]
#[table_name="notes"]
pub struct NewNote {
    pub user_id: i64,
    pub guild_id: i64,
    pub note: String,
    pub moderator: i64,
}

#[derive(Insertable)]
#[table_name="roles"]
pub struct NewRole {
    pub id: i64,
    pub guild_id: i64,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
}

#[derive(Insertable)]
#[table_name="timers"]
pub struct NewTimer {
    pub starttime: i64,
    pub endtime: i64,
    pub data: String,
}

#[derive(Insertable)]
#[table_name="cases"]
pub struct NewCase {
    pub user_id: i64,
    pub guild_id: i64,
    pub casetype: String,
    pub reason: Option<String>,
    pub moderator: i64,
}

#[derive(Insertable)]
#[table_name="tags"]
pub struct NewTag {
    pub author: i64,
    pub guild_id: i64,
    pub name: String,
    pub data: String,
}

#[derive(Insertable, Debug)]
#[table_name="premium"]
pub struct NewPremium {
    pub id: i64,
}

// END INSERTABLES
// OTHER STUFF

#[derive(Insertable, AsChangeset, Debug)]
#[table_name="users"]
#[primary_key(id, guild_id)]
pub struct UserUpdate {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
}

impl Display for Guild {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "**Admin Roles:** {}\n**Audit:** {}\n**Audit Channel:** {}\n**Audit Threshold:** {}\n**Autorole:** {}\n**Autoroles:** {}\n**Ignored Channels:** {}\n**Ignore Level:** {}\n**Introduction:** {}\n**Introduction Channel:** {}\n**Introduction Type:** {}\n**Introduction Message:** {}\n**Mod Roles: ** {}\n**Modlog:** {}\n**Modlog Channel:** {}\n**Mute Setup:** {}\n**Prefix:** {}\n**Welcome:** {}\n**Welcome Channel:** {}\n**Welcome Type:** {}\n**Welcome Message:** {}\n**Disabled Commands:** {}\n**Disabled Log Types:** {}",
            self.admin_roles.iter().map(|e| match RoleId(*e as u64).to_role_cached() {
                Some(role) => role.name,
                None => format!("{}", e),
            }).collect::<Vec<String>>().join(", "),
            self.audit,
            format!("<#{}>", self.audit_channel),
            self.audit_threshold,
            self.autorole,
            self.autoroles.iter().map(|e| match RoleId(*e as u64).to_role_cached() {
                Some(role) => role.name,
                None => format!("{}", e),
            }).collect::<Vec<String>>().join(", "),
            self.ignored_channels.iter().map(|e| format!("<#{}>", e)).collect::<Vec<String>>().join(", "),
            self.ignore_level,
            self.introduction,
            format!("<#{}>", self.introduction_channel),
            self.introduction_type,
            self.introduction_message,
            self.mod_roles.iter().map(|e| match RoleId(*e as u64).to_role_cached() {
                Some(role) => role.name,
                None => format!("{}", e),
            }).collect::<Vec<String>>().join(", "),
            self.modlog,
            format!("<#{}>", self.modlog_channel),
            self.mute_setup,
            self.prefix,
            self.welcome,
            format!("<#{}>", self.welcome_channel),
            self.welcome_type,
            self.welcome_message,
            self.commands.join(", "),
            self.logging.join(", ")
    )}
}

impl Display for Note<Utc> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{} wrote on {} (ID: {})\n`{}`",
            match UserId(self.moderator as u64).to_user() {
                Ok(user) => user.tag(),
                Err(_) => format!("{}", self.moderator),
            },
            self.timestamp.format("%a, %d %h %Y @ %H:%M:%S").to_string(),
            self.id,
            self.note)
    }
}

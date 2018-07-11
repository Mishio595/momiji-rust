use chrono::{DateTime, TimeZone};
use super::schema::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use serenity::model::id::RoleId;

// QUERYABLES

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
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
    pub mod_roles: Vec<i64>,
    pub modlog: bool,
    pub modlog_channel: i64,
    pub mute_setup: bool,
    pub prefix: String,
    pub welcome: bool,
    pub welcome_channel: i64,
    pub welcome_message: String,
    pub premium: bool,
    pub premium_tier: i16,
    pub commands: Vec<String>,
    pub logging: Vec<String>,
    pub hackbans: Vec<i64>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
pub struct User<Tz: TimeZone> {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<i64>,
    pub watchlist: bool,
    pub xp: i64,
    pub last_message: DateTime<Tz>
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(index)]
pub struct Note<Tz: TimeZone> {
    pub index: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub note: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
pub struct Role {
    pub id: i64,
    pub guild_id: i64,
    pub category: String,
    pub aliases: Vec<String>,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
pub struct Timer {
    pub id: i32,
    pub starttime: i64,
    pub endtime: i64,
    pub data: String,
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
pub struct Case<Tz: TimeZone> {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub casetype: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>
}

#[derive(Queryable, Identifiable, AsChangeset, Debug)]
pub struct Tag {
    pub id: i32,
    pub author: i64,
    pub guild_id: i64,
    pub name: String,
    pub data: String,
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

// END INSERTABLES
// OTHER STUFF

#[derive(Insertable, AsChangeset, Debug)]
#[table_name="users"]
pub struct UserUpdate {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<i64>,
}

impl Display for Guild {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "**Admin Roles:** {}\n**Audit:** {}\n**Audit Channel:** {}\n**Audit Threshold:** {}\n**Autorole:** {}\n**Autoroles:** {}\n**Ignored Channels:** {}\n**Ignore Level:** {}\n**Introduction:** {}\n**Introduction Channel:** {}\n**Introduction Message:** {}\n**Mod Roles: ** {}\n**Modlog:** {}\n**Modlog Channel:** {}\n**Mute Setup:** {}\n**Prefix:** {}\n**Welcome:** {}\n**Welcome Channel:** {}\n**Welcome Message:** {}\n**Disabled Commands:** {}\n**Disabled Log Types:** {}\n**Hackbans:** {}",
            self.admin_roles.iter().map(|e| RoleId(*e as u64).find().unwrap().name).collect::<Vec<String>>().join(", "),
            self.audit,
            format!("<#{}>", self.audit_channel),
            self.audit_threshold,
            self.autorole,
            self.autoroles.iter().map(|e| RoleId(*e as u64).find().unwrap().name).collect::<Vec<String>>().join(", "),
            self.ignored_channels.iter().map(|e| format!("<#{}>", e)).collect::<Vec<String>>().join(", "),
            self.ignore_level,
            self.introduction,
            format!("<#{}>", self.introduction_channel),
            self.introduction_message,
            self.mod_roles.iter().map(|e| RoleId(*e as u64).find().unwrap().name).collect::<Vec<String>>().join(", "),
            self.modlog,
            format!("<#{}>", self.modlog_channel),
            self.mute_setup,
            self.prefix,
            self.welcome,
            format!("<#{}>", self.welcome_channel),
            self.welcome_message,
            self.commands.join(", "),
            self.logging.join(", "),
            self.hackbans.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(", "))
    }
}

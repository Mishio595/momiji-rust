use chrono::{DateTime, TimeZone};
use super::schema::*;

// QUERYABLES

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Guild {
    pub id: i64,
    pub admin_roles: Vec<i64>,
    pub audit: bool,
    pub audit_channel: i64,
    pub audit_threshold: i16,
    pub autorole: bool,
    pub autoroles: Vec<i64>,
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

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct User {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<i64>,
    pub watchlist: bool,
}

#[derive(Queryable, Identifiable, AsChangeset)]
#[primary_key(user_id)]
pub struct Note<Tz: TimeZone> {
    pub user_id: i64,
    pub guild_id: i64,
    index: i32,
    pub note: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>,
}

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Role {
    pub id: i64,
    pub guild_id: i64,
    pub category: String,
    pub aliases: Vec<String>,
}

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Timer {
    pub id: i32,
    pub starttime: i32,
    pub endtime: i32,
    pub data: String,
}

#[derive(Queryable, Identifiable, AsChangeset)]
pub struct Case<Tz: TimeZone> {
    pub id: i64,
    pub guild_id: i64,
    pub casetype: String,
    pub moderator: i64,
    pub timestamp: DateTime<Tz>
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
    pub starttime: i32,
    pub endtime: i32,
    pub data: String,
}

#[derive(Insertable)]
#[table_name="cases"]
pub struct NewCase {
    pub id: i64,
    pub guild_id: i64,
    pub casetype: String,
    pub moderator: i64,
}

// END INSERTABLES

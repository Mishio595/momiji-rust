use chrono::DateTime;
use chrono::offset::TimeZone;
use super::schema::*;

// QUERYABLES

#[derive(Queryable)]
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
}

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub guild_id: i64,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<i64>,
}

#[derive(Queryable)]
pub struct Note<Tz: TimeZone> {
    pub id: i64,
    pub guild_id: i64,
    pub note: String,
    pub timestamp: DateTime<Tz>,
    pub moderator: i64,
}

#[derive(Queryable)]
pub struct Role {
    pub role_id: i64,
    pub guild_id: i64,
    pub category: String,
    pub aliases: Vec<String>,
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
    pub role_id: i64,
    pub guild_id: i64,
    pub category: String,
    pub aliases: Vec<String>,
}

// END INSERTABLES

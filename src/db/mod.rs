//! A set of abstractions for manipulating a PgSQL database relevant to Momiji's stored data.
mod models;
mod schema;

use kankyo;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::offset::Utc;
use std::env;
use self::models::*;
use self::schema::*;

/// While the struct itself and the connection are public, Database cannot be manually
/// instantiated. Use Database::connect() to start it.
pub struct Database {
    pub conn: PgConnection,
    hidden: (),
}

impl Database {
    /// Create a new database with a connection.
    /// Returns a new Database.
    pub fn connect() -> Self {
        kankyo::load().expect("Failed to load .env");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = PgConnection::establish(&database_url)
            .expect(&format!("Error connection to {}", database_url));

        Database {
            conn,
            hidden: (),
        }
    }

    // Guild Tools
    /// Add a guild with a given ID.
    /// Returns the Guild on success.
    /// Uses default values.
    pub fn new_guild(&self, id: i64) -> QueryResult<Guild> {
        let guild = NewGuild {
            id,
        };
        diesel::insert_into(guilds::table)
            .values(&guild)
            .get_result(&self.conn)
    }
    /// Delete a guild by the ID.
    /// Returns the ID on success.
    pub fn del_guild(&self, id: i64) -> QueryResult<i64> {
        use db::schema::guilds::dsl::*;
        use db::schema::guilds::columns;
        diesel::delete(guilds)
            .filter(id.eq(&id))
            .returning(columns::id)
            .get_result(&self.conn)
    }
    /// Select a guild
    /// Returns the guild on success
    pub fn get_guild(&self, id: i64) -> QueryResult<Guild> {
        use db::schema::guilds::dsl::*;
        guilds.filter(id.eq(&id))
            .first(&self.conn)
    }

    // User Tools
    /// Add a user with a given user ID and guild ID.
    /// Returns the User on success.
    pub fn new_user(&self, id: i64, guild_id: i64) -> QueryResult<User> {
        let user = NewUser {
           id,
           guild_id,
        };
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(&self.conn)
    }
    /// Delete a user by user ID and guild ID.
    /// Returns the ID on success.
    pub fn del_user(&self, id: i64, guild_id: i64) -> QueryResult<i64> {
        use db::schema::users::dsl::*;
        use db::schema::users::columns;
        diesel::delete(users)
            .filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .returning(columns::id)
            .get_result(&self.conn)
    }
    /// Select a user
    /// Returns the user on success
    pub fn get_user(&self, id: i64, guild_id: i64) -> QueryResult<User> {
        use db::schema::users::dsl::*;
        users.filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .first(&self.conn)
    }

    // Role Tools
    /// Add a role with the given role ID, guild ID, and optionally a category and aliases.
    /// Returns the Role on success.
    pub fn new_role(&self, id: i64, guild_id: i64, category: Option<String>, aliases: Option<Vec<String>>) -> QueryResult<Role> {
        let role = NewRole {
            id,
            guild_id,
            category,
            aliases,
        };
        diesel::insert_into(roles::table)
            .values(&role)
            .get_result(&self.conn)
    }
    /// Delete a role by role ID and guild ID.
    /// Returns the ID on success.
    pub fn del_role(&self, id: i64, guild_id: i64) -> QueryResult<i64> {
        use db::schema::roles::dsl::*;
        use db::schema::roles::columns;
        diesel::delete(roles)
            .filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .returning(columns::id)
            .get_result(&self.conn)
    }
    /// Select a role
    /// Returns the role on success
    pub fn get_role(&self, id: i64, guild_id: i64) -> QueryResult<Role> {
        use db::schema::roles::dsl::*;
        roles.filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .first(&self.conn)
    }
    // TODO add get all roles by guild id

    // Note Tools
    /// Add a note to the given user in the given guild by a given moderator
    /// Returns the Note on success.
    pub fn new_note(&self, user_id: i64, guild_id: i64, note: String, moderator: i64) -> QueryResult<Note<Utc>> {
        let note = NewNote {
            user_id,
            guild_id,
            note,
            moderator,
        };
        diesel::insert_into(notes::table)
            .values(&note)
            .get_result(&self.conn)
    }
    /// Delete a note by index, user ID, and guild ID.
    /// Returns the Note.note on success.
    pub fn del_note(&self, index: i32, user_id: i64, guild_id: i64) -> QueryResult<String> {
        use db::schema::notes::dsl::*;
        use db::schema::notes::columns;
        diesel::delete(notes)
            .filter(user_id.eq(&user_id))
            .filter(guild_id.eq(&guild_id))
            .filter(index.eq(&index))
            .returning(columns::note)
            .get_result(&self.conn)
    }
    /// Select a note
    /// Returns the note on success
    pub fn get_note(&self, index: i64, user_id: i64, guild_id: i64) -> QueryResult<Note<Utc>> {
        use db::schema::notes::dsl::*;
        notes.filter(index.eq(&index))
            .filter(user_id.eq(&user_id))
            .filter(guild_id.eq(&guild_id))
            .first(&self.conn)
    }
    // TODO add get all notes by user and guild id

    // Timer Tools
    /// Add a timer
    /// Returns the timer on success.
    pub fn new_timer(&self, starttime: i32, endtime: i32, data: String) -> QueryResult<Timer> {
        let timer = NewTimer {
            starttime,
            endtime,
            data,
        };
        diesel::insert_into(timers::table)
            .values(&timer)
            .get_result(&self.conn)
    }
    /// Delete a timer with the given ID.
    /// Returns the note data on success.
    pub fn del_timer(&self, id: i32) -> QueryResult<String> {
        use db::schema::timers::dsl::*;
        use db::schema::timers::columns;
        diesel::delete(timers)
            .filter(id.eq(&id))
            .returning(columns::data)
            .get_result(&self.conn)
    }
    /// Select a timer
    /// Returns the timer on success
    pub fn get_timer(&self, id: i64) -> QueryResult<Timer> {
        use db::schema::timers::dsl::*;
        timers.filter(id.eq(&id))
            .first(&self.conn)
    }

    // Case Tools
    /// Add a Case
    /// Returns the Case on success
    pub fn new_case(&self, user_id: i64, guild_id: i64, casetype: String, moderator: i64) -> QueryResult<Case<Utc>> {
        let case = NewCase {
            user_id,
            guild_id,
            casetype,
            moderator,
        };
        diesel::insert_into(cases::table)
            .values(&case)
            .get_result(&self.conn)
    }
    /// Delete a case
    /// Returns the case on success.
    pub fn del_case(&self, id: i32, user_id: i64, guild_id: i64) -> QueryResult<Case<Utc>> {
        use db::schema::cases::dsl::*;
        use db::schema::cases::columns;
        diesel::delete(cases)
            .filter(id.eq(&id))
            .filter(user_id.eq(&user_id))
            .filter(guild_id.eq(&guild_id))
            .get_result(&self.conn)
    }
    /// Select a case
    /// Returns the case on success
    pub fn get_case(&self, id: i64, user_id: i64, guild_id: i64) -> QueryResult<Case<Utc>> {
        use db::schema::cases::dsl::*;
        cases.filter(id.eq(&id))
            .filter(user_id.eq(&user_id))
            .filter(guild_id.eq(&guild_id))
            .first(&self.conn)
    }
    // TODO add get all cases by user and guild id
}

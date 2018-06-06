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
    pub fn connect() -> Database {
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
    pub fn add_guild(&self, id: i64) -> Guild {
        let guild = NewGuild {
            id,
        };
        diesel::insert_into(guilds::table)
            .values(&guild)
            .get_result(&self.conn)
            .expect("Failed to create guild entry")
    }
    /// Delete a guild by the ID.
    /// Returns the ID on success.
    pub fn del_guild(&self, id: i64) -> i64 {
        use db::schema::guilds::dsl::*;
        use db::schema::guilds::columns;
        diesel::delete(guilds)
            .filter(id.eq(&id))
            .returning(columns::id)
            .get_result(&self.conn)
            .expect(&format!("Unable to delete guild: {:?}", id))
    }

    // User Tools
    /// Add a user with a given user ID and guild ID.
    /// Returns the User on success.
    pub fn add_user(&self, id: i64, guild_id: i64) -> User {
        let user = NewUser {
           id,
           guild_id,
        };
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(&self.conn)
            .expect("Failed to create user entry")
    }
    /// Delete a user by user ID and guild ID.
    /// Returns the ID on success.
    pub fn del_user(&self, id: i64, guild_id: i64) -> i64 {
        use db::schema::users::dsl::*;
        use db::schema::users::columns;
        diesel::delete(users)
            .filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .returning(columns::id)
            .get_result(&self.conn)
            .expect(&format!("Unable to delete user {:?} from guild {:?}", id, guild_id))
    }

    // Role Tools
    /// Add a role with the given role ID, guild ID, and optionally a category and aliases.
    /// Returns the Role on success.
    pub fn add_role(&self, id: i64, guild_id: i64, category: Option<String>, aliases: Option<Vec<String>>) -> Role {
        let role = NewRole {
            id,
            guild_id,
            category,
            aliases,
        };
        diesel::insert_into(roles::table)
            .values(&role)
            .get_result(&self.conn)
            .expect("Failed to create role entry")
    }
    /// Delete a role by role ID and guild ID.
    /// Returns the ID on success.
    pub fn del_role(&self, id: i64, guild_id: i64) -> i64 {
        use db::schema::roles::dsl::*;
        use db::schema::roles::columns;
        diesel::delete(roles)
            .filter(id.eq(&id))
            .filter(guild_id.eq(&guild_id))
            .returning(columns::id)
            .get_result(&self.conn)
            .expect(&format!("Unable to delete role {:?} from guild {:?}", id, guild_id))
    }

    // Note Tools
    /// Add a note to the given user in the given guild by a given moderator
    /// Returns the Note on success.
    pub fn add_note(&self, user_id: i64, guild_id: i64, note: String, moderator: i64) -> Note<Utc> {
        let note = NewNote {
            user_id,
            guild_id,
            note,
            moderator,
        };
        diesel::insert_into(notes::table)
            .values(&note)
            .get_result(&self.conn)
            .expect("Failed to create note entry")
    }
    /// Delete a note by index, user ID, and guild ID.
    /// Returns the Note.note on success.
    pub fn del_note(&self, index: i32, user_id: i64, guild_id: i64) -> String {
        use db::schema::notes::dsl::*;
        use db::schema::notes::columns;
        diesel::delete(notes)
            .filter(user_id.eq(&user_id))
            .filter(guild_id.eq(&guild_id))
            .filter(index.eq(&index))
            .returning(columns::note)
            .get_result(&self.conn)
            .expect(&format!("Unable to delete note {:?} from user {:?} in guild {:?}", index, user_id, guild_id))
    }
}

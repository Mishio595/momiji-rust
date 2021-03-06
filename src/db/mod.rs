//! A set of abstractions for manipulating a PgSQL database relevant to Momiji's stored data.
pub mod models;
mod schema;

use chrono::offset::Utc;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::r2d2::{
    ConnectionManager,
    Pool,
    PooledConnection
};
use diesel;
use self::models::*;
use self::schema::*;
use std::env;
use std::ops::Deref;
use std::sync::Arc;

/// While the struct itself and the connection are public, Database cannot be manually
/// instantiated. Use Database::connect() to start it.
#[derive(Clone)]
#[non_exhaustive]
pub struct DatabaseConnection {
    pub pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl DatabaseConnection {
    /// Create a new database with a connection.
    /// Returns a new Database.
    pub fn connect() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("Failed to make connection pool");

        Self {
            pool: Arc::new(pool),
        }
    }

    /// Request a connection from the connection pool
    fn conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.clone().get().expect("Attempt to get connection timed out")
    }

    // Guild Tools
    /// Add a guild with a given ID.
    /// Returns the Ok(Some(Guild)) on success or Ok(None) if there is a conflict.
    /// May return Err(DatabaseError) in the event of some other failure.
    pub fn new_guild(&self, id: i64) -> QueryResult<Option<Guild>> {
        let guild = NewGuild {
            id,
        };
        diesel::insert_into(guilds::table)
            .values(&guild)
            .on_conflict_do_nothing()
            .get_result(self.conn().deref())
            .optional()
    }
    /// Add multiple guilds with a vector of IDs
    /// Does nothing on conflict
    /// Returns Result<count, err>
    pub fn new_guilds(&self, ids: &[i64]) -> QueryResult<usize> {
        let guilds = {
            ids.iter().map(|e| {
                NewGuild {
                    id: *e,
                }
            }).collect::<Vec<NewGuild>>()
        };
        diesel::insert_into(guilds::table)
            .values(&guilds)
            .on_conflict_do_nothing()
            .execute(self.conn().deref())
    }
    /// Delete a guild by the ID.
    /// Returns Result<guild_id, err>
    pub fn del_guild(&self, g_id: i64) -> QueryResult<i64> {
        use crate::db::schema::guilds::columns::id;
        diesel::delete(guilds::table)
            .filter(id.eq(&g_id))
            .returning(id)
            .get_result(self.conn().deref())
    }
    /// Select a guild
    /// Returns Result<Guild, Err>
    pub fn get_guild(&self, g_id: i64) -> QueryResult<Guild> {
        guilds::table.find(&g_id)
            .first(self.conn().deref())
    }
    /// Update a guild
    /// Returns Result<Guild, Err>
    pub fn update_guild(&self, g_id: i64, guild: Guild) -> QueryResult<Guild> {
        let target = guilds::table.find(&g_id);
        diesel::update(target)
            .set(&guild)
            .get_result(self.conn().deref())
    }
    /// Get the count of guilds in the database
    pub fn count_guilds(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        guilds::table.select(count_star())
            .get_result(self.conn().deref())
    }

    // User Tools
    /// Add a user with a given user ID and guild ID.
    /// Returns the User on success.
    pub fn new_user(&self, id: i64, guild_id: i64) -> QueryResult<User<Utc>> {
        let user = NewUser {
           id,
           guild_id,
        };
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(self.conn().deref())
    }
    /// Delete a user by user ID and guild ID.
    /// Returns the ID on success.
    pub fn del_user(&self, u_id: i64, g_id: i64) -> QueryResult<i64> {
        use crate::db::schema::users::columns::{id, guild_id};
        diesel::delete(users::table)
            .filter(id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .returning(id)
            .get_result(self.conn().deref())
    }
    /// Select a user
    /// Returns the user on success
    pub fn get_user(&self, u_id: i64, g_id: i64) -> QueryResult<User<Utc>> {
        users::table.find((u_id, g_id))
            .first(self.conn().deref())
    }
    /// Select all users in a guild
    /// Returns a vector of users on success
    pub fn get_users(&self, g_id: i64) -> QueryResult<Vec<User<Utc>>> {
        use crate::db::schema::users::columns::guild_id;
        users::table.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a user
    /// Returns the new user on success
    pub fn update_user(&self, u_id: i64, g_id: i64, user: User<Utc>) -> QueryResult<User<Utc>> {
        let target = users::table.find((u_id, g_id));
        diesel::update(target)
            .set(&user)
            .get_result(self.conn().deref())
    }
    /// Upsert a user
    /// Returns the new user on success
    pub fn upsert_user(&self, user: UserUpdate) -> QueryResult<User<Utc>> {
        use crate::db::schema::users::columns::{id, guild_id};
        diesel::insert_into(users::table)
            .values(&user)
            .on_conflict((id, guild_id))
            .do_update()
            .set(&user)
            .get_result(self.conn().deref())
    }
    /// Upserts multiple users with a vector of UserUpdates
    /// Returns Result<count, err>
    pub fn upsert_users(&self, users: &[UserUpdate]) -> QueryResult<usize> {
        use crate::db::schema::users::columns::*;
        diesel::insert_into(users::table)
            .values(users)
            .on_conflict((id, guild_id))
            .do_update()
            .set((nickname.eq(excluded(nickname)),
                username.eq(excluded(username)),
                roles.eq(excluded(roles))))
            .execute(self.conn().deref())
    }
    /// Get the count of users in the database
    pub fn count_users(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        users::table.select(count_star())
            .get_result(self.conn().deref())
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
            .get_result(self.conn().deref())
    }
    /// Delete a role by role ID and guild ID.
    /// Returns the ID on success.
    pub fn del_role(&self, r_id: i64, g_id: i64) -> QueryResult<i64> {
        use crate::db::schema::roles::columns::{id, guild_id};
        diesel::delete(roles::table)
            .filter(id.eq(&r_id))
            .filter(guild_id.eq(&g_id))
            .returning(id)
            .get_result(self.conn().deref())
    }
    /// Select a role
    /// Returns the role on success
    pub fn get_role(&self, r_id: i64, g_id: i64) -> QueryResult<Role> {
        roles::table.find((r_id, g_id))
            .first(self.conn().deref())
    }
    /// Select all roles by guild id
    /// Returns a vector of roles on success
    pub fn get_roles(&self, g_id: i64) -> QueryResult<Vec<Role>> {
        use crate::db::schema::roles::columns::guild_id;
        roles::table.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a role
    /// Returns the new role on success
    pub fn update_role(&self, r_id: i64, g_id: i64, role: Role) -> QueryResult<Role> {
        let target = roles::table.find((r_id, g_id));
        diesel::update(target)
            .set(&role)
            .get_result(self.conn().deref())
    }
    /// Get the count of roles in the database
    pub fn count_roles(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        roles::table.select(count_star())
            .get_result(self.conn().deref())
    }

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
            .get_result(self.conn().deref())
    }
    /// Delete a note by index, user ID, and guild ID.
    /// Returns the Note.note on success.
    pub fn del_note(&self, n_id: i32, u_id: i64, g_id: i64) -> QueryResult<String> {
        use crate::db::schema::notes::columns::{user_id, guild_id, id, note};
        diesel::delete(notes::table)
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .filter(id.eq(&n_id))
            .returning(note)
            .get_result(self.conn().deref())
    }
    /*
    /// Select a note
    /// Returns the note on success
    pub fn get_note(&self, n_id: i32, u_id: i64, g_id: i64) -> QueryResult<Note<Utc>> {
        notes::table.find((n_id, u_id, g_id))
            .first(self.conn().deref())
    }*/
    /// Select all notes for a user
    /// Returns a vec of notes on success
    pub fn get_notes(&self, u_id: i64, g_id: i64) -> QueryResult<Vec<Note<Utc>>> {
        use crate::db::schema::notes::columns::{user_id, guild_id};
        notes::table.filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Get the count of notes in the database
    pub fn count_notes(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        notes::table.select(count_star())
            .get_result(self.conn().deref())
    }

    // Timer Tools
    /// Add a timer
    /// Returns the timer on success.
    pub fn new_timer(&self, starttime: i64, endtime: i64, data: String) -> QueryResult<Timer> {
        let timer = NewTimer {
            starttime,
            endtime,
            data,
        };
        diesel::insert_into(timers::table)
            .values(&timer)
            .get_result(self.conn().deref())
    }
    /// Delete a timer with the given ID.
    /// Returns the note data on success.
    pub fn del_timer(&self, t_id: i32) -> QueryResult<String> {
        use crate::db::schema::timers::columns::{id, data};
        diesel::delete(timers::table)
            .filter(id.eq(&t_id))
            .returning(data)
            .get_result(self.conn().deref())
    }
    /*
    /// Select a timer
    /// Returns the timer on success
    pub fn get_timer(&self, t_id: i32) -> QueryResult<Timer> {
        timers::table.find(t_id)
            .first(self.conn().deref())
    }*/
    /// Select all timers
    /// Returns a vec of timers on success
    pub fn get_timers(&self) -> QueryResult<Vec<Timer>> {
        timers::table.get_results(self.conn().deref())
    }
    /// Get the count of timers in the database
    pub fn count_timers(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        timers::table.select(count_star())
            .get_result(self.conn().deref())
    }
    /// Get the timer with the closest expiration time to the present
    pub fn get_earliest_timer(&self) -> QueryResult<Timer> {
        use crate::db::schema::timers::{all_columns, columns::endtime};
        timers::table.select(all_columns)
            .order(endtime.asc())
            .first(self.conn().deref())
    }

    // Case Tools
    /// Add a Case
    /// Returns the Case on success
    pub fn new_case(&self, user_id: i64, guild_id: i64, casetype: String, reason: Option<String>, moderator: i64) -> QueryResult<Case<Utc>> {
        let case = NewCase {
            user_id,
            guild_id,
            casetype,
            reason,
            moderator,
        };
        diesel::insert_into(cases::table)
            .values(&case)
            .get_result(self.conn().deref())
    }
    /*
    /// Delete a case
    /// Returns the case on success.
    pub fn del_case(&self, c_id: i32, u_id: i64, g_id: i64) -> QueryResult<Case<Utc>> {
        use db::schema::cases::columns::{id, user_id, guild_id};
        diesel::delete(cases)
            .filter(id.eq(&c_id))
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_result(self.conn().deref())
    }
    /// Select a case
    /// Returns the case on success
    pub fn get_case(&self, c_id: i32, u_id: i64, g_id: i64) -> QueryResult<Case<Utc>> {
        cases::table.find((c_id, u_id, g_id))
            .first(self.conn().deref())
    }*/
    /// Select all cases for a user
    /// Returns a vector of cases on success
    pub fn get_cases(&self, u_id: i64, g_id: i64) -> QueryResult<Vec<Case<Utc>>> {
        use crate::db::schema::cases::columns::{guild_id, user_id};
        cases::table.filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Get the count of cases in the database
    pub fn count_cases(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        cases::table.select(count_star())
            .get_result(self.conn().deref())
    }

    // Tag Tools
    /// Add a Tag
    /// Returns the Tag on success
    pub fn new_tag(&self, author: i64, guild_id: i64, name: String, data: String) -> QueryResult<Tag> {
        let tag = NewTag {
            author,
            guild_id,
            name,
            data,
        };
        diesel::insert_into(tags::table)
            .values(&tag)
            .get_result(self.conn().deref())
    }
    /// Delete a Tag
    /// Returns the Tag on success.
    pub fn del_tag(&self, g_id: i64, nm: String) -> QueryResult<Tag> {
        use crate::db::schema::tags::columns::{name, guild_id};
        diesel::delete(tags::table)
            .filter(name.eq(&nm))
            .filter(guild_id.eq(&g_id))
            .get_result(self.conn().deref())
    }
    /// Select a Tag
    /// Returns the Tag on success
    pub fn get_tag(&self, g_id: i64, nm: String) -> QueryResult<Tag> {
        tags::table.find((g_id, nm))
            .first(self.conn().deref())
    }
    /// Select all tags by guild
    /// Returns Vec<Tag> on success on success
    pub fn get_tags(&self, g_id: i64) -> QueryResult<Vec<Tag>> {
        use crate::db::schema::tags::columns::guild_id;
        tags::table.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a tag
    /// Returns the new tag on success
    pub fn update_tag(&self, g_id: i64, nm: String, tag: Tag) -> QueryResult<Tag> {
        let target = tags::table.find((g_id, nm));
        diesel::update(target)
            .set(&tag)
            .get_result(self.conn().deref())
    }
    /// Get the count of tags in the database
    pub fn count_tags(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        tags::table.select(count_star())
            .get_result(self.conn().deref())
    }

    // Tag Tools
    /// Add a Hackban
    /// Returns the Hackban on success
    pub fn new_hackban(&self, id: i64, guild_id: i64, reason: Option<String>) -> QueryResult<Hackban> {
        let hb = Hackban {
            id,
            guild_id,
            reason,
        };
        diesel::insert_into(hackbans::table)
            .values(&hb)
            .get_result(self.conn().deref())
    }
    /// Delete a Hackban
    /// Returns the Hackban on success.
    pub fn del_hackban(&self, h_id: i64, g_id: i64) -> QueryResult<Hackban> {
        use crate::db::schema::hackbans::columns::{id, guild_id};
        diesel::delete(hackbans::table)
            .filter(id.eq(&h_id))
            .filter(guild_id.eq(&g_id))
            .get_result(self.conn().deref())
    }
    /// Select a Hackban
    /// Returns the Hackban on success
    pub fn get_hackban(&self, id: i64, g_id: i64) -> QueryResult<Hackban> {
        hackbans::table.find((id, g_id))
            .first(self.conn().deref())
    }
    /// Select all hackbans by guild
    /// Returns Vec<Hackban> on success on success
    pub fn get_hackbans(&self, g_id: i64) -> QueryResult<Vec<Hackban>> {
        use crate::db::schema::hackbans::columns::guild_id;
        hackbans::table.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Get the count of hackbans in the database
    pub fn count_hackbans(&self) -> QueryResult<i64> {
        use diesel::dsl::count_star;
        hackbans::table.select(count_star())
            .get_result(self.conn().deref())
    }
}

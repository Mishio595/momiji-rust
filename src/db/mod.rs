//! A set of abstractions for manipulating a PgSQL database relevant to Momiji's stored data.
pub mod models;
mod schema;

use kankyo;
use diesel;
use r2d2;
use r2d2_diesel::ConnectionManager;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use chrono::offset::Utc;
use std::env;
use std::ops::Deref;
use self::models::*;
use self::schema::*;

/// While the struct itself and the connection are public, Database cannot be manually
/// instantiated. Use Database::connect() to start it.
pub struct Database {
    pub pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    _hidden: (),
}

impl Database {
    /// Create a new database with a connection.
    /// Returns a new Database.
    pub fn connect() -> Self {
        kankyo::load().expect("Failed to load .env");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("Failed to make connection pool");

        Database {
            pool,
            _hidden: (),
        }
    }

    /// Request a connection from the connection pool
    fn conn(&self) -> r2d2::PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.clone().get().expect("Attempt to get connection timed out")
    }

    // Guild Tools
    /// Add a guild with a given ID.
    /// Returns the Ok(Some(Guild)) on success or Ok(None) if there is a conflict.
    /// Uses default values.
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
    /// Returns the ID on success.
    pub fn del_guild(&self, g_id: i64) -> QueryResult<i64> {
        use db::schema::guilds::dsl::*;
        use db::schema::guilds::columns;
        diesel::delete(guilds)
            .filter(id.eq(&g_id))
            .returning(columns::id)
            .get_result(self.conn().deref())
    }
    /// Select a guild
    /// Returns the guild on success
    pub fn get_guild(&self, g_id: i64) -> QueryResult<Guild> {
        use db::schema::guilds::dsl::*;
        guilds.filter(id.eq(&g_id))
            .first(self.conn().deref())
    }
    /// Update a guild
    /// Returns the new guild on success
    pub fn update_guild(&self, g_id: i64, guild: Guild) -> QueryResult<Guild> {
        use db::schema::guilds::dsl::*;
        let target = guilds.filter(id.eq(&g_id));
        diesel::update(target)
            .set(&guild)
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
        use db::schema::users::dsl::*;
        use db::schema::users::columns;
        diesel::delete(users)
            .filter(id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .returning(columns::id)
            .get_result(self.conn().deref())
    }
    /// Select a user
    /// Returns the user on success
    pub fn get_user(&self, u_id: i64, g_id: i64) -> QueryResult<User<Utc>> {
        use db::schema::users::dsl::*;
        users.filter(id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .first(self.conn().deref())
    }
    /// Select all users in a guild
    /// Returns a vector of users on success
    pub fn get_users(&self, g_id: i64) -> QueryResult<Vec<User<Utc>>> {
        use db::schema::users::dsl::*;
        users.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a user
    /// Returns the new user on success
    pub fn update_user(&self, u_id: i64, g_id: i64, user: User<Utc>) -> QueryResult<User<Utc>> {
        use db::schema::users::dsl::*;
        let target = users.filter(id.eq(&u_id))
            .filter(guild_id.eq(&g_id));
        diesel::update(target)
            .set(&user)
            .get_result(self.conn().deref())
    }
    /// Upsert a user
    /// Returns the new user on success
    pub fn upsert_user(&self, user: UserUpdate) -> QueryResult<User<Utc>> {
        use db::schema::users::columns;
        diesel::insert_into(users::table)
            .values(&user)
            .on_conflict((columns::id, columns::guild_id))
            .do_update()
            .set(&user)
            .get_result(self.conn().deref())
    }
    /// Upserts multiple users with a vector of UserUpdates
    pub fn upsert_users(&self, users: &[UserUpdate]) -> QueryResult<usize> {
        use db::schema::users::columns::*;
        diesel::insert_into(users::table)
            .values(users)
            .on_conflict((id, guild_id))
            .do_update()
            .set((nickname.eq(excluded(nickname)),
                username.eq(excluded(username)),
                roles.eq(excluded(roles))))
            .execute(self.conn().deref())
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
        use db::schema::roles::dsl::*;
        use db::schema::roles::columns;
        diesel::delete(roles)
            .filter(id.eq(&r_id))
            .filter(guild_id.eq(&g_id))
            .returning(columns::id)
            .get_result(self.conn().deref())
    }
    /// Select a role
    /// Returns the role on success
    pub fn get_role(&self, r_id: i64, g_id: i64) -> QueryResult<Role> {
        use db::schema::roles::dsl::*;
        roles.filter(id.eq(&r_id))
            .filter(guild_id.eq(&g_id))
            .first(self.conn().deref())
    }
    /// Select all roles by guild id
    /// Returns a vector of roles on success
    pub fn get_roles(&self, g_id: i64) -> QueryResult<Vec<Role>> {
        use db::schema::roles::dsl::*;
        roles.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a role
    /// Returns the new role on success
    pub fn update_role(&self, r_id: i64, g_id: i64, role: Role) -> QueryResult<Role> {
        use db::schema::roles::dsl::*;
        let target = roles.filter(id.eq(&r_id))
            .filter(guild_id.eq(&g_id));
        diesel::update(target)
            .set(&role)
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
    pub fn del_note(&self, ind: i32, u_id: i64, g_id: i64) -> QueryResult<String> {
        use db::schema::notes::dsl::*;
        use db::schema::notes::columns;
        diesel::delete(notes)
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .filter(index.eq(&ind))
            .returning(columns::note)
            .get_result(self.conn().deref())
    }
    /*
    /// Select a note
    /// Returns the note on success
    pub fn get_note(&self, ind: i32, u_id: i64, g_id: i64) -> QueryResult<Note<Utc>> {
        use db::schema::notes::dsl::*;
        notes.filter(index.eq(&ind))
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .first(self.conn().deref())
    }*/
    /// Select all notes for a user
    /// Returns a vec of notes on success
    pub fn get_notes(&self, u_id: i64, g_id: i64) -> QueryResult<Vec<Note<Utc>>> {
        use db::schema::notes::dsl::*;
        notes.filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
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
        use db::schema::timers::dsl::*;
        use db::schema::timers::columns;
        diesel::delete(timers)
            .filter(id.eq(&t_id))
            .returning(columns::data)
            .get_result(self.conn().deref())
    }
    /*
    /// Select a timer
    /// Returns the timer on success
    pub fn get_timer(&self, t_id: i32) -> QueryResult<Timer> {
        use db::schema::timers::dsl::*;
        timers.filter(id.eq(&t_id))
            .first(self.conn().deref())
    }*/
    /// Select all timers
    /// Returns a vec of timers on success
    pub fn get_timers(&self) -> QueryResult<Vec<Timer>> {
        use db::schema::timers::dsl::*;
        timers.get_results(self.conn().deref())
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
            .get_result(self.conn().deref())
    }
    /*
    /// Delete a case
    /// Returns the case on success.
    pub fn del_case(&self, c_id: i32, u_id: i64, g_id: i64) -> QueryResult<Case<Utc>> {
        use db::schema::cases::dsl::*;
        diesel::delete(cases)
            .filter(id.eq(&c_id))
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_result(self.conn().deref())
    }
    /// Select a case
    /// Returns the case on success
    pub fn get_case(&self, c_id: i32, u_id: i64, g_id: i64) -> QueryResult<Case<Utc>> {
        use db::schema::cases::dsl::*;
        cases.filter(id.eq(&c_id))
            .filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .first(self.conn().deref())
    }*/
    /// Select all cases for a user
    /// Returns a vector of cases on success
    pub fn get_cases(&self, u_id: i64, g_id: i64) -> QueryResult<Vec<Case<Utc>>> {
        use db::schema::cases::dsl::*;
        cases.filter(user_id.eq(&u_id))
            .filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
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
        use db::schema::tags::dsl::*;
        diesel::delete(tags)
            .filter(name.eq(&nm))
            .filter(guild_id.eq(&g_id))
            .get_result(self.conn().deref())
    }
    /// Select a Tag
    /// Returns the Tag on success
    pub fn get_tag(&self, g_id: i64, nm: String) -> QueryResult<Tag> {
        use db::schema::tags::dsl::*;
        tags.filter(name.eq(&nm))
            .filter(guild_id.eq(&g_id))
            .first(self.conn().deref())
    }
    /// Select all tags by guild
    /// Returns Vec<Tag> on success on success
    pub fn get_tags(&self, g_id: i64) -> QueryResult<Vec<Tag>> {
        use db::schema::tags::dsl::*;
        tags.filter(guild_id.eq(&g_id))
            .get_results(self.conn().deref())
    }
    /// Update a tag
    /// Returns the new tag on success
    pub fn update_tag(&self, g_id: i64, nm: String, tag: Tag) -> QueryResult<Tag> {
        use db::schema::tags::dsl::*;
        let target = tags.filter(name.eq(&nm))
            .filter(guild_id.eq(&g_id));
        diesel::update(target)
            .set(&tag)
            .get_result(self.conn().deref())
    }
    // TODO add premium abstractions
    // Premium Tools
    /// Add premium with a given guild ID.
    /// Returns the PremiumSettings on success.
    pub fn new_premium(&self, id: i64) -> QueryResult<PremiumSettings> {
        let prem = NewPremium {
            id,
        };
        diesel::insert_into(premium::table)
            .values(&prem)
            .get_result(self.conn().deref())
    }
    /// Delete premium by a guild ID.
    /// Returns the ID on success.
    pub fn del_premium(&self, g_id: i64) -> QueryResult<i64> {
        use db::schema::premium::dsl::*;
        use db::schema::premium::columns;
        diesel::delete(premium)
            .filter(id.eq(&g_id))
            .returning(columns::id)
            .get_result(self.conn().deref())
    }
    /// Select PremiumSettings by guild ID
    /// Returns the settings on success
    /// Will return Err if the guild is not premium
    pub fn get_premium(&self, g_id: i64) -> QueryResult<PremiumSettings> {
        use db::schema::premium::dsl::*;
        premium.filter(id.eq(&g_id))
            .first(self.conn().deref())
    }
    /// Update PremiumSettings
    /// Returns the new settings on success
    pub fn update_premium(&self, g_id: i64, settings: PremiumSettings) -> QueryResult<PremiumSettings> {
        use db::schema::premium::dsl::*;
        let target = premium.filter(id.eq(&g_id));
        diesel::update(target)
            .set(&settings)
            .get_result(self.conn().deref())
    }
}

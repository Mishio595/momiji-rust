mod models;
mod schema;

use kankyo;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use self::models::*;
use self::schema::*;

pub struct Database {
    pub conn: PgConnection,
}

impl Database {
    pub fn connect() -> Database {
        kankyo::load().expect("Failed to load .env");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = PgConnection::establish(&database_url)
            .expect(&format!("Error connection to {}", database_url));

        Database {
            conn,
        }
    }

    pub fn new_guild(&self, id: i64) -> Guild {
        let guild = NewGuild {
            id,
        };
        diesel::insert_into(guilds::table)
            .values(&guild)
            .get_result(&self.conn)
            .expect("Failed to create guild entry")
    }
}

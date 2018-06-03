use reql::{Config, Client, Connection, Document, Run};
use futures::Stream;

// Eventually this will be where my database abstractions go. For now its empty

pub struct Database {
    client: Client,
    conn: Connection,
    // error_log: file?,
    // cache: unknown type,
}

impl Database {
    pub fn connect(config: Config) -> Database {
        let client = Client::new();
        let conn = client.connect(config).expect("Unable to connect to RethinkDB");
        Database {
            client,
            conn,
        }
    }
}

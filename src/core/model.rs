use crate::core::api;
use crate::core::timers::TimerClient;
use crate::db::Database;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::UserId;
use serenity::prelude::Mutex;
use std::sync::Arc;
use typemap::Key;

pub struct Owner;
impl Key for Owner {
    type Value = UserId;
}

pub struct SerenityShardManager;
impl Key for SerenityShardManager {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ApiClient;
impl Key for ApiClient {
    type Value = Arc<api::ApiClient>;
}

pub struct DB;
impl Key for DB {
    type Value = Arc<Database>;
}

pub struct TC;
impl Key for TC {
    type Value = Arc<Mutex<TimerClient>>;
}

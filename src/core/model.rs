use db::Database;
use core::api;
use core::timers::TimerClient;
use serenity::prelude::Mutex;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::UserId;
use typemap::Key;
use std::sync::Arc;

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
    type Value = api::ApiClient;
}

pub struct DB;
impl Key for DB {
    type Value = Arc<Mutex<Database>>;
}

pub struct TC;
impl Key for TC {
    type Value = Arc<Mutex<TimerClient>>;
}

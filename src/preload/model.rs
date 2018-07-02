use serenity::prelude::Mutex;
use typemap::Key;
use ::db::Database;
use ::api;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::UserId;
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

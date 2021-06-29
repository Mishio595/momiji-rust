#[macro_use] extern crate diesel;
#[macro_use] extern crate async_trait;

pub mod core;
pub mod db;
pub mod framework;

use crate::core::timers::TimerClient;
use db::DatabaseConnection;
use framework::parser::Parser;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::{
    id::UserId,
    user::{CurrentUser, User}
};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Context {
    pub cache: InMemoryCache,
    pub cluster: Cluster,
    pub db: DatabaseConnection,
    pub http: HttpClient,
    pub parser: Parser,
    pub tc: TimerClient,
    pub user: Arc<CurrentUser>,
    pub owners: Arc<HashMap<UserId, Arc<User>>>,
}
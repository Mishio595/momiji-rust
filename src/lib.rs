#![recursion_limit="128"]

#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serenity;
extern crate chrono;
extern crate forecast;
extern crate fuzzy_match;
extern crate geocoding;
extern crate kankyo;
extern crate kitsu;
extern crate levenshtein;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate sys_info;
extern crate sysinfo;
extern crate threadpool;
extern crate typemap;

pub mod macros;
pub mod core;
pub mod db;
pub mod modules;

use self::core::{
    api,
    handler::Handler,
    model::*,
    framework::MomijiFramework,
    timers
};
use serenity::Error as SerenityError;
use serenity::http;
use serenity::model::id::UserId;
use serenity::prelude::{
    Client,
    Mutex,
};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

pub struct MomijiClient(Client);

impl MomijiClient {
    pub fn new() -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Expected token in environment");
        let mut client = Client::new(&token, Handler).expect("Unable to initialize client");
        {
            let mut data = client.data.lock();
            let api_client = api::ApiClient::new();
            let tc = timers::TimerClient::new();
            data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
            data.insert::<ApiClient>(api_client);
            data.insert::<TC>(Arc::new(Mutex::new(tc)));
        }
        let owners = match http::get_current_application_info() {
            Ok(info) => {
                let mut set = HashSet::new();
                let mut data = client.data.lock();
                data.insert::<Owner>(info.owner.id);
                set.insert(info.owner.id);
                set
            },
            Err(why) => panic!("Couldn't get the application info: {:?}", why),
        };
        client.with_framework(MomijiFramework::new(owners));
        MomijiClient(client)
    }

    pub fn new_with_owners(owners: HashSet<UserId>) -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Expected token in environment");
        let mut client = Client::new(&token, Handler).expect("Unable to initialize client");
        {
            let mut data = client.data.lock();
            let api_client = api::ApiClient::new();
            let tc = timers::TimerClient::new();
            data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
            data.insert::<ApiClient>(api_client);
            data.insert::<TC>(Arc::new(Mutex::new(tc)));
        }
        client.with_framework(MomijiFramework::new(owners));
        MomijiClient(client)
    }

    pub fn start_autosharded(&mut self) -> Result<(), SerenityError> { self.0.start_autosharded() }
}

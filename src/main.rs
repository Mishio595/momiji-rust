#![recursion_limit="128"]

#[macro_use] extern crate log;
#[macro_use] extern crate serenity;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
extern crate pretty_env_logger;
extern crate kankyo;
extern crate threadpool;
extern crate typemap;
extern crate chrono;
extern crate sysinfo;
extern crate sys_info;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

mod modules;
mod core;
mod db;

use core::{
    api,
    handler::Handler,
    model::*,
    framework,
    timers,
};

use serenity::prelude::*;
use serenity::http;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

fn main() {
    kankyo::load().expect("Failed to load .env file");
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected token in environment");

    let mut client = Client::new(&token, Handler).expect("Unable to initialize client");
    {
        let mut data = client.data.lock();
        let api_client = api::ApiClient::new();
        let db = Arc::new(Mutex::new(db::Database::connect()));
        let tc = timers::TimerClient::new(Arc::clone(&db));
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<ApiClient>(api_client);
        data.insert::<DB>(Arc::clone(&db));
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

    client.with_framework(framework::new(owners));

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}

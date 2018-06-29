#[macro_use] extern crate log;
#[macro_use] extern crate serenity;
extern crate pretty_env_logger;
extern crate kankyo;
extern crate threadpool;
extern crate typemap;
extern crate chrono;
extern crate sys_info;
extern crate procinfo;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

mod utils;
mod modules;
use modules::commands::*;
use modules::api;

use serenity::framework::{
    StandardFramework,
};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::model::id::UserId;
use serenity::client::bridge::gateway::ShardManager;
use serenity::http;
use std::sync::Arc;
use std::collections::HashSet;
use std::env;
use typemap::Key;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.name);
    }
}

struct Owner;
impl Key for Owner { type Value = UserId; }
struct SerenityShardManager;
impl Key for SerenityShardManager { type Value = Arc<Mutex<ShardManager>>; }
struct ApiClient;
impl Key for ApiClient { type Value = api::ApiClient; }

fn main() {
    kankyo::load().expect("Failed to load .env file");
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected token in environment");

    let mut client = Client::new(&token, Handler).expect("Unable to initialize client");
    {
        let mut data = client.data.lock();
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        let api_client = api::ApiClient::new();
        data.insert::<ApiClient>(api_client);
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

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix("!")
            .delimiters(vec!(",", ", ", " "))
            .owners(owners))
        .before(|_ctx, msg, command_name| {
            println!("Got command {} by user {}",
                command_name,
                msg.author.name);
            true
        })
        .command("ping", |c| c.cmd(ping))
        .command("bi", |c| c.cmd(bot_info))
        .command("ni", |c| c.cmd(nerdy_info))
        .command("si", |c| c.cmd(server_info))
        .command("ui", |c| c.cmd(user_info))
        .command("ri", |c| c.cmd(role_info))
        .command("roll", |c| c.cmd(roll))
        .command("now", |c| c.cmd(now))
        .command("cat", |c| c.cmd(cat))
        .command("dog", |c| c.cmd(dog))
        .command("joke", |c| c.cmd(dad_joke))
        .command("ud", |c| c.cmd(urban))
        .command("e621", |c| c.cmd(e621))
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}

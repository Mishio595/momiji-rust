#[macro_use] extern crate log;
#[macro_use] extern crate serenity;
extern crate pretty_env_logger;
extern crate kankyo;
extern crate threadpool;
extern crate momiji;

use momiji::framework::*;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use std::collections::HashSet;
use std::env;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.name);
    }
}

fn main() {
    kankyo::load().expect("Failed to load .env file");
    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected token in environment");

    let mut client = Client::new(&token, Handler).expect("Unable to initialize client");

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);
            set
        },
        Err(why) => panic!("Couldn't get the application info: {:?}", why),
    };

    //TODO: Make own framework
    client.with_framework(MomijiFramework::new()
        .configure(|c| c
           .use_mention(true)
           .owners(owners)
           .prefix("!")
        )
        .command("ping", |message, _| {
            let _ = message.channel_id.say("Pong!");
            true
        }, 0, false)
    );

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}

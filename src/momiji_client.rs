use core::{
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
            data.insert::<ApiClient>(Arc::new(api_client));
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
            data.insert::<ApiClient>(Arc::new(api_client));
            data.insert::<TC>(Arc::new(Mutex::new(tc)));
        }
        client.with_framework(MomijiFramework::new(owners));
        MomijiClient(client)
    }

    pub fn start(&mut self) -> Result<(), SerenityError> { self.start_autosharded() }
    pub fn start_autosharded(&mut self) -> Result<(), SerenityError> { self.0.start_autosharded() }
}

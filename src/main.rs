#[macro_use] extern crate async_trait;

pub(crate) mod client;
pub(crate) mod standard_framework;
pub(crate) mod commands;

use std::{env, error::Error};
use client::Client;
use twilight_model::gateway::Intents;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("momiji=debug")
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global default subscriber");

    kankyo::load(false).expect("Failed to load .env file");
    
    let token = env::var("DISCORD_TOKEN")?;
    let intents = Intents::all()
        ^ Intents::GUILD_PRESENCES
        ^ Intents::GUILD_MESSAGE_TYPING
        ^ Intents::DIRECT_MESSAGE_TYPING;

    let client = Client::new(&token, intents).await;
    client.start().await;

    Ok(())
}
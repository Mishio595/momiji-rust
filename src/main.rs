#[macro_use] extern crate log;
extern crate chrono;
extern crate fern;
extern crate kankyo;
extern crate momiji;
extern crate serenity;

use fern::colors::{
    Color,
    ColoredLevelConfig
};
use momiji::core::{
    api,
    handler::Handler,
    model::*,
    framework,
    timers
};
use serenity::http;
use serenity::prelude::{
    Client,
    Mutex
};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

fn main() {
    kankyo::load().expect("Failed to load .env file");
    // This is a bit verbose, but it allows for logging to console with colors and to a file
    // without to avoid ANSI color codes showing up in the log. This is mostly to improve
    // visibility.
    let colors = ColoredLevelConfig::new()
        .trace(Color::Magenta)
        .debug(Color::Cyan)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let term_out = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}  {:level_width$}\t{:target_width$}\t> {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message,
                level_width = 8,
                target_width = 80
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("serenity", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .into_shared();

    let file_out = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}  {:level_width$}\t{:target_width$}\t> {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message,
                level_width = 8,
                target_width = 80
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("serenity", log::LevelFilter::Trace)
        .chain(fern::log_file("output.log").expect("Unable to access log file"))
        .into_shared();

    fern::Dispatch::new()
        .chain(term_out)
        .chain(file_out)
        .apply().expect("Failed to apply fern settings");

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

    client.with_framework(framework::new(owners));

    if let Err(why) = client.start_autosharded() {
        error!("Client error: {:?}", why);
    }
}

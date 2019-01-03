#[macro_use] extern crate log;
#[macro_use] extern crate momiji;
extern crate chrono;
extern crate fern;
extern crate kankyo;

use fern::colors::{
    Color,
    ColoredLevelConfig
};
use momiji::MomijiClient;

fn main() {
    kankyo::load().expect("Failed to load .env file");
    fern_setup().expect("Failed to apply fern settings.");
    let mut client = MomijiClient::new();
    check_error!(client.start());
}

fn fern_setup() -> Result<(), log::SetLoggerError> {
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
                "{time}  {level:level_width$}{target:target_width$}> {msg}",
                time = chrono::Utc::now().format("%F %T"),
                level = colors.color(record.level()),
                target = format!("{}:{}", record.target(), record.line().unwrap_or(0)),
                msg = message,
                level_width = 8,
                target_width = 60
            ))
        })
        .chain(std::io::stdout())
        .into_shared();

    let file_out = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{time}  {level:level_width$}{target:target_width$}> {msg}",
                time = chrono::Utc::now().format("%F %T"),
                level = record.level(),
                target = format!("{}:{}", record.target(), record.line().unwrap_or(0)),
                msg = message,
                level_width = 8,
                target_width = 60
            ))
        })
        .chain(fern::log_file("output.log").expect("Failed to load log file"))
        .into_shared();

    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .level_for("serenity", log::LevelFilter::Debug)
        .level_for("momiji", log::LevelFilter::Debug)
        .chain(term_out)
        .chain(file_out)
        .apply()
}

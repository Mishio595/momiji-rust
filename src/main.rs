#[macro_use] extern crate log;
#[macro_use] extern crate momiji;
extern crate chrono;
extern crate fern;
extern crate kankyo;
extern crate parking_lot;

use fern::colors::{
    Color,
    ColoredLevelConfig
};
use momiji::MomijiClient;

use std::thread;
use std::time::Duration;
use parking_lot::deadlock;

fn main() {
    kankyo::load().expect("Failed to load .env file");
    fern_setup().expect("Failed to apply fern settings.");
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let deadlocks = deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        }
    });
    let mut client = MomijiClient::new();
    check_error!(client.start_autosharded());
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
                "{time}  {level:level_width$}  {target:target_width$}> {msg}",
                time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level = colors.color(record.level()),
                target = format!("{}[#{}]", record.target(), record.line().unwrap_or(0)),
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
                time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level = record.level(),
                target = format!("{}[#{}]", record.target(), record.line().unwrap_or(0)),
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

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
pub mod momiji_client;

pub use momiji_client::MomijiClient;

#![recursion_limit="128"]

#[macro_use] extern crate log;
#[macro_use] extern crate serenity;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
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
extern crate levenshtein;
extern crate fuzzy_match;
extern crate forecast;
extern crate geocoding;
extern crate kitsu;

pub mod macros;
pub mod core;
pub mod db;
pub mod modules;

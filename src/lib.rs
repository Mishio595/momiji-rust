#[macro_use] extern crate serenity;
#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
extern crate chrono;
extern crate kankyo;
extern crate threadpool;
extern crate futures;

pub mod db;
pub mod framework;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fn() {}
}

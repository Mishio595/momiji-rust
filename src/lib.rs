#[macro_use] extern crate serenity;
extern crate threadpool;

use self::framework::*;
use self::framework::command::Command;
use serenity::prelude::*;

pub mod framework;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fn() {}
}

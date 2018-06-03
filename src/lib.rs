#[macro_use] extern crate serenity;
extern crate threadpool;
extern crate reql;
extern crate futures;

pub mod framework;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fn() {}
}

pub mod log;
pub mod premium;

use self::log::*;
use self::premium::*;
use serenity::framework::standard::CreateGroup;

pub fn init() -> CreateGroup {
    CreateGroup::default()
        .cmd("log", Log)
        .cmd("op", Premium)
}
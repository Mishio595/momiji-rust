#![macro_use]

macro_rules! now {
    () => { Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string() };
}

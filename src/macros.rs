#![macro_use]

#[macro_export]
macro_rules! check_error {
    ($e:expr) => {
        if let Err(err) = $e {
            warn!("ERROR [{}:{}] {:?}", line!(), column!(), err);
        }
    };
}

macro_rules! failed {
    ($e:expr) => { warn!("[{}:{}] {}", line!(), column!(), $e); };
    ($e:expr, $w:expr) => { warn!("[{}:{}] {} | {}", line!(), column!(), $e, $w); };
}

macro_rules! now {
    () => { Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string() };
}

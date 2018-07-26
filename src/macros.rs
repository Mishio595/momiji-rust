#![macro_use]

macro_rules! check_error {
    ($e:expr) => {
        if let Err(err) = $e {
            warn!("ERROR [{}:{}] {:?}", line!(), column!(), err);
            ERROR_LOG.send_message(|m| m
                .embed(|e| e
                    .title(format!("Error in {}#{}:{}", module_path!(), line!(), column!()))
                    .description(err)
            )).expect("Failed to send message.");
        }
    };
}

macro_rules! now {
    () => { Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string() };
}

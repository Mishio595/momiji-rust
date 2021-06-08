// use momiji::core::consts::*;
use momiji::db::DatabaseConnection;
use momiji::core::timers::TimerClient;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::channel::Message;
use std::sync::Arc;
use std::error::Error;

pub struct Log;
#[async_trait]
impl Command for Log {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            owner_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, http: HttpClient, _: InMemoryCache, _: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id).reply(message.id).content("Command not yet implemented")?.await?;

        Ok(())
    }
}
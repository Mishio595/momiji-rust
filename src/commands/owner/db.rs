// use momiji::core::consts::*;
use momiji::db::DatabaseConnection;
use momiji::core::timers::TimerClient;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::debug;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::channel::Message;
use std::sync::Arc;
use std::error::Error;

pub struct NewGuild;
#[async_trait]
impl Command for NewGuild {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            owner_only: true,
            ..Options::default()
        };
        
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, http: HttpClient, _: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        let id = args.single::<i64>()?;
        db.new_guild(id)?;
        http.create_message(message.channel_id).reply(message.id).content(format!("Created guild entry for {}", id))?.await?;

        Ok(())
    }
}
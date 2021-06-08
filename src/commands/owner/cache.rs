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

pub struct CacheStats;
#[async_trait]
impl Command for CacheStats {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            owner_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, http: HttpClient, cache: InMemoryCache, _: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stats = cache.stats();
        let create = http.create_message(message.channel_id).reply(message.id);
        let which = args.single::<String>().unwrap_or(String::new());
        let create = match which.as_str() {
            "messages" => { create.content(format!("{:?}", stats.channel_messages(message.channel_id)))? }
            _ => { create.file("cache_stats.txt", format!("{:?}", stats).as_bytes()) }
        };
        
        create.await?;

        Ok(())
    }
}
// use momiji::core::consts::*;
use momiji::Context;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use twilight_model::channel::Message;
use std::sync::Arc;
use std::error::Error;

pub struct CacheStats;
#[async_trait]
impl Command for CacheStats {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            owner_only: true,
            help_available: false,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stats = ctx.cache.stats();
        let create = ctx.http.create_message(message.channel_id).reply(message.id);
        let which = args.single::<String>().unwrap_or(String::new());
        let create = match which.as_str() {
            "messages" => { create.content(format!("{:?}", stats.channel_messages(message.channel_id)))? }
            _ => { create.file("cache_stats.txt", format!("{:?}", stats).as_bytes()) }
        };
        
        create.await?;

        Ok(())
    }
}
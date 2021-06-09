// use momiji::core::consts::*;
use momiji::Context;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
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

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        ctx.http.create_message(message.channel_id).reply(message.id).content("Command not yet implemented")?.await?;

        Ok(())
    }
}
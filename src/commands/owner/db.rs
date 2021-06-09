// use momiji::core::consts::*;
use momiji::Context;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
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

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let id = args.single::<i64>()?;
        ctx.db.new_guild(id)?;
        ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Created guild entry for {}", id))?.await?;

        Ok(())
    }
}
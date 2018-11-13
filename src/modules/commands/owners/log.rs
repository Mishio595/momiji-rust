use std::path::Path;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct Log;
impl Command for Log {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            owners_only: true,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        message.channel_id.send_files(vec![Path::new("./output.log")], |m| m)?;
        Ok(())
    }
}
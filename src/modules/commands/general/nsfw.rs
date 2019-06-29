use crate::core::model::ApiClient;
use crate::core::consts::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct Furry;
impl Command for Furry {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Random image from e621.net. Provide your own tags like you would on the website.".to_string()),
            usage: Some("[tags]".to_string()),
            example: Some("male/male dragon double_penetration".to_string()),
            aliases: vec!["furry"].iter().map(|e| e.to_string()).collect(),
            owner_privileges: false,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, ctx: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        let data = ctx.data.lock();
        message.channel_id.broadcast_typing()?;
        if let Some(api) = data.get::<ApiClient>() {
            let res = api.furry(args.full(), 1)?;
            let post = &res[0];
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .image(&post.file_url)
                    .description(format!("**Tags:** {}\n**Post:** [{}]({})\n**Artist:** {}\n**Score:** {}",
                        &post.tags,
                        &post.id,
                        format!("https://e621.net/post/show/{}", &post.id),
                        &post.artist[0],
                        &post.score
                    ))
            ))?;
        } else { failed!(API_FAIL); }
        Ok(())
    }
}

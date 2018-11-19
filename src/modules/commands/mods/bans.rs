use core::consts::DB as db;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct BanUser;
impl Command for BanUser {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Bans a user. If unable to resolve to a bannable user, the user is hackbanned instead.".to_string()),
            usage: Some("<user_resolvable> [reason]".to_string()),
            example: Some("242675474927583232 makes links for raiders".to_string()),
            required_permissions: Permissions::BAN_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.ok_or("Failed to get guild_id")?;
        
    }
}
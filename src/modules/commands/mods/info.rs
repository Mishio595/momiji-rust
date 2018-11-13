use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct ModInfo;
impl Command for ModInfo {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("View some useful information on a user.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            aliases: vec!["mi", "minfo"].iter().map(|e| e.to_string()).collect(),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, _)) => {
                    let user = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    let cases = db.get_cases(user_id.0 as i64, guild_id.0 as i64)?;
                    let case_fmt = cases.iter().map(|c| format!("Type: {}\nModerator: {}\nTimestamp: {}", c.casetype, c.moderator, c.timestamp)).collect::<Vec<String>>().join("\n");
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Moderator info")
                            .field("Watchlist", { if user.watchlist { "Yes" } else { "No" } }, false)
                            .field("Cases", if case_fmt.is_empty() { "None" } else { case_fmt.as_str() }, false)
                    ))?;
                },
                None => { message.channel_id.say("I couldn't find that user.")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}
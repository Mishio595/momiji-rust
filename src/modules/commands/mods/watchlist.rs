use core::colours;
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
use serenity::model::id::UserId;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct WatchlistAdd;
impl Command for WatchlistAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add a user to the watchlist.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, member)) => {
                    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    user_data.watchlist = true;
                    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data)?;
                    message.channel_id.say(format!("Set {} to watchlist status.", member.display_name().into_owned()))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct WatchlistRemove;
impl Command for WatchlistRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Remove a user from the watchlist.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user_id, member)) => {
                    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
                    user_data.watchlist = false;
                    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data)?;
                    message.channel_id.say(format!("Unset {} from watchlist status.", member.display_name().into_owned()))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct WatchlistList;
impl Command for WatchlistList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List users on the watchlist.".to_string()),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let users = db.get_users(guild_id.0 as i64)?;
            let user_map = users.iter()
                .filter(|e| e.watchlist)
                .map(|u| {
                    match UserId(u.id as u64).to_user() {
                        Ok(user) => user.tag(),
                        Err(_) => format!("<#{}>", u.id),
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Watchlist")
                    .description(user_map)
                    .colour(*colours::MAIN)
            ))?;
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}
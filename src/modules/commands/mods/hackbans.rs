use crate::core::colours;
use crate::core::consts::DB as db;
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

pub struct HackbanAdd;
impl Command for HackbanAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Adds a user to the hackban list. Users on this list will be banned on joining.".to_string()),
            usage: Some("<user_id> [reason]".to_string()),
            example: Some("242675474927583232 makes links for raiders".to_string()),
            required_permissions: Permissions::BAN_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        let user_id = UserId(args.single::<u64>()?);
        match hackbans.iter().find(|e| e.id as u64 == user_id.0) {
            Some(_) => { message.channel_id.say("User is already hackbanned.")?; },
            None => {
                let reason = args.single::<String>().ok();
                db.new_hackban(user_id.0 as i64, guild_id.0 as i64, reason.clone())?;
                message.channel_id.say(format!(
                    "Added {} to the hackban list{}",
                    user_id.0,
                    match reason {
                        Some(r) => format!(" with reason `{}`", r),
                        None => String::new(),
                    }
                ))?;
            }
        }
        Ok(())
    }
}

pub struct HackbanRemove;
impl Command for HackbanRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Removes a user from the hackban list.".to_string()),
            usage: Some("<user_id>".to_string()),
            example: Some("242675474927583232".to_string()),
            required_permissions: Permissions::BAN_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        let user_id = UserId(args.single::<u64>()?);
        match hackbans.iter().find(|e| e.id as u64 == user_id.0) {
            None => { message.channel_id.say("User isn't hackbanned.")?; },
            Some(_) => {
                db.del_hackban(user_id.0 as i64, guild_id.0 as i64)?;
                message.channel_id.say(format!(
                    "Removed {} from the hackban list",
                    user_id.0
                ))?;
            }
        }
        Ok(())
    }
}

pub struct HackbanList;
impl Command for HackbanList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Lets users on the hackban list along with their reasons, if provided.".to_string()),
            required_permissions: Permissions::BAN_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.unwrap();
        let hackbans = db.get_hackbans(guild_id.0 as i64)?;
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .title("Hackbans")
                .description(
                    hackbans.iter().cloned().map(|e| format!(
                        "{}{}",
                        e.id,
                        format!(": `{}`", e.reason.unwrap_or(String::new()))
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
                )
                .colour(*colours::MAIN)
        ))?;
        Ok(())
    }
}
use core::consts::DB as db;
use core::utils::parse_user;
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

pub struct BanUser;
impl Command for BanUser {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Bans a user. If unable to resolve to a bannable user, the user is hackbanned instead.".to_string()),
            usage: Some("<user_resolvable> [time_resolvable] [reason]".to_string()),
            example: Some("242675474927583232 makes links for raiders".to_string()),
            required_permissions: Permissions::BAN_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.ok_or("Failed to get guild_id")?;
        let arg = args.single::<String>()?;
        if let Some((_, member)) = parse_user(arg.clone(), guild_id) {
            let (days, reason) = {
                match args.single_quoted_n::<u8>().ok() {
                    None => {
                        args.skip();
                        (None, Some(args.rest()))
                    },
                    days => {
                        (days, Some(args.rest()))
                    }
                }
            };
            match (days, reason) {
                (Some(d), Some(r)) => { member.ban(&(d,r))?; },
                (Some(d), None) => { member.ban(&d)?; },
                (None, Some(r)) => { member.ban(&r)?; },
                (None, None) => { member.ban(&0)?; },
            }
        } else {
            if let Ok(id) = arg.parse::<i64>() {
                db.new_hackban(id, guild_id.0 as i64, args.single::<String>().ok())?;
            } else {
                message.channel_id.say("User does not exist in guild and argument is not a valid ID.")?;
            }
        }
        Ok(())
    }
}

pub struct KickUser;
impl Command for KickUser {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Kicks a user. Reason is discarded due to a limitation in Serenity but will be implemented at a later date.".to_string()),
            usage: Some("<user_resolvable> [reason]".to_string()),
            example: Some("242675474927583232 makes links for raiders".to_string()),
            required_permissions: Permissions::KICK_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let guild_id = message.guild_id.ok_or("Failed to get guild_id")?;
        if let Some((_, member)) = parse_user(args.single::<String>()?, guild_id) {
            let _reason = args.rest();
            member.kick()?;
        } else {
            message.channel_id.say("User does not exist in guild.")?;
        }
        Ok(())
    }
}
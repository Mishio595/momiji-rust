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
use serenity::model::id::ChannelId;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct TestWelcome;
impl Command for TestWelcome {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Generates a welcome message to test your current setup.".to_string()),
            max_args: Some(0),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some(member) = message.member() {
                let guild_data = db.get_guild(guild_id.0 as i64)?;
                if guild_data.welcome {
                    let channel = ChannelId(guild_data.welcome_channel as u64);
                    if guild_data.welcome_type.as_str() == "embed" {
                        send_welcome_embed(guild_data.welcome_message, &member, channel)?;
                    } else {
                        channel.say(parse_welcome_items(guild_data.welcome_message, &member))?;
                    }
                }
            } else { failed!(MEMBER_FAIL); }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TestIntro;
impl Command for TestIntro {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Generates an introduction message to test your current setup.".to_string()),
            aliases: vec!["introduction"].iter().map(|e| e.to_string()).collect(),
            max_args: Some(0),
            required_permissions: Permissions::MANAGE_GUILD,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            if let Some(member) = message.member() {
                let guild_data = db.get_guild(guild_id.0 as i64)?;
                if guild_data.welcome {
                    let channel = ChannelId(guild_data.introduction_channel as u64);
                    if guild_data.introduction_type.as_str() == "embed" {
                        send_welcome_embed(guild_data.introduction_message, &member, channel)?;
                    } else {
                        channel.say(parse_welcome_items(guild_data.introduction_message, &member))?;
                    }
                }
            } else { failed!(MEMBER_FAIL); }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

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
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct NoteAdd;
impl Command for NoteAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Add a note to a user.".to_string()),
            usage: Some("<user_resolvable> <note>".to_string()),
            example: Some("@Adelyn test note".to_string()),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user,_)) => {
                    let note = args.rest().to_string();
                    let data = db.new_note(user.0 as i64, guild_id.0 as i64, note, message.author.id.0 as i64)?;
                    message.channel_id.say(format!("Added note `{}`.", data.note))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct NoteRemove;
impl Command for NoteRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Delete a note from a user.".to_string()),
            usage: Some("<user_resolvable> <index>".to_string()),
            example: Some("@Adelyn 3".to_string()),
            required_permissions: Permissions::MANAGE_MESSAGES,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
                Some((user,_)) => {
                    let index = args.single::<i32>().unwrap_or(0);
                    let data = db.del_note(index, user.0 as i64, guild_id.0 as i64)?;
                    message.channel_id.say(format!("Deleted note `{}`.", data))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct NoteList;
impl Command for NoteList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("List all notes for a user.".to_string()),
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
                    let notes = db.get_notes(user_id.0 as i64, guild_id.0 as i64)?;
                    let notes_fmt = notes.iter().map(|n| n.to_string()).collect::<Vec<String>>().join("\n\n");
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .colour(*colours::MAIN)
                            .title(format!("Notes for {}", member.display_name().into_owned()))
                            .description(notes_fmt)
                    ))?;
                },
                None => { message.channel_id.say("I couldn't find that user")?; }
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}
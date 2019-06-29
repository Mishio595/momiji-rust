use crate::core::consts::*;
use crate::core::consts::DB as db;
use fuzzy_match::algorithms::*;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::cmp::Ordering;
use std::sync::Arc;

pub struct TagList;
impl Command for TagList {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Alias to `tag list`".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let tags = db.get_tags(guild_id.0 as i64)?;
            if !tags.is_empty() {
                message.channel_id.say(tags.iter().map(|e| e.name.as_str()).collect::<Vec<&str>>().join("\n"))?;
            } else {
                message.channel_id.say("No tags founds.")?;
            }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagSingle;
impl Command for TagSingle {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("View a tag.".to_string()),
            usage: Some("<tag name>".to_string()),
            example: Some("foobar".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.full().trim().to_string();
            let tags = db.get_tags(guild_id.0 as i64)?;
            if !tags.is_empty() {
                if let Some(tag) = tags.iter().find(|e| e.name == tag_input) {
                    message.channel_id.say(&tag.data)?;
                } else {
                    let mut sdc = SorensenDice::new();
                    let mut matches = Vec::new();
                    for tag in tags.iter() {
                        let dist = sdc.get_similarity(tag.name.as_str(), &tag_input);
                        matches.push((tag, dist));
                    }
                    matches.retain(|e| e.1 > 0.2);
                    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
                    matches.truncate(5);
                    let matches = matches.iter().map(|e| e.0.name.clone()).collect::<Vec<String>>();
                    message.channel_id.say(format!("No tag found. Did you mean...\n{}", matches.join("\n")))?;
                }
            } else { message.channel_id.say("There are no tags yet.")?; }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagAdd;
impl Command for TagAdd {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Create a new tag.".to_string()),
            usage: Some("<tag name, quoted> <tag value>".to_string()),
            example: Some(r#""my new tag" look, I made a tag!"#.to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let value = args.rest().to_string();
            let tag = db.new_tag(message.author.id.0 as i64, guild_id.0 as i64, tag_input.clone(), value)?;
            message.channel_id.say(format!("Successfully created tag `{}`", tag.name))?;
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagRemove;
impl Command for TagRemove {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Delete a tag.".to_string()),
            usage: Some("<tag name>".to_string()),
            example: Some("foobar".to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let tag = db.get_tag(guild_id.0 as i64, tag_input.clone())?;
            let check = guild_id
                .member(message.author.id)
                .and_then(|m| m
                    .permissions()
                    .map(|p| p
                        .manage_messages()))
                .unwrap_or(false);
            if message.author.id.0 as i64 == tag.author || check {
                let tag = db.del_tag(guild_id.0 as i64, tag_input.clone())?;
                message.channel_id.say(format!("Successfully deleted tag `{}`", tag.name))?;
            } else { message.channel_id.say("You must own this tag in order to delete it.")?; }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagEdit;
impl Command for TagEdit {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Edit a tag. Only works if you are the author.".to_string()),
            usage: Some("<tag name, quoted> <new value>".to_string()),
            example: Some(r#""my edited tag" I had to edit this tag"#.to_string()),
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let value = args.rest().to_string();
            let mut tag = db.get_tag(guild_id.0 as i64, tag_input.clone())?;
            let check = guild_id
                .member(message.author.id)
                .and_then(|m| m
                    .permissions()
                    .map(|p| p
                        .manage_messages()))
                .unwrap_or(false);
            if message.author.id.0 as i64 == tag.author || check {
                tag.data = value.clone();
                let t = db.update_tag(guild_id.0 as i64, tag_input.clone(), tag)?;
                message.channel_id.say(format!("Successfully edited tag `{}`", t.name))?;
            } else { message.channel_id.say("You must own this tag in order to edit it.")?; }
        } else { failed!(GUILDID_FAIL); }
        Ok(())
    }
}
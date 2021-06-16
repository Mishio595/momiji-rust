use fuzzy_match::algorithms::*;
use momiji::Context;
use momiji::core::{consts::*, utils::get_permissions_for_member};
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use tracing::debug;
use twilight_model::channel::Message;
use twilight_model::guild::Permissions;
use std::cmp::Ordering;
use std::error::Error;
use std::sync::Arc;

pub struct TagList;
#[async_trait]
impl Command for TagList {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Alias to `tag list`".to_string()),
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let tags = ctx.db.get_tags(guild_id.0 as i64)?;
            if !tags.is_empty() {
                ctx.http.create_message(message.channel_id).reply(message.id).content(tags.iter().map(|e| e.name.as_str()).collect::<Vec<&str>>().join("\n"))?.await?;
            } else {
                ctx.http.create_message(message.channel_id).reply(message.id).content("No tags founds.")?.await?;
            }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagSingle;
#[async_trait]
impl Command for TagSingle {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("View a tag.".to_string()),
            usage: Some("<tag name>".to_string()),
            examples: vec!["foobar".to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.rest().trim().to_string();
            let tags = ctx.db.get_tags(guild_id.0 as i64)?;
            if !tags.is_empty() {
                if let Some(tag) = tags.iter().find(|e| e.name == tag_input) {
                    ctx.http.create_message(message.channel_id).reply(message.id).content(&tag.data)?.await?;
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
                    ctx.http.create_message(message.channel_id).reply(message.id).content(format!("No tag found. Did you mean...\n{}", matches.join("\n")))?.await?;
                }
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("There are no tags yet.")?.await?; }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagAdd;
#[async_trait]
impl Command for TagAdd {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Create a new tag.".to_string()),
            usage: Some("<tag name, quoted> <tag value>".to_string()),
            examples: vec![r#""my new tag" look, I made a tag!"#.to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let value = args.rest().to_string();
            let tag = ctx.db.new_tag(message.author.id.0 as i64, guild_id.0 as i64, tag_input.clone(), value)?;
            ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully created tag `{}`", tag.name))?.await?;
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagRemove;
#[async_trait]
impl Command for TagRemove {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Delete a tag.".to_string()),
            usage: Some("<tag name>".to_string()),
            examples: vec!["foobar".to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let tag = ctx.db.get_tag(guild_id.0 as i64, tag_input.clone())?;
            let check = ctx.cache.member(guild_id, message.author.id)
                .and_then(|m| {
                    Some(get_permissions_for_member(m, ctx.clone()).contains(Permissions::MANAGE_MESSAGES))
                }).unwrap_or(false);
            if message.author.id.0 as i64 == tag.author || check {
                let tag = ctx.db.del_tag(guild_id.0 as i64, tag_input.clone())?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully deleted tag `{}`", tag.name))?.await?;
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("You must own this tag in order to delete it.")?.await?; }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}

pub struct TagEdit;
#[async_trait]
impl Command for TagEdit {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Edit a tag. Only works if you are the author.".to_string()),
            usage: Some("<tag name, quoted> <new value>".to_string()),
            examples: vec![r#""my edited tag" I had to edit this tag"#.to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            let tag_input = args.single_quoted::<String>()?;
            let value = args.rest().to_string();
            let mut tag = ctx.db.get_tag(guild_id.0 as i64, tag_input.clone())?;
            let check = ctx.cache.member(guild_id, message.author.id)
                .and_then(|m| {
                    Some(get_permissions_for_member(m, ctx.clone()).contains(Permissions::MANAGE_MESSAGES))
                }).unwrap_or(false);
            if message.author.id.0 as i64 == tag.author || check {
                tag.data = value.clone();
                let t = ctx.db.update_tag(guild_id.0 as i64, tag_input.clone(), tag)?;
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("Successfully edited tag `{}`", t.name))?.await?;
            } else { ctx.http.create_message(message.channel_id).reply(message.id).content("You must own this tag in order to edit it.")?.await?; }
        } else { debug!("{}", GUILDID_FAIL); }
        Ok(())
    }
}
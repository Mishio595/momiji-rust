use crate::core::consts::*;
use crate::core::timers::TimerClient;
use crate::core::utils::user_tag;
use crate::db::DatabaseConnection;
use crate::framework::Framework;
use crate::framework::parser::Parser;
use futures::stream::StreamExt;
use tracing::{event, Level};
use twilight_mention::Mention;
use std::error::Error;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder};
use twilight_gateway::{cluster::Cluster, Event, EventType};
use twilight_http::Client as HttpClient;
use twilight_model::id::{ChannelId, GuildId};

pub struct EventHandler<P: Parser + Clone>(Arc<Framework<P>>);

impl<P: Parser + Clone> EventHandler<P> {
    pub fn new(framework: Framework<P>) -> Self {
        Self(Arc::new(framework))
    }
 
    pub async fn start(&self, cluster: &Cluster, http: &HttpClient, cache: &InMemoryCache, db: DatabaseConnection, timers: TimerClient) {
        let mut events = cluster.events();
        while let Some((shard_id, event)) = events.next().await {
            // Bypass cache update for message delete to enable logging
            if event.kind() != EventType::MessageDelete {
                cache.update(&event);
            }
    
            tokio::spawn(handle_event(shard_id, event, http.clone(), self.0.clone(), cache.clone(), db.clone(), timers.clone()));
        }
    }
}

async fn handle_event<P: Parser + Clone>(
    shard_id: u64,
    event: Event,
    http: HttpClient,
    framework: Arc<Framework<P>>,
    cache: InMemoryCache,
    db: DatabaseConnection,
    timers: TimerClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(message) => {
            if let Err(e) = (*framework).handle_command(message.0, http, cache, db, timers).await {
                event!(Level::DEBUG, "{}", e);
            }
        }
        Event::MessageDelete(message) => {
            if let Some(guild_id) = message.guild_id {
                let channel_name = cache.guild_channel(message.channel_id).and_then(|c| Some(c.name().to_string())).unwrap_or("unknown".to_string());
                let guild_data = db.get_guild(guild_id.0 as i64)?;
                if guild_data.logging.contains(&String::from("message_delete")) { return Ok(()); }
                let audit_channel = ChannelId(guild_data.audit_channel as u64);
                if guild_data.audit && audit_channel.0 > 0 {
                    if let Some(cached_message) = cache.message(message.channel_id, message.id) {
                        if let Some(author) = cache.user(cached_message.author) {
                            if author.bot { return Ok(()); }
                            let embed = EmbedBuilder::new()
                                .title("Message Deleted")
                                .color(colors::RED)
                                .footer(EmbedFooterBuilder::new(format!("ID: {}", message.id.0)))
                                .description(format!("**Author:** {} ({}) - {}\n**Channel:** {} ({}) - <#{}>\n**Content:**\n{}",
                                    user_tag(author.clone()),
                                    author.id.0,
                                    author.mention(),
                                    channel_name,
                                    message.channel_id.0,
                                    message.channel_id.0,
                                    cached_message.content))
                                .timestamp(chrono::Utc::now().to_rfc3339())
                                .build()?;
                            
                            http.create_message(audit_channel).embed(embed)?.await?;
                        }
                    } else {
                        let channel_name = cache.guild_channel(message.channel_id).and_then(|c| Some(c.name().to_string())).unwrap_or("unknown".to_string());
                        let embed = EmbedBuilder::new()
                            .title("Uncached Message Deleted")
                            .color(colors::RED)
                            .footer(EmbedFooterBuilder::new(format!("ID: {}", message.id.0)))
                            .description(format!("**Channel:** {} ({}) - <#{}>",
                                channel_name,
                                message.channel_id.0,
                                message.channel_id.0,))
                            .timestamp(chrono::Utc::now().to_rfc3339())
                            .build()?;

                        http.create_message(audit_channel).embed(embed)?.await?;
                    }
                }
            }
        }
        Event::ShardConnected(_) => {
            event!(Level::DEBUG, "Connected on shard {}", shard_id);
        }
        Event::GuildCreate(guild) => {
            event!(Level::DEBUG, "Guild received: {} ({})", guild.name, guild.id);
            match db.new_guild(guild.id.0 as i64) {
                Err(why) => { event!(Level::DEBUG, "{}: {}", DB_GUILD_ENTRY_FAIL, why); }
                _ => {}
            }
        }
        Event::Ready(ready) => {
            event!(Level::DEBUG, "Connected with session_id {}", ready.session_id);
        }
        Event::Resumed => {
            event!(Level::DEBUG, "Session resumed");
        }
        _ => { event!(Level::DEBUG, "Unhandled event: {:?}", event.kind()); }
    }

    Ok(())
}
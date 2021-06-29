use crate::core::consts::*;
use crate::core::utils::*;
use crate::db::DatabaseConnection;
use chrono::Utc;
use futures::{StreamExt, channel::mpsc::{
    Sender,
    Receiver,
    channel
}};
use futures::future;
use parking_lot::Mutex;
use twilight_embed_builder::EmbedBuilder;
use twilight_mention::Mention;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use twilight_cache_inmemory::InMemoryCache as Cache;
use twilight_http::Client as HttpClient;
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

#[derive(Clone)]
pub struct TimerClient {
    tx: Arc<Mutex<Sender<()>>>,
    rx: Arc<Mutex<Receiver<()>>>,
    http: HttpClient,
    cache: Cache,
    db: DatabaseConnection,
}

impl TimerClient {
    pub fn new(http: HttpClient, cache: Cache, db: DatabaseConnection) -> Self {
        let (tx, rx) = channel(32);
        
        TimerClient {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            http,
            cache,
            db
        }
    }

    pub async fn start(&self) {
        loop {
            let rx_lock = self.rx.clone();
            let mut rx = rx_lock.lock();
            let f1 = Box::pin(self.run_timer());
            let f2 = Box::pin(rx.next());

            future::select(f1, f2).await;
        }
    }

    async fn run_timer(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = self.db.clone();
        let timer = match db.get_earliest_timer() {
            Ok(timer) => timer,
            _ => future::pending().await,
        };

        let dur = u64::checked_sub(timer.endtime as u64, Utc::now().timestamp() as u64).unwrap_or(0);

        tokio::time::sleep(Duration::from_secs(dur)).await;
            
        let parts = timer.data.split("||").map(|s| s.to_string()).collect::<Vec<String>>();
        match parts[0].as_str() {
            "REMINDER" => {
                // type, channel_id, user_id, dur, reminder
                let cid = parts[1].parse::<u64>().ok().map(ChannelId);
                let uid = parts[2].parse::<u64>().ok().map(UserId);
                let dur = seconds_to_hrtime(parts[3].parse::<usize>().unwrap_or(0));
                let rem = &parts[4];
                match (cid, uid) {
                    (Some(cid), Some(uid)) => { self.reminder(cid, uid, dur, rem).await?; },
                    _ => (),
                }
            },
            "UNMUTE" => {
                // type, user_id, guild_id, mute_role, channel_id, dur
                let uid = parts[1].parse::<u64>().ok().map(UserId);
                let gid = parts[2].parse::<u64>().ok().map(GuildId);
                let rid = parts[3].parse::<u64>().ok().map(RoleId);
                let cid = parts[4].parse::<u64>().ok().map(ChannelId);
                match (uid, gid, cid, rid) {
                    (Some(u), Some(g), Some(c), Some(r)) => { self.unmute(u,g,c,r).await?; },
                    _ => (),
                }
            },
            "COOLDOWN" => {
                // type, user_id, guild_id, member_role_id, cooldown_role_id
                let uid = parts[1].parse::<u64>().ok().map(UserId);
                let gid = parts[2].parse::<u64>().ok().map(GuildId);
                let mrid = parts[3].parse::<u64>().ok().map(RoleId);
                let crid = parts[4].parse::<u64>().ok().map(RoleId);
                match (uid, gid, mrid, crid) {
                    (Some(u), Some(g), Some(m), Some(c)) => { self.cooldown(u,g,m,c).await?; },
                    _ => (),
                }
            },
            _ => {},
        }
        db.del_timer(timer.id)?;

        Ok(())
    }

    async fn reminder(&self, channel_id: ChannelId, user_id: UserId, dur: String, reminder: &String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let content = match self.cache.guild_channel(channel_id) {
            Some(_) => user_id.mention().to_string(),
            None => String::new(),
        };

        let embed = EmbedBuilder::new()
            .title(format!("Reminder from {} ago", dur))
            .color(colors::MAIN)
            .description(reminder)
            .build()?;

        self.http.create_message(channel_id)
            .content(content)?
            .embed(embed)?
            .await?;

        Ok(())
    }
    
    async fn unmute(&self, user_id: UserId, guild_id: GuildId, channel_id: ChannelId, role_id: RoleId) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.cache.user(user_id) {
            Some(user) => {
                self.http.remove_guild_member_role(guild_id, user_id, role_id).await?;

                let embed = EmbedBuilder::new()
                    .title("Member unmuted automatically")
                    .color(colors::GREEN)
                    .description(format!("**Member:** {} ({})", user_tag(user), user_id.0))
                    .build()?;

                self.http.create_message(channel_id)
                    .embed(embed)?
                    .await?;
            },
            _ => {},
        }

        Ok(())
    }
    
    async fn cooldown(&self, user_id: UserId, guild_id: GuildId, mrole_id: RoleId, crole_id: RoleId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.http.add_guild_member_role(guild_id, user_id, mrole_id).await?;
        self.http.remove_guild_member_role(guild_id, user_id, crole_id).await?;

        Ok(())
    }

    pub fn request(&self) {
        let _ = self.tx.lock().try_send(());
    }
}

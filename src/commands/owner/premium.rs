use momiji::core::timers::TimerClient;
use momiji::db::DatabaseConnection;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use std::error::Error;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::channel::Message;

pub struct Premium;
#[async_trait]
impl Command for Premium {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            owner_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, mut args: Args, http: HttpClient, cache: InMemoryCache, db: DatabaseConnection, _: TimerClient) -> Result<(), Box<dyn Error + Send + Sync>> {
        let op = args.single::<String>()?;
        let g = args.single_quoted::<String>()?;
        let mut reply = http.create_message(message.channel_id).reply(message.id);
        if let Some((guild_id, guild)) = parse_guild(g, &cache) {
            match op.to_lowercase().as_str() {
                "enable" => {
                    if let Ok(_) = db.new_premium(guild_id.0 as i64) {
                        reply = reply.content(format!("{} is now premium!", guild.name))?;
                    }
                },
                "disable" => {
                    if let Ok(_) = db.del_premium(guild_id.0 as i64) {
                        reply = reply.content(format!("{} is no longer premium.", guild.name))?;
                    }
                },
                "set" => {
                    let mut prem = db.get_premium(guild_id.0 as i64)?;
                    prem.tier = args.single::<i32>()?;
                    let pr = db.update_premium(guild_id.0 as i64, prem)?;
                    reply = reply.content(format!("Updated premium tier for {} to {}.", guild.name, pr.tier))?;
                },
                "show" => {
                    if let Ok(prem) = db.get_premium(guild_id.0 as i64) {
                        // TODO add impl Display for PremiumSettings
                        reply = reply.content(format!("{:?}", prem))?;
                    }
                },
                _ => {},
            }
        }
        reply.await?;

        Ok(())
    }
}

use chrono::Utc;
use momiji::Context;
use momiji::core::consts::*;
use momiji::core::utils::*;
use momiji::framework::args::Args;
use momiji::framework::command::{Command, Options};
use std::error::Error;
use std::sync::Arc;
use sysinfo::{System, SystemExt, ProcessExt};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder, ImageSource};
use twilight_model::channel::Message;

// lazy_static! {
//     static ref DICE_MATCH: Regex = Regex::new(r"(?P<count>\d+)d?(?P<sides>\d*)").expect("Failed to create Regex");
// }

pub struct BotInfo;
#[async_trait]
impl Command for BotInfo {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Information about the bot.".to_string()),
            usage: Some("".to_string()),
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let guild_count = ctx.cache.stats().guilds();
        let shard_count = ctx.cluster.shards().len();
        let mut owners = String::new();
        for owner in ctx.owners.values() {
            owners = format!("{}\nName: {}#{} ({})", owners, owner.name, owner.discriminator, owner.id.0);
        }
        let sys = System::new_all();
        let system_info = format!(
            "Type: {} {}\nUptime: {}",
            sys.get_name().unwrap_or(String::from("OS Not Found")),
            sys.get_os_version().unwrap_or(String::from("Release Not Found")),
            seconds_to_hrtime(sys.get_uptime() as usize));
        let mut embed = EmbedBuilder::new()
            .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Twilight](https://github.com/twilight-rs/twilight/).")
            .thumbnail(ImageSource::url(avatar_url_from_parts(&ctx.user.avatar, ctx.user.id, ctx.user.discriminator.as_str()))?)
            .color(colors::MAIN)
            .field(EmbedFieldBuilder::new("Owners", owners))
            .field(EmbedFieldBuilder::new("Counts", format!("Guilds: {}\nShards: {}", guild_count, shard_count)).inline())
            .field(EmbedFieldBuilder::new("Links", format!("[Support Server]({})\n[Invite]({})\n[GitLab]({})\n", SUPPORT_SERV_INVITE, BOT_INVITE, GITLAB_LINK)).inline())
            .field(EmbedFieldBuilder::new("System Info", system_info).inline());
        if let Ok(pid) = sysinfo::get_current_pid() {
            if let Some(process) = sys.get_process(pid) {
                let process_info = format!(
                    "Memory Usage: {} MB\nCPU Usage: {}%\nUptime: {}",
                    process.memory()/1000,
                    (process.cpu_usage()*100.0).round()/100.0,
                    seconds_to_hrtime(((Utc::now().timestamp() as u64) - process.start_time()) as usize));
                embed = embed
                    .field(EmbedFieldBuilder::new("Process Info", process_info).inline());
            }
        }

        ctx.http.create_message(message.channel_id).reply(message.id)
            .embed(embed.build()?)?
            .await?;

        Ok(())
    }
}

// pub struct Cat;
// impl Command for Cat {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Random cat photo or gif.".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let data = ctx.data.lock();
//         if let Some(api) = data.get::<ApiClient>() {
//             let res = api.cat()?;
//             message.channel_id.send_message(|m| m
//                 .embed(|e| e
//                     .image(res.file)
//             ))?;
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct Dog;
// impl Command for Dog {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Random dog photo or gif.".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let data = ctx.data.lock();
//         if let Some(api) = data.get::<ApiClient>() {
//             let res = api.dog()?;
//             message.channel_id.send_message(|m| m
//                 .embed(|e| e
//                     .image(res.message)
//             ))?;
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct DadJoke;
// impl Command for DadJoke {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Dad jokes, now in discord.".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let data = ctx.data.lock();
//         if let Some(api) = data.get::<ApiClient>() {
//             let res = api.joke()?;
//             message.channel_id.say(res)?;
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct Anime;
// impl Command for Anime {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Search for an anime using kitsu.io".to_string()),
//             usage: Some("<anime title>".to_string()),
//             example: Some("darling in the franxx".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         use kitsu::model::Status::*;
//         let data = ctx.data.lock();
//         message.channel_id.broadcast_typing()?;
//         if let Some(api) = data.get::<ApiClient>() {
//             let res = api.anime(args.full())?;
//             if let Some(anime) = res.data.first() {
//                 let status = match anime.attributes.status {
//                     Some(stat) => { match stat {
//                         Current => "Current",
//                         Finished => "Complete",
//                         TBA => "To Be Announced",
//                         Unreleased => "Unreleased",
//                         Upcoming => "Upcoming",
//                     }},
//                     None => "Status Not Found",
//                 };
//                 let cover_url = match anime.attributes.cover_image.clone() {
//                     Some(cover) => { match cover.original {
//                         Some(url) => url,
//                         None => String::new(),
//                     }},
//                     None => String::new(),
//                 };
//                 message.channel_id.send_message(|m| m
//                     .embed(|e| e
//                         .title(anime.attributes.canonical_title.clone())
//                         .url(anime.url())
//                         .descriptionription(format!("{}\n\n{}\n**Score:** {}\n**Status:** {}",
//                             anime.attributes.synopsis,
//                             if let Some(count) = anime.attributes.episode_count {
//                                 let mut out = format!("**Episodes:** {}", count);
//                                 if let Some(length) = anime.attributes.episode_length {
//                                     out.push_str(format!(" ({} min/ep)", length).as_str());
//                                 }
//                                 out
//                             } else { String::from("Episode Information Not Found") },
//                             anime.attributes.average_rating.clone().unwrap_or(String::from("Not Found")),
//                             status
//                         ))
//                         .thumbnail(cover_url)
//                         .colour(*colours::MAIN)
//                 ))?;
//             }
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct Manga;
// impl Command for Manga {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Search for a manga using kitsu.io".to_string()),
//             usage: Some("<anime title>".to_string()),
//             example: Some("tsubasa".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         use kitsu::model::Status::*;
//         let data = ctx.data.lock();
//         message.channel_id.broadcast_typing()?;
//         if let Some(api) = data.get::<ApiClient>() {
//             let res = api.manga(args.full())?;
//             if let Some(manga) = res.data.first() {
//                 let status = match manga.attributes.status {
//                     Some(stat) => { match stat {
//                         Current => "Current",
//                         Finished => "Complete",
//                         TBA => "To Be Announced",
//                         Unreleased => "Unreleased",
//                         Upcoming => "Upcoming",
//                     }},
//                     None => "Status Not Found",
//                 };
//                 let cover_url = match manga.attributes.cover_image.clone() {
//                     Some(cover) => { match cover.original {
//                         Some(url) => url,
//                         None => String::new(),
//                     }},
//                     None => String::new(),
//                 };
//                 message.channel_id.send_message(|m| m
//                     .embed(|e| e
//                         .title(manga.attributes.canonical_title.clone())
//                         .url(manga.url())
//                         .descriptionription(format!("{}\n\n**Volumes:** {}\n**Chapters:** {}\n**Score:** {}\n**Status:** {}",
//                             manga.attributes.synopsis,
//                             manga.attributes.volume_count.map_or(String::from("Not Found"), |count| count.to_string()),
//                             manga.attributes.chapter_count.map_or(String::from("Not Found"), |count| count.to_string()),
//                             manga.attributes.average_rating.clone().unwrap_or(String::from("Not Found")),
//                             status
//                         ))
//                         .thumbnail(cover_url)
//                         .colour(*colours::MAIN)
//                 ))?;
//             }
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct Now;
// impl Command for Now {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Current time. Optionally provide an amount of hours to offset by.".to_string()),
//             usage: Some("[hour]".to_string()),
//             example: Some("-5".to_string()),
//             aliases: vec!["time"].iter().map(|e| e.to_string()).collect(),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         use chrono::offset::FixedOffset;
//         let utc = Utc::now();
//         let datetime = match args.single::<i32>() {
//             Ok(data) => {
//                 let tz = FixedOffset::east(data * 3600);
//                 utc.with_timezone(&tz)
//             },
//             Err(_) => {
//                 let tz = FixedOffset::east(0);
//                 utc.with_timezone(&tz)
//             },
//         };

//         let time = datetime.format("%H:%M").to_string();
//         let date = datetime.format("%A %e %B %Y").to_string();
//         message.channel_id.send_message(|m| m
//             .embed(|e| e
//                 .colour(*colours::MAIN)
//                 .descriptionription(format!("**Time:** {}\n**Date:** {}\n**Timezone:** UTC{}", time, date, datetime.timezone()))
//         ))?;
//         Ok(())
//     }
// }

//TODO how to get shard latency here?
pub struct Ping;
#[async_trait]
impl Command for Ping {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Make sure the bot is alive.".to_string()),
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        use chrono::DateTime;
        let shard_latency = {
            let info = ctx.cluster.info();
            // This only works in a single sharded situation
            info.get(&0).and_then(|info| info.latency().average())
                .map(|dur| dur.as_millis().to_string())
                .unwrap_or("unknown".to_string())
        };
        let embed = EmbedBuilder::new()
            .title("Pong!")
            .color(colors::MAIN)
            .description(format!("**Shard Latency:** {}", shard_latency));
        let response = ctx.http.create_message(message.channel_id).reply(message.id)
            .embed(embed.clone().build()?)?
            .await?;
        let rtt =
            DateTime::parse_from_rfc3339(response.timestamp.as_str()).map(|ts| ts.timestamp_millis()).unwrap_or(0)
            - DateTime::parse_from_rfc3339(message.timestamp.as_str()).map(|ts| ts.timestamp_millis()).unwrap_or(0);
        let embed = embed
            .description(format!("**Shard Latency:** {} ms\n**HTTP Response Time:** {} ms", shard_latency, rtt));
        ctx.http.update_message(response.channel_id, response.id).embed(embed.build()?)?.await?;
        
        Ok(())
    }
}

pub struct Prefix;
#[async_trait]
impl Command for Prefix {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Echoes the prefix of the current guild.".to_string()),
            guild_only: true,
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, _: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(guild_id) = message.guild_id {
            if let Ok(settings) = ctx.db.get_guild(guild_id.0 as i64) {
                ctx.http.create_message(message.channel_id).reply(message.id).content(format!("The prefix for this guild is `{}`", settings.prefix))?.await?;
            } else {
                ctx.http.create_message(message.channel_id).reply(message.id).content("Failed to get guild data.")?.await?;
            }
        }
        Ok(())
    }
}

pub struct Reminder;
#[async_trait]
impl Command for Reminder {
    fn options(&self) -> Arc<Options> {
        let options = Options {
            description: Some("Set a reminder. The reminder is sent to whatever channel it originated in.".to_string()),
            usage: Some("<reminder text> </t time_resolvable>".to_string()),
            examples: vec!["do the thing /t 1 day 10 min 25 s".to_string()],
            ..Options::default()
        };
        Arc::new(options)
    }

    async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
        let channel_id = message.channel_id;
        let user_id = message.author.id;
        let switches = get_switches(args.rest().to_string());
        let reminder = match switches.get("rest") {
            Some(s) => s.clone(),
            None => String::new(),
        };
        let start_time = Utc::now().timestamp();
        let dur = hrtime_to_seconds(match switches.get("t") {
            Some(s) => s.clone(),
            None => String::new(),
        });
        if dur>0 {
            let end_time = start_time + dur;
            let reminder_fmt = format!("REMINDER||{}||{}||{}||{}", channel_id.0, user_id.0, dur, reminder);
            ctx.db.new_timer(start_time, end_time, reminder_fmt.clone())?;
            ctx.tc.request();
            ctx.http.create_message(channel_id).content(format!("Got it! I'll remind you to {} in {}",
                reminder,
                seconds_to_hrtime(dur as usize)
            ))?.await?;
        } else {
            ctx.http.create_message(channel_id).content("Sorry, I wasn't able to find a time there. Make sure you to add `/t time_resolvable` after your reminder text.")?.await?;
        }

        Ok(())
    }
}

// pub struct RoleInfo;
// impl Command for RoleInfo {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Information about a role.".to_string()),
//             usage: Some("<role_resolvable>".to_string()),
//             example: Some("@example role".to_string()),
//             guild_only: true,
//             aliases: vec!["ri", "rinfo"].iter().map(|e| e.to_string()).collect(),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             match parse_role(args.rest().to_string(), guild_id) {
//                 Some((role_id, role)) => {
//                     let role_data = db.get_role(role_id.0 as i64, guild_id.0 as i64).ok();
//                     let mut fields = vec![
//                         ("Name", role.name.clone(), true),
//                         ("ID", role_id.0.to_string(), true),
//                         ("Hex", format!("#{}", role.colour.hex()), true),
//                         ("Hoisted", String::from(if role.hoist { "Yes" } else { "No" }), true),
//                         ("Mentionable", String::from(if role.mentionable { "Yes" } else { "No" }), true),
//                         ("Position", role.position.to_string(), true),
//                     ];
//                     match role_data {
//                         Some(r) => {
//                             fields.push(("Self Assignable", String::from("Yes"), true));
//                             if !r.aliases.is_empty() {
//                                 fields.push(("Self Role Aliases", r.aliases.join(", "), true));
//                             }
//                         },
//                         None => {
//                             fields.push(("Self Assignable", String::from("No"), true));
//                         }
//                     }
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .thumbnail(format!("https://www.colorhexa.com/{}.png", role.colour.hex().to_lowercase()))
//                             .colour(role.colour)
//                             .fields(fields)
//                     ))?;
//                 },
//                 None => { message.channel_id.say("Unable to find that role.")?; }
//             }
//         } else { failed!(GUILDID_FAIL); }
//         Ok(())
//     }
// }

// // TODO eval expressions such as "2d10 + 5"
// pub struct Roll;
// impl Command for Roll {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Roll some dice. Defaults to 6-sided.".to_string()),
//             usage: Some("<Nd>[X]".to_string()),
//             example: Some("2d10".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let expr = args.single::<String>().unwrap_or(String::new());
//         if let Some(caps) = DICE_MATCH.captures(expr.as_str()) {
//             let count: u32 = caps["count"].parse().unwrap_or(1);
//             let sides: u32 = caps["sides"].parse().unwrap_or(6);
//             if count > 0 && count <= 1000 {
//                 if sides > 0 && sides <= 100 {
//                     let mut total = 0;
//                     for _ in 1..&count+1 {
//                         let r = thread_rng().gen_range(1,&sides+1);
//                         total += r;
//                     }
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .colour(*colours::MAIN)
//                             .field(format!("{} 🎲 [1-{}]", count, sides), format!("You rolled {}", total), true)
//                     ))?;
//                 } else { message.channel_id.say("Sides out of bounds. Max: 100")?; }
//             } else { message.channel_id.say("Count out of bounds. Max: 1000")?; }
//         } else { message.channel_id.say("Sorry, I didn't understand your input.")?; }
//         Ok(())
//     }
// }

// pub struct ServerInfo;
// impl Command for ServerInfo {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Information about the current server (guild).".to_string()),
//             guild_only: true,
//             aliases: vec!["si", "sinfo"].iter().map(|e| e.to_string()).collect(),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         use serenity::model::channel::ChannelType::*;
//         use serenity::model::user::OnlineStatus::*;

//         let switches = get_switches(args.full().to_string());
//         let g = match switches.get("rest") {
//             Some(s) => {
//                 if let Some((_, lock)) = parse_guild(s.to_string()) {
//                     Some(lock)
//                 } else {
//                     None
//                 }
//             },
//             None => message.guild()
//         };
//         if let Some(guild_lock) = g {
//             let guild = guild_lock.read().clone();
//             match switches.get("roles") {
//                 None => {
//                     let mut channels = (0,0,0);
//                     for (_, channel_lock) in guild.channels.iter() {
//                         let mut channel = channel_lock.read();
//                         match channel.kind {
//                             Text => { channels.0 += 1; },
//                             Voice => { channels.1 += 1; },
//                             Category => { channels.2 += 1; },
//                             Group => {},
//                             Private => {},
//                         }
//                     }
//                     let mut members = (0,0,0);
//                     for (user_id, _) in guild.members.iter() {
//                         match user_id.to_user() {
//                             Ok(u) => {
//                                 if u.bot {
//                                     members.1 += 1;
//                                 } else {
//                                     members.0 += 1;
//                                 }
//                             },
//                             Err(_) => {},
//                         }
//                     }
//                     for (_, presence) in guild.presences.iter() {
//                         match presence.status {
//                             DoNotDisturb => { members.2 += 1; },
//                             Idle => { members.2 += 1; },
//                             Invisible => {},
//                             Offline => {},
//                             Online => { members.2 += 1; },
//                         }
//                     }
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .thumbnail(guild.icon_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()))
//                             .color(*colours::MAIN)
//                             .field("ID", guild.id, true)
//                             .field("Name", &guild.name, true)
//                             .field("Owner", guild.owner_id.mention(), true)
//                             .field("Region", guild.region, true)
//                             .field(format!("Channels [{}]", guild.channels.len()), format!("Categories: {}\nText: {}\nVoice: {}", channels.2, channels.0, channels.1), true)
//                             .field(format!("Members [{}/{}]", members.2, guild.members.len()), format!("Humans: {}\nBots: {}", members.0, members.1), true)
//                             .field("Created", guild.id.created_at().format("%a, %d %h %Y @ %H:%M:%S").to_string(), false)
//                             .field("Roles", guild.roles.len(), true)
//                             .field("Emojis", guild.emojis.len(), true)
//                             .title(guild.name)
//                     ))?;
//                 },
//                 Some(_) => {
//                     let mut roles_raw = guild.roles.values().collect::<Vec<&Role>>();
//                     roles_raw.sort_by(|a, b| b.position.cmp(&a.position));
//                     let roles = roles_raw.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .title(format!("Roles for {}. Count: {}", guild.name, roles.len()))
//                             .descriptionription(roles.join("\n"))
//                             .colour(*colours::BLUE)
//                     ))?;
//                 },
//             }
//         } else { message.channel_id.say("Could not find that guild.")?; }
//         Ok(())
//     }
// }

// pub struct Urban;
// impl Command for Urban {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Look something up on UrbanDictionary.".to_string()),
//             usage: Some(r#"<"term"> [count]"#.to_string()),
//             example: Some(r#""boku no pico" 5"#.to_string()),
//             aliases: vec!["ud", "urbandict"].iter().map(|e| e.to_string()).collect(),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let api = {
//             let data = ctx.data.lock();
//             data.get::<ApiClient>().cloned()
//         };
//         if let Some(api) = api {
//             let term = args.single_quoted::<String>().unwrap_or(String::new());
//             let res = api.urban(term.as_str())?;
//             if !res.definitions.is_empty() {
//                 let count = args.single::<u32>().unwrap_or(1);
//                 let mut tags: Vec<String> = Vec::new();
//                 if let Some(res_tags) = &res.tags {
//                     tags = res_tags.clone();
//                     tags.sort();
//                     tags.dedup();
//                 }
//                 if count == 1 {
//                     let item = &res.definitions[0];
//                     let tags_list = {
//                         let list = tags.iter().map(|t| "#".to_string()+t).collect::<Vec<String>>().join(", ");
//                         if !list.is_empty() {
//                             list
//                         } else {
//                             "None".to_string()
//                         }
//                     };
//                     let definition = {
//                         let mut i = item.definition.clone();
//                         if i.len() > 1000 {
//                             i.truncate(997);
//                             i += "...";
//                         }
//                         i
//                     };
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .colour(*colours::MAIN)
//                             .field(format!(r#"Definition of "{}" by {}"#, item.word, item.author), &item.permalink, false)
//                             .field("Thumbs Up", &item.thumbs_up, true)
//                             .field("Thumbs Down", &item.thumbs_down, true)
//                             .field("Definition", definition, false)
//                             .field("Example", &item.example, false)
//                             .field("Tags", tags_list, false)
//                     ))?;
//                 } else {
//                     let mut list = res.definitions;
//                     list.truncate(count as usize);
//                     let list = list.iter()
//                         .map(|c| format!(r#""{}" by {}: {}"#, c.word, c.author, c.permalink))
//                         .collect::<Vec<String>>()
//                         .join("\n");
//                     message.channel_id.send_message(|m| m
//                         .embed(|e| e
//                             .title(format!("Top {} results for {}", count, term))
//                             .descriptionription(list)
//                             .colour(*colours::MAIN)
//                     ))?;
//                 }
//             }
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct UserId;
// impl Command for UserId {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Get the unique ID of a user.".to_string()),
//             usage: Some("[user_resolvable]".to_string()),
//             example: Some("@Adelyn".to_string()),
//             guild_only: true,
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             if let Some((id,_)) = parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
//                 message.channel_id.say(format!("{}", id.0))?;
//             } else {
//                 message.channel_id.say("I couldn't find that user.")?;
//             }
//         }
//         Ok(())
//     }
// }

// pub struct UserInfo;
// impl Command for UserInfo {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Information about a user. Defaults to the author of the command.".to_string()),
//             usage: Some("[user_resolvable]".to_string()),
//             example: Some("@Adelyn".to_string()),
//             guild_only: true,
//             aliases: vec!["ui", "uinfo"].iter().map(|e| e.to_string()).collect(),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         if let Some(guild_id) = message.guild_id {
//             let (user, member) = match parse_user(args.single::<String>().unwrap_or(String::new()), guild_id) {
//                 Some((id, member)) => (id.to_user()?, member),
//                 None => (message.author.clone(), message.member().ok_or("Failed to get member.")?),
//             };
//             let user_data = db.get_user(user.id.0 as i64, guild_id.0 as i64)?;
//             let mut roles = member.roles.iter()
//                 .map(|c| match c.to_role_cached() {
//                     Some(r) => r.name,
//                     None => c.0.to_string(),
//                 })
//                 .collect::<Vec<String>>();
//             roles.sort();
//             let roles = if roles.is_empty() {
//                 "None".to_string()
//             } else {
//                 roles.join(", ")
//             };
//             let dates = format!(
//                 "Created: {}\nJoined: {}{}",
//                 user.created_at()
//                     .format("%a, %d %h %Y @ %T")
//                     .to_string(),
//                 member.joined_at
//                     .and_then(|t| Some(t.with_timezone(&Utc)))
//                     .unwrap_or(Utc::now())
//                     .format("%a, %d %h %Y @ %T")
//                     .to_string(),
//                 user_data.registered.map_or(String::new(), |r| {
//                     format!("\nRegistered: {}", r
//                         .format("%a, %d %h %Y @ %T")
//                         .to_string())
//                 })
//             );
//             message.channel_id.send_message(|m| m
//                 .embed(|e| e
//                     .colour(member.colour().unwrap_or(*colours::MAIN))
//                     .thumbnail(user.face())
//                     .title(&user.tag())
//                     .field("ID", user.id, true)
//                     .field("Mention", user.mention(), true)
//                     .field("Nickname", member.display_name().into_owned(), true)
//                     .field("Dates", dates, false)
//                     .field(format!("Roles [{}]", member.roles.len()), roles, false)
//             ))?;
//         }
//         Ok(())
//     }
// }

// pub struct Weather;
// impl Command for Weather {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             bucket: Some("weather".to_string()),
//             description: Some("Check on the current weather at a given city. By default this will use the units used at that location, but units can be manually selected. Options are si, us, uk, ca".to_string()),
//             usage: Some("<city name> [/unit]".to_string()),
//             example: Some("london /us".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let data = ctx.data.lock();
//         if let Some(api) = data.get::<ApiClient>() {
//             let switches = get_switches(args.full().to_string());
//             let rest = switches.get("rest");
//             // TODO Refactor this
//             let mut units = Units::Auto;
//             if switches.len() > 1 {
//                 switches.keys().for_each(|k| {
//                     match k.as_str() {
//                         "uk" => { units = Units::UK; },
//                         "c" | "ca"  => { units = Units::CA; },
//                         "si" => { units = Units::SI; },
//                         "us" => { units = Units::Imperial; },
//                         _ => {},
//                     }
//                 });
//             }
//             message.channel_id.broadcast_typing()?;
//             if let Some(loc) = rest {
//                 match api.weather(loc, units) {
//                     Some((city_info, Ok(body))) => {
//                         if let Some(current) = body.currently {
//                             if let Some(daily_data) = body.daily {
//                                 let daily = &daily_data.data[0];
//                                 let temp = current.temperature.unwrap_or(NAN);
//                                 let temp_high = current.temperature_high.unwrap_or(daily.temperature_high.unwrap_or(NAN));
//                                 let temp_low = current.temperature_low.unwrap_or(daily.temperature_low.unwrap_or(NAN));
//                                 let feels_like = current.apparent_temperature.unwrap_or(NAN);
//                                 let wind = current.wind_speed.unwrap_or(NAN);
//                                 let visi = current.visibility.unwrap_or(NAN);
//                                 let pressure = current.pressure.unwrap_or(NAN);
//                                 let humidity = current.humidity.unwrap_or(NAN)*100.0;
//                                 let icon = match current.icon {
//                                     Some(ic) => {
//                                         match ic {
//                                             ClearDay => "The sky is clear",
//                                             ClearNight => "The sky is clear",
//                                             Rain => "It is raining",
//                                             Snow => "It is snowing",
//                                             Sleet => "It is sleeting",
//                                             Wind => "It is windy",
//                                             Fog => "It is foggy",
//                                             Cloudy => "The sky is cloudy",
//                                             PartlyCloudyDay => "The sky is partly cloudy",
//                                             PartlyCloudyNight => "The sky is partly cloudy",
//                                             Hail => "It is hailing",
//                                             Thunderstorm => "There is a thunderstorm",
//                                             Tornado => "There is a tornado",
//                                         }
//                                     },
//                                     None => "The sky is clear",
//                                 };
//                                 let response_units = body.flags.and_then(|e| Some(e.units)).unwrap_or(Units::Imperial);
//                                 let (temp_unit, speed_unit, dist_unit) = match response_units {
//                                     Units::SI => { ("C", "m/s", "km") },
//                                     Units::CA => { ("C", "kmph", "km") },
//                                     Units::UK => { ("C", "mph", "mi") },
//                                     _ => { ("F", "mph", "mi") },
//                                 };
//                                 message.channel_id.send_message(|m| m
//                                     .embed(|e| e
//                                         .title(format!("Weather in {}", city_info))
//                                         .descriptionription(format!("_It is currently **{}°{temp}** with wind of **{} {speed}** making it feel like **{}°{temp}**. {} with a visibility of about **{} {dist}**._",
//                                             temp,
//                                             wind,
//                                             feels_like,
//                                             icon,
//                                             visi,
//                                             temp = temp_unit,
//                                             speed = speed_unit,
//                                             dist = dist_unit
//                                         ))
//                                         .field("Temperature", format!(
//                                             "Current: **{}°{temp}**\nLow/High: **{}°{temp} / {}°{temp}**",
//                                             temp,
//                                             temp_low,
//                                             temp_high,
//                                             temp = temp_unit
//                                         ), true)
//                                         .field("Wind Chill", format!(
//                                             "Feels Like: **{}°{temp}**\nWind Speed: **{} {speed}**",
//                                             feels_like,
//                                             wind,
//                                             temp = temp_unit,
//                                             speed = speed_unit
//                                         ), true)
//                                         .field("Atmosphere", format!(
//                                             "Humidity: **{}%**\nPressure: **{} mbar**",
//                                             humidity,
//                                             pressure,
//                                         ), true)
//                                         .colour(*colours::MAIN)
//                                         .timestamp(now!())
//                                         .footer(|f| f.text("Forecast by Dark Sky"))
//                                 ))?;
//                             }
//                         }
//                     },
//                     Some((_, Err(why))) => {
//                         message.channel_id.say(format!("Something went wrong while getting the forecast.\n{}", why))?;
//                     },
//                     None => {
//                         message.channel_id.say("An error occurred while resolving the location.")?;
//                     },
//                 }
//             } else { message.channel_id.say("Please enter a location.")?; }
//         } else { failed!(API_FAIL); }
//         Ok(())
//     }
// }

// pub struct Stats;
// impl Command for Stats {
//     fn options(&self) -> Arc<Options> {
//         let default = Options::default();
//         let options = Options {
//             description: Some("Display some interesting, but useless statistics.".to_string()),
//             ..Options::default()
//         };
//         Arc::new(options)
//     }

//     async fn run(&self, message: Message, args: Args, ctx: Context) -> Result<(), Box<dyn Error + Send + Sync>> {
//         let (cached_guilds
//             ,cached_channels
//             ,cached_users
//             ,cached_messages) = {
//                 let cache = CACHE.read();
//                 (cache.guilds.len()
//                 ,cache.channels.len()
//                 ,cache.users.len()
//                 ,cache.messages.values()
//                     .fold(0, |a,m| {
//                         a + m.len()
//                     }))
//             };
//         let (db_guilds
//             ,db_users
//             ,db_notes
//             ,db_roles
//             ,db_timers
//             ,db_cases
//             ,db_tags
//             ,db_hackbans
//             ,db_premium) = {
//                 (db.count_guilds().unwrap_or(-1)
//                 ,db.count_users().unwrap_or(-1)
//                 ,db.count_notes().unwrap_or(-1)
//                 ,db.count_roles().unwrap_or(-1)
//                 ,db.count_timers().unwrap_or(-1)
//                 ,db.count_cases().unwrap_or(-1)
//                 ,db.count_tags().unwrap_or(-1)
//                 ,db.count_hackbans().unwrap_or(-1)
//                 ,db.count_premium().unwrap_or(-1)
//                 )
//             };
//         message.channel_id.send_message(|m| m
//             .embed(|e| e
//                 .title("Bot Stats")
//                 .field("Cache", format!(
//                     "Guilds: {}\nChannels: {}\nUsers: {}\nMessages: {}"
//                     ,cached_guilds
//                     ,cached_channels
//                     ,cached_users
//                     ,cached_messages
//                 ), false)
//                 .field("Database", format!(
//                     "Guilds: {}\nUsers: {}\nNotes: {}\nSelf Roles: {}\nTimers: {}\nCases: {}\nTags: {}\nHackbans: {}\nPremium Guilds: {}"
//                     ,db_guilds
//                     ,db_users
//                     ,db_notes
//                     ,db_roles
//                     ,db_timers
//                     ,db_cases
//                     ,db_tags
//                     ,db_hackbans
//                     ,db_premium
//                 ), false)
//                 .field("More coming soon", "...", false)
//                 .colour(*colours::MAIN)
//                 .timestamp(now!())
//         ))?;
//         Ok(())
//     }
// }

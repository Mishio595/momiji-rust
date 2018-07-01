use serenity::CACHE;
use serenity::prelude::*;
use serenity::model::channel::*;
use serenity::model::id::*;
use serenity::utils::Colour;
use chrono::offset::Utc;
use sysinfo;
use sysinfo::{ProcessorExt, SystemExt, ProcessExt};
use sys_info;
use rand::prelude::*;
use ::utils::*;

// Rank 0

//TODO: better info
command!(bot_info(ctx, message, _args) {
    let mut data = ctx.data.lock();
    let cache = CACHE.read();
    let shard_count = match data.get::<::SerenityShardManager>() {
        Some(s) => s.lock().shards_instantiated().len(),
        None => {
            error!("Unable to get the shard manager!");
            0
        },
    };
    let owner = data.get::<::Owner>().unwrap().get().unwrap();
    let sys = sysinfo::System::new();
    let process = sys.get_process(sysinfo::get_current_pid()).unwrap();

    if let Err(why) = message.channel_id.send_message(|m| m
        .embed(|e| e
            .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
            .field("Owner", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
            .field("Links", "[Momiji's House](https://discord.gg/YYdpsNc)\n[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)\n[Github](https://github.com/Mishio595/momiji-rust\n)[Patreon](https://www.patreon.com/momijibot)", true)
            .field("Counts", format!("Guilds: {}\nShards: {}", cache.guilds.len(), shard_count), true)
            .field("System Info", format!("OS: {} {}\nUptime: {}",
                sys_info::os_type().unwrap(),
                sys_info::os_release().unwrap(),
                sys.get_uptime()), true)
            .thumbnail(&cache.user.avatar_url().unwrap_or(cache.user.default_avatar_url()))
            .colour(Colour::new(6138367))
        )
    ) {
        error!("Failed to send message: {:?}", why);
    };
});

command!(cat(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<::ApiClient>().unwrap().cat() {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(res.file)));
    };
});

command!(color(ctx, message, args) {
});

command!(danbooru(ctx, message, args) {
});

command!(dog(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<::ApiClient>().unwrap().dog() {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(res.message)));
    };
});

command!(dad_joke(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<::ApiClient>().unwrap().joke() {
        message.channel_id.say(res);
    };
});

command!(e621(ctx, message, args) {
    let mut data = ctx.data.lock();
    match data.get::<::ApiClient>().unwrap().furry(args.full(), 1) {
        Ok(res) => {
        let post = &res[0];
        let o = message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(&post.file_url)
                .description(format!("**Tags:** {}\n**Post:** [{}]({})\n**Artist:** {}\n**Score:** {}",
                    &post.tags,
                    &post.id,
                    format!("https://e621.net/post/show/{}", &post.id),
                    &post.artist[0],
                    &post.score
                ))
            ));
        },
        Err(why) => { error!("{:?}", why); },
    }
});

command!(anime(ctx, message, args) {
});

command!(manga(ctx, message, args) {
});

command!(now(_ctx, message, args) {
    use chrono::offset::{FixedOffset, Utc};
    let utc = Utc::now();
    let datetime = match args.single::<i32>() {
        Ok(data) => {
            let tz = FixedOffset::east(data * 3600);
            utc.with_timezone(&tz)
        },
        Err(_) => {
            let tz = FixedOffset::east(0);
            utc.with_timezone(&tz)
        },
    };

    let time = datetime.format("%H:%M").to_string();
    let date = datetime.format("%A %e %B %Y").to_string();
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(Colour::new(6138367))
            .description(format!("**Time:** {}\n**Date:** {}\n**Timezone:** UTC{}", time, date, datetime.timezone())))
    );
});

command!(ping(_ctx, message, _args) {
    if let Ok(mut m) = message.channel_id.say("Pong!") {
        let t = m.timestamp.timestamp_millis() - message.timestamp.timestamp_millis();
        let _ = m.edit(|m| m.content(format!("Pong! `{} ms`", t)));
    };
});

command!(prefix(ctx, message, args) {
});

command!(remind(ctx, message, args) {
});

command!(asr(ctx, message, args) {
});

command!(rsr(ctx, message, args) {
});

command!(lsr(ctx, message, args) {
});

command!(role_info(_ctx, message, args) {
    if let Some(id) = parse_role(args.single::<String>().unwrap()) {
        let role = id.find().unwrap(); //unsafe, needs error checked
        if let Err(why) = message.channel_id.send_message(|m| m
            .embed(|e| e
                .thumbnail(format!("https://www.colorhexa.com/{}.png", role.colour.hex().to_lowercase()))
                .colour(role.colour)
                .field("Name", role.name, true)
                .field("ID", role.id, true)
                .field("Hex", format!("#{}", role.colour.hex()), true)
                .field("Hoisted", { if role.hoist { "Yes" } else { "No" } }, true)
                .field("Mentionable", { if role.mentionable { "Yes" } else { "No" } }, true)
                .field("Position", role.position, true)
        )) {

        };
    };
});

command!(roll(_ctx, message, args) {
    let expr = args.single::<String>().unwrap_or(String::new());
    let mut iter = expr.split(|c| c == 'd' || c == 'D');
    let count: u32 = iter.next().unwrap_or("0").parse().unwrap();
    let sides: u32 = iter.next().unwrap_or("0").parse().unwrap();
    if count>0 && sides>0 {
        let mut total = 0;
        for _ in 1..&count+1 {
            let r = thread_rng().gen_range(1,&sides+1);
            total += r;
        }
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(Colour::new(6138367))
                .field(format!("{} 🎲 [1-{}]", count, sides), format!("You rolled {}", total), true)
        ));
    }
});

command!(server_info(_ctx, message, _args) {
    use serenity::model::channel::ChannelType::*;
    use serenity::model::user::OnlineStatus::*;
    if let Some(guild_lock) = message.guild() {
        let guild = guild_lock.read().clone();
        let mut channels = (0,0,0);
        for (_, channel_lock) in guild.channels.iter() {
            let mut channel = channel_lock.read();
            match channel.kind {
                Text => { channels.0 += 1; },
                Voice => { channels.1 += 1; },
                Category => { channels.2 += 1; },
                Group => {},
                Private => {},
            }
        }
        let mut members = (0,0,0);
        for (user_id, _) in guild.members.iter() {
            match user_id.get() {
                Ok(u) => {
                    if u.bot {
                        members.1 += 1;
                    } else {
                        members.0 += 1;
                    }
                },
                Err(why) => {},
            }
        }
        for (_, presence) in guild.presences.iter() {
            match presence.status {
                DoNotDisturb => { members.2 += 1; },
                Idle => { members.2 += 1; },
                Invisible => {},
                Offline => {},
                Online => { members.2 += 1; },
            }
        }
        if let Err(why) = message.channel_id.send_message(|m| m
            .embed(|e| e
                .thumbnail(guild.icon_url().unwrap())
                .color(Colour::new(6138367))
                .field("ID", guild.id, true)
                .field("Name", &guild.name, true)
                .field("Owner", guild.owner_id.mention(), true)
                .field("Region", guild.region, true)
                .field(format!("Channels [{}]", guild.channels.len()), format!("Categories: {}\nText: {}\nVoice: {}", channels.2, channels.0, channels.1), true)
                .field(format!("Members [{}/{}]", members.2, guild.members.len()), format!("Humans: {}\nBots: {}", members.0, members.1), true)
                .field("Roles", guild.roles.len(), true)
                .field("Emojis", guild.emojis.len(), true)
                .title(guild.name)
        )) {
            error!("Unable to send message: {:?}", why);
        };
    }
});

command!(tag(ctx, message, args) {
});

command!(urban(ctx, message, args) {
    let mut data = ctx.data.lock();
    let term = args.single_quoted::<String>().unwrap_or(String::new());
    if let Ok(mut res) = data.get::<::ApiClient>().unwrap().urban(term.as_str()) {
        if !res.list.is_empty() {
            let count = args.single::<u32>().unwrap_or(1);
            res.tags.dedup();
            if count == 1 {
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .colour(Colour::new(6138367))
                        .field(format!(r#"Definition of "{}" by {}"#, res.list[0].word, res.list[0].author), &res.list[0].permalink, false)
                        .field("Thumbs Up", &res.list[0].thumbs_up, true)
                        .field("Thumbs Down", &res.list[0].thumbs_down, true)
                        .field("Definition", &res.list[0].definition, false)
                        .field("Example", &res.list[0].example, false)
                        .field("Tags", res.tags.iter().map(|t| { String::from("#")+t }).collect::<Vec<String>>().join(", "), false)));
            } else {
                res.list.truncate(count as usize);
                let list = res.list.iter()
                    .map(|c| format!(r#""{}" by {}: {}"#, c.word, c.author, c.permalink))
                    .collect::<Vec<String>>()
                    .join("\n");
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .title(format!("Top {} results for {}", count, term))
                        .description(list)
                        .colour(Colour::new(6138367))
                    ));
            }
        }
    };
});

command!(user_info(_ctx, message, args) {
    if let Some(guild_lock) = message.guild() {
        let guild = guild_lock.read();
        let user = match parse_user(args.single::<String>().unwrap_or(String::new())) {
            Some(id) => id.get().unwrap(),
            None => message.author.clone(),
        };
        let member = match guild.member(user.id) {
            Ok(member) => member,
            Err(_) => message.member().unwrap(),
        };
        let roles = member.roles.iter().map(|c| c.find().unwrap().name).collect::<Vec<String>>().join(", ");
        if let Err(why) = message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(member.colour().unwrap())
                .thumbnail(user.face())
                .title(&user.tag())
                .field("ID", user.id, true)
                .field("Mention", user.mention(), true)
                .field("Nickname", member.nick.unwrap_or(user.name.clone()), true)
                .field("Dates", format!("Created: {}\nJoined: {}", user.created_at(), member.joined_at.unwrap()), false)
                .field(format!("Roles [{}]", member.roles.len()), roles, false)
        )) {
            error!("Unable to send message: {:?}", why);
        };
    };
});

command!(weather(ctx, message, args) {
});

// Rank 1

command!(mod_info(ctx, message, args) {
});

command!(mute(ctx, message, args) {
});

command!(unmute(ctx, message, args) {
});

command!(notes(ctx, message, args) {
});

command!(register(ctx, message, args) {
});

command!(ar(ctx, message, args) {
});

command!(rr(ctx, message, args) {
});

command!(role_colour(ctx, message, args) {
});

command!(watchlist(ctx, message, args) {
});

// Rank 2

command!(config(ctx, message, args) {
});

command!(hackban(ctx, message, args) {
});

command!(ignore(ctx, message, args) {
});

command!(csr(ctx, message, args) {
});

command!(dsr(ctx, message, args) {
});

command!(prune(ctx, message, args) {
});

command!(test(ctx, message, args) {
});

// Rank 3

command!(setup_mute(ctx, message, args) {
});

// Rank 4

command!(git(ctx, message, args) {
});

command!(log(ctx, message, args) {
});

command!(restart(ctx, message, args) {
});

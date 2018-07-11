use std::collections::HashMap;
use std::str::FromStr;
use serenity::CACHE;
use serenity::prelude::*;
use serenity::model::id::*;
use serenity::model::channel::Message;
use serenity::client::bridge::gateway::ShardId;
use serenity::builder::GetMessages;
use serenity::framework::standard::CommandError as Error;
use sysinfo;
use sysinfo::SystemExt;
use sys_info;
use rand::prelude::*;
use ::core::utils::*;
use ::core::model::*;
use ::core::consts::*;
use chrono::Utc;

// Rank 0

// TODO: better info
command!(bot_info(ctx, message, _args) {
    let mut data = ctx.data.lock();
    let cache = CACHE.read();
    let shard_count = cache.shard_count;
    let owner = data.get::<Owner>().expect("Failed to get owner from sharemap").get().expect("Failed to get user from id");
    let sys = sysinfo::System::new();
    //let process = sys.get_process(sysinfo::get_current_pid()).unwrap();

    message.channel_id.send_message(|m| m
        .embed(|e| e
            .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
            .field("Owner", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
            .field("Links", "[Momiji's House](https://discord.gg/YYdpsNc)\n[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)\n[Github](https://github.com/Mishio595/momiji-rust)\n[Patreon](https://www.patreon.com/momijibot)", true)
            .field("Counts", format!("Guilds: {}\nShards: {}", cache.guilds.len(), shard_count), true)
            .field("System Info", format!("OS: {} {}\nUptime: {}",
                sys_info::os_type().unwrap(),
                sys_info::os_release().unwrap(),
                seconds_to_hrtime(sys.get_uptime() as usize)), false)
            .thumbnail(&cache.user.avatar_url().unwrap_or(cache.user.default_avatar_url()))
            .colour(Colours::Main.val())
        )).expect("Failed to send message");
    // return Err(Error(String::from("test"));
});

command!(cat(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<ApiClient>().expect("Failed to get API Client").cat() {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(res.file))).expect("Failed to send message");
    };
});

command!(color(ctx, message, args) {
});

command!(danbooru(ctx, message, args) {
});

command!(dog(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<ApiClient>().expect("Failed to get API Client").dog() {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(res.message))).expect("Failed to send message");
    };
});

command!(dad_joke(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<ApiClient>().expect("Failed to get API Client").joke() {
        message.channel_id.say(res).expect("Failed to send message");
    };
});

command!(e621(ctx, message, args) {
    let mut data = ctx.data.lock();
    match data.get::<ApiClient>().expect("Failed to get API Client").furry(args.full(), 1) {
        Ok(res) => {
        let post = &res[0];
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(&post.file_url)
                .description(format!("**Tags:** {}\n**Post:** [{}]({})\n**Artist:** {}\n**Score:** {}",
                    &post.tags,
                    &post.id,
                    format!("https://e621.net/post/show/{}", &post.id),
                    &post.artist[0],
                    &post.score
                ))
            )).expect("Failed to send message");
        },
        Err(why) => { error!("{:?}", why); },
    }
});

command!(anime(ctx, message, args) {
});

command!(manga(ctx, message, args) {
});

command!(now(_ctx, message, args) {
    use chrono::offset::FixedOffset;
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
            .colour(Colours::Main.val())
            .description(format!("**Time:** {}\n**Date:** {}\n**Timezone:** UTC{}", time, date, datetime.timezone())))
    ).expect("Failed to send message");
});

command!(ping(ctx, message, _args) {
    let data = ctx.data.lock();
    let sm = data.get::<SerenityShardManager>().expect("Failed to get Shard Manager").lock();
    let lat = match sm.runners.lock().get(&ShardId(ctx.shard_id)).expect("Failed to get shard runner").latency {
        Some(la) => { la.as_secs() as u32 + la.subsec_millis() },
        None => 0,
    };
    if let Ok(mut m) = message.channel_id.send_message(|m| m.embed(|e| e.title("Pong!"))) {
        let t = m.timestamp.timestamp_millis() - message.timestamp.timestamp_millis();
        let _ = m.edit(|m| m.embed(|e| e
            .title("Pong!")
            .description(format!("**Shard Latency:** {}\n**Response Time:** {} ms", if lat==0 { String::from("Failed to retrieve") } else { format!("{} ms", lat) }, t))
            .colour(Colours::Main.val())
        )).expect("Failed to edit message");
    };
});

command!(prefix(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let settings = db.get_guild(message.guild_id.unwrap().0 as i64).unwrap();
    message.channel_id.say(format!("The prefix for this guild is `{}`", settings.prefix)).expect("Failed to send message");
});

command!(remind(ctx, message, args) {
    let data = ctx.data.lock();
    let tc = data.get::<TC>().expect("Failed to get Timer Client").lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();

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
        let mut reminder_fmt = format!("REMINDER||{}||{}||{}||{}", channel_id.0, user_id.0, dur, reminder);

        match db.new_timer(start_time, end_time, reminder_fmt.clone()) {
            Ok(timer) => {
                reminder_fmt.push_str(format!("||{}", timer.id).as_str());
                tc.request(reminder_fmt, dur as u64);
                message.channel_id.say(format!("Got it! I'll remind you to {} in {}",
                    reminder,
                    seconds_to_hrtime(dur as usize)))
                    .expect("Failed to send message");
            },
            Err(why) => {
                message.channel_id.say(format!("Sorry, I couldn't make the reminder. Here's why: {:?}", why)).expect("Failed to send message");
            },
        }
    }
});

command!(asr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let roles = db.get_roles(guild_id.0 as i64).unwrap();
    let list = args.rest().split(",").map(|s| s.trim().to_string());
    let mut to_add = Vec::new();
    let mut failed = Vec::new();
    let role_names = roles.iter().map(|r| RoleId(r.id as u64).find().unwrap().name.to_lowercase()).collect::<Vec<String>>();
    for r1 in list {
        if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
            to_add.push(s);
        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
            to_add.push(RoleId(roles[i].id as u64));
        } else if let Some(i) = role_names.iter().position(|r| r.contains(&r1.to_lowercase())) {
            to_add.push(RoleId(roles[i].id as u64));
        } else {
            failed.push(format!("Could not locate {}", r1));
        }
    }
    for (i, role_id) in to_add.clone().iter().enumerate() {
        if member.roles.contains(role_id) {
            to_add.remove(i);
            failed.push(format!("You already have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.add_role(*role_id) {
            to_add.remove(i);
            failed.push(format!("Failed to add {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_add.is_empty() {
        fields.push(("Added Roles", format!("{}", to_add.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Add", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Add Self Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap()))).expect("Failed to send message");
});

command!(rsr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let roles = db.get_roles(guild_id.0 as i64).unwrap();
    let list = args.rest().split(",").map(|s| s.trim().to_string());
    let mut to_remove = Vec::new();
    let mut failed = Vec::new();
    let role_names = roles.iter().map(|r| RoleId(r.id as u64).find().unwrap().name.to_lowercase()).collect::<Vec<String>>();
    for r1 in list {
        if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
            to_remove.push(s);
        } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
            to_remove.push(RoleId(roles[i].id as u64));
        } else if let Some(i) = role_names.iter().position(|r| r.contains(&r1.to_lowercase())) {
            to_remove.push(RoleId(roles[i].id as u64));
        } else {
            failed.push(format!("Could not locate {}", r1));
        }
    }
    for (i, role_id) in to_remove.clone().iter().enumerate() {
        if !member.roles.contains(role_id) {
            to_remove.remove(i);
            failed.push(format!("You don't have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.remove_role(*role_id) {
            to_remove.remove(i);
            failed.push(format!("Failed to remove {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_remove.is_empty() {
        fields.push(("Removed Roles", format!("{}", to_remove.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Remove", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Remove Self Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap()))).expect("Failed to send message");
});

// TODO sort list
command!(lsr(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let roles = db.get_roles(guild_id.0 as i64).unwrap();
    let mut map: HashMap<String, Vec<i64>> = HashMap::new();
    for role in roles.iter() {
        if map.contains_key(&role.category) {
            if let Some(v) = map.get_mut(&role.category) {
                v.push(role.id);
            };
        } else {
            map.insert(role.category.clone(), vec![role.id]);
        }
    }
    let mut fields = Vec::new();
    for (key, val) in map.iter() {
        fields.push((key, val.iter().map(|c| RoleId(*c as u64).find().unwrap().name).collect::<Vec<String>>().join("\n"), true));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Self Roles")
            .fields(fields)
            .colour(Colours::Main.val())
        )).expect("Failed to send message");
});

command!(role_info(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    if let Some((_, role)) = parse_role(args.rest().to_string(), guild_id) {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .thumbnail(format!("https://www.colorhexa.com/{}.png", role.colour.hex().to_lowercase()))
                .colour(role.colour)
                .field("Name", role.name, true)
                .field("ID", role.id, true)
                .field("Hex", format!("#{}", role.colour.hex()), true)
                .field("Hoisted", { if role.hoist { "Yes" } else { "No" } }, true)
                .field("Mentionable", { if role.mentionable { "Yes" } else { "No" } }, true)
                .field("Position", role.position, true)
        )).expect("Failed to send message");
    };
});

//TODO parse expr without a d
command!(roll(_ctx, message, args) {
    let expr = args.single::<String>().unwrap_or(String::new());
    let mut iter = expr.split(|c| c == 'd' || c == 'D');
    let count: u32 = iter.next().unwrap_or("0").parse().unwrap_or(0);
    let sides: u32 = iter.next().unwrap_or("6").parse().unwrap_or(6);
    if count>0 {
        let mut total = 0;
        for _ in 1..&count+1 {
            let r = thread_rng().gen_range(1,&sides+1);
            total += r;
        }
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(Colours::Main.val())
                .field(format!("{} 🎲 [1-{}]", count, sides), format!("You rolled {}", total), true)
        )).expect("Failed to send message");
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
                Err(_) => {},
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
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .thumbnail(guild.icon_url().unwrap())
                .color(Colours::Main.val())
                .field("ID", guild.id, true)
                .field("Name", &guild.name, true)
                .field("Owner", guild.owner_id.mention(), true)
                .field("Region", guild.region, true)
                .field(format!("Channels [{}]", guild.channels.len()), format!("Categories: {}\nText: {}\nVoice: {}", channels.2, channels.0, channels.1), true)
                .field(format!("Members [{}/{}]", members.2, guild.members.len()), format!("Humans: {}\nBots: {}", members.0, members.1), true)
                .field("Roles", guild.roles.len(), true)
                .field("Emojis", guild.emojis.len(), true)
                .title(guild.name)
        )).expect("Failed to send message");
    }
});

command!(tag(ctx, message, args) {
});

command!(urban(ctx, message, args) {
    let mut data = ctx.data.lock();
    let term = args.single_quoted::<String>().unwrap_or(String::new());
    if let Ok(mut res) = data.get::<ApiClient>().expect("Failed to get API Client").urban(term.as_str()) {
        if !res.list.is_empty() {
            let count = args.single::<u32>().unwrap_or(1);
            res.tags.dedup();
            if count == 1 {
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .colour(Colours::Main.val())
                        .field(format!(r#"Definition of "{}" by {}"#, res.list[0].word, res.list[0].author), &res.list[0].permalink, false)
                        .field("Thumbs Up", &res.list[0].thumbs_up, true)
                        .field("Thumbs Down", &res.list[0].thumbs_down, true)
                        .field("Definition", &res.list[0].definition, false)
                        .field("Example", &res.list[0].example, false)
                        .field("Tags", res.tags.iter().map(|t| { String::from("#")+t }).collect::<Vec<String>>().join(", "), false))).expect("Failed to send message");
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
                        .colour(Colours::Main.val())
                    )).expect("Failed to send message");
            }
        }
    };
});

command!(user_info(_ctx, message, args) {
    if let Some(guild_lock) = message.guild() {
        let guild = guild_lock.read();
        let (user, member) = match parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
            Some((id, member)) => (id.get().unwrap(), member),
            None => (message.author.clone(), message.member().unwrap().clone()),
        };
        let roles = member.roles.iter().map(|c| c.find().unwrap().name).collect::<Vec<String>>().join(", ");
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(member.colour().unwrap())
                .thumbnail(user.face())
                .title(&user.tag())
                .field("ID", user.id, true)
                .field("Mention", user.mention(), true)
                .field("Nickname", member.display_name().into_owned(), true)
                .field("Dates", format!("Created: {}\nJoined: {}", user.created_at(), member.joined_at.unwrap()), false)
                .field(format!("Roles [{}]", member.roles.len()), roles, false)
        )).expect("Failed to send message");
    };
});

command!(weather(ctx, message, args) {
});

// Rank 1

//TODO obtain data safely
command!(mod_info(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = db.get_user(user_id.0 as i64, guild_id.0 as i64).unwrap();
    let cases = db.get_cases(user_id.0 as i64, guild_id.0 as i64).unwrap();
    let case_fmt = cases.iter().map(|c| format!("Type: {}\nModerator: {}\nTimestamp: {}", c.casetype, c.moderator, c.timestamp)).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Moderator info")
            .field("Watchlist", { if user.watchlist { "Yes" } else { "No" } }, false)
            .field("Cases", case_fmt, false))).expect("Failed to send message");
});

command!(mute(ctx, message, args) {
});

command!(unmute(ctx, message, args) {
});

command!(note_add(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let note = String::from(args.rest());
    match db.new_note(user.0 as i64, message.guild_id.unwrap().0 as i64, note, message.author.id.0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Added note `{}`.", data.note)).expect("Failed to send message"); },
        Err(why) => { message.channel_id.say(format!("Failed to add note. Reason: {:?}", why)).expect("Failed to send message"); },
    }
});

command!(note_del(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let index = args.single::<i32>().unwrap_or(0);
    match db.del_note(index, user.0 as i64, message.guild_id.unwrap().0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Deleted note `{}`.", data)).expect("Failed to send message"); },
        Err(why) => { message.channel_id.say(format!("Failed to delete note. Reason: {:?}", why)).expect("Failed to send message"); },
    }
});

//TODO legible output
command!(note_list(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = user_id.get().unwrap();
    let notes = db.get_notes(user_id.0 as i64, message.guild_id.unwrap().0 as i64).unwrap();
    let notes_fmt = notes.iter().map(|n| format!("`{}` by {} at {}", n.note, n.moderator, n.timestamp)).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(Colours::Main.val())
            .title(format!("Notes for {}", user.tag()))
            .description(notes_fmt))).expect("Failed to send message");
});

command!(register(ctx, message, args) {
});

command!(ar(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let list = args.rest().split(",").map(|s| s.trim().to_string());
    let mut to_add = Vec::new();
    let mut failed = Vec::new();
    for r1 in list {
        if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
            to_add.push(s);
        } else {
            failed.push(format!("Could not locate {}", r1));
        }
    }
    for (i, role_id) in to_add.clone().iter().enumerate() {
        if member.roles.contains(role_id) {
            to_add.remove(i);
            failed.push(format!("You already have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.add_role(*role_id) {
            to_add.remove(i);
            failed.push(format!("Failed to add {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_add.is_empty() {
        fields.push(("Added Roles", format!("{}", to_add.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Add", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Add Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap()))).expect("Failed to send message");
});

command!(rr(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let list = args.rest().split(",").map(|s| s.trim().to_string());
    let mut to_remove = Vec::new();
    let mut failed = Vec::new();
    for r1 in list {
        if let Some((s,_)) = parse_role(r1.clone(), guild_id) {
            to_remove.push(s);
        } else {
            failed.push(format!("Could not locate {}", r1));
        }
    }
    for (i, role_id) in to_remove.clone().iter().enumerate() {
        if !member.roles.contains(role_id) {
            to_remove.remove(i);
            failed.push(format!("You don't have {}", role_id.find().unwrap().name));
        }
        if let Err(_) = member.remove_role(*role_id) {
            to_remove.remove(i);
            failed.push(format!("Failed to remove {}", role_id.find().unwrap().name));
        };
    }
    let mut fields = Vec::new();
    if !to_remove.is_empty() {
        fields.push(("Removed Roles", format!("{}", to_remove.iter().map(|r| r.find().unwrap().name).collect::<Vec<String>>().join("\n")), false));
    }
    if !failed.is_empty() {
        fields.push(("Failed to Remove", format!("{}", failed.join("\n")), false));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Remove Role Summary")
            .fields(fields)
            .colour(member.colour().unwrap()))).expect("Failed to send message");
});

//TODO make not shit
command!(role_colour(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (_, mut role) = parse_role(args.single::<String>().unwrap(), guild_id).unwrap();
    let colour_as_hex = args.single::<String>().unwrap();
    let colour = u64::from_str_radix(colour_as_hex.as_str(), 16).unwrap();
    if let Ok(_) = role.edit(|r| r.colour(colour)) {
        message.channel_id.say("Colour changed successfully.").expect("Failed to send message");
    }
});

command!(watchlist_add(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64).unwrap();
    user_data.watchlist = true;
    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
});

command!(watchlist_del(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64).unwrap();
    user_data.watchlist = true;
    db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data).expect("Failed to update user");
});

command!(watchlist_list(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let users = db.get_users(guild_id.0 as i64).unwrap_or(Vec::new());
    let user_map = users.iter().map(|u| UserId(u.id as u64).get().unwrap()).map(|u| u.tag()).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Watchlist")
            .description(user_map)
            .colour(Colours::Main.val()))).expect("Failed to send message");
});

// Rank 2

command!(config_raw(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    message.channel_id.say(format!("{:?}", guild_data)).expect("Failed to send message");
});

command!(config_list(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(Colours::Main.val())
            .description(format!("{}", guild_data))
    )).expect("Failed to send message");
});

command!(config_prefix(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let pre = args.single::<String>().unwrap();
    guild_data.prefix = pre;
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.say(format!("Set prefix to {}", guild.prefix)).expect("Failed to send message");
        },
        Err(_) => {
            message.channel_id.say("Failed to change prefix").expect("Failed to send message");
        },
    };
});

command!(config_autorole(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.autoroles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.autoroles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "enable" => {
            guild_data.autorole = true;
        },
        "disable" => {
            guild_data.autorole = false;
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Autorole Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.autorole) } else { val } ,
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_admin(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.admin_roles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.admin_roles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Admin Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_mod(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "add" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.mod_roles.push(role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        "remove" => {
            let (role_id, role) = parse_role(val.to_string(), guild_id).unwrap();
            guild_data.mod_roles.retain(|e| *e != role_id.0 as i64);
            val = format!("{} ({})", role.name, role_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Mod Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        val,
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_audit(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.audit = true;
        },
        "disable" => {
            guild_data.audit = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.audit_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "threshold" => {
            let th = val.parse::<i16>().unwrap();
            guild_data.audit_threshold = th;
            val = format!("{}", th);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Audit Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.audit) } else { val },
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_modlog(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.modlog = true;
        },
        "disable" => {
            guild_data.modlog = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.modlog_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Modlog Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.modlog) } else { val },
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_welcome(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.welcome = true;
        },
        "disable" => {
            guild_data.welcome = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.welcome_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "message" => {
            guild_data.welcome_message = val.to_string();
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Welcome Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.welcome) } else { val },
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

command!(config_introduction(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let op = args.single::<String>().unwrap();
    let mut val = args.rest().to_string();
    match op.to_lowercase().as_str() {
        "enable" => {
            guild_data.introduction = true;
        },
        "disable" => {
            guild_data.introduction = false;
        },
        "channel" => {
            let (channel_id, channel) = parse_channel(val.to_string(), guild_id).unwrap();
            guild_data.introduction_channel = channel_id.0 as i64;
            val = format!("{} ({})", channel.name, channel_id.0);
        },
        "message" => {
            guild_data.introduction_message = val.to_string();
        },
        _ => {},
    }
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title("Config Introduction Summary")
                    .colour(Colours::Main.val())
                    .description(format!("**Operation:** {}\n**Value:** {}",
                        op,
                        if val.is_empty() { format!("{}", guild.introduction) } else { val },
                    ))
            )).expect("Failed to send message");
        },
        Err(why) => {},
    }
});

// TODO add hackban and ignore lists views
command!(hackban(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let user_id = match UserId::from_str(args.full()) {
        Ok(id) => id,
        Err(_) => {
            message.channel_id.say("Unable to resolve ID").expect("Failed to send message");
            panic!("Failed to resolve ID");
        },
    };
    if !guild_data.hackbans.contains(&(user_id.0 as i64)) {
        guild_data.hackbans.push(user_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("Added {} to the hackban list",
                    user_id.0
                )).expect("Failed to send message");
            },
            Err(why) =>{
                message.channel_id.say("Failed to add hackban")
                    .expect("Failed to send message");
            },
        };
    } else {
        guild_data.hackbans.retain(|e| *e != user_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("Removed {} from the hackban list",
                    user_id.0
                )).expect("Failed to send message");
            },
            Err(why) =>{
                message.channel_id.say("Failed to remove hackban")
                    .expect("Failed to send message");
            },
        };
    }
});

command!(ignore(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let (channel_id, channel) = parse_channel(args.full().to_string(), guild_id).unwrap();
    if !guild_data.ignored_channels.contains(&(channel_id.0 as i64)) {
        guild_data.ignored_channels.push(channel_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("I will now ignore messages in {}",
                    channel.name
                )).expect("Failed to send message");
            },
            Err(why) =>{
                message.channel_id.say("Failed to add channel to ignore list")
                    .expect("Failed to send message");
            },
        };
    } else {
        guild_data.ignored_channels.retain(|e| *e != channel_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("I will no longer ignore messages in {}",
                    channel.name
                )).expect("Failed to send message");
            },
            Err(why) =>{
                message.channel_id.say("Failed to remove channel to ignore list")
                    .expect("Failed to send message");
            },
        };
    }
});

command!(csr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (role_id, role) = parse_role(args.single_quoted::<String>().unwrap(), guild_id).unwrap();
    let switches = get_switches(args.rest().to_string());
    let category = match switches.get("c") {
        Some(s) => Some( s.clone()),
        None => None,
    };
    let aliases = match switches.get("a") {
        Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
        None => None,
    };
    match db.new_role(role_id.0 as i64, guild_id.0 as i64, category, aliases) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully added role {} to category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            )).expect("Failed to send message");
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to add role: {:?}", why)).expect("Failed to send message");
        },
    };
});

command!(dsr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (role_id, role) = parse_role(args.single_quoted::<String>().unwrap(), guild_id).unwrap();
    match db.del_role(role_id.0 as i64, guild_id.0 as i64) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully deleted role {}", data)).expect("Failed to send message");
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to delete role: {:?}", why)).expect("Failed to send message");
        },
    };
});

command!(prune(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64).unwrap();
    let mut count = args.single::<usize>().unwrap();
    let fsel = args.single::<String>().unwrap_or(String::new());
    let mut filter = get_filter(fsel, guild_id);
    let mut deletions = message.channel_id.messages(|_| re_retriever(100)).unwrap();
    let mut next_deletions;
    let mut num_del = 0;
    message.delete().expect("Failed to delete message");
    if count<1000 {
        while count>0 {
            deletions.retain(|m| filter(m));
            let mut len = deletions.len();
            if len>count {
                deletions.truncate(count);
                len = count;
            }
            count -= len;
            if count>0 {
                next_deletions = match message.channel_id.messages(|_| be_retriever(deletions.first().unwrap().id, 100)) {
                    Ok(msgs) => Some(msgs),
                    Err(_) => None,
                }
            } else {
                next_deletions = None;
            }
            match message.channel_id.delete_messages(deletions) {
                Ok(_) => {
                    num_del += len;
                    deletions = match next_deletions {
                        Some(s) => s,
                        None => { break; },
                    }
                },
                Err(why) => {
                    error!("{:?}", why);
                    break;
                },
            }
        }
        if guild_data.modlog {
            let channel_lock = message.channel_id.get().unwrap().guild().unwrap();
            let channel = channel_lock.read();
            ChannelId(guild_data.modlog_channel as u64).send_message(|m| m
                .embed(|e| e
                    .title("Messages Pruned")
                    .description(format!("**Count:** {}\n**Moderator:** {} ({})\n**Channel:** {} ({})",
                        num_del,
                        message.author.mention(),
                        message.author.tag(),
                        channel.mention(),
                        channel.name))
                    .timestamp(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string())
            )).expect("Failed to send message");
        } else {
            message.channel_id.say(format!("Pruned {} message!", num_del)).expect("Failed to send message");
        }
    }
});

command!(test(ctx, message, args) {
});

// Rank 3

command!(setup_mute(ctx, message, _args) {
});

// Rank 4

command!(git(_ctx, message, args) {
});

command!(log(_ctx, message, _args) {
    use std::path::Path;
    message.channel_id.send_files(vec![Path::new("./log.txt")], |m| m).expect("Failed to send message");
});

command!(restart(_ctx, message, _args) {
});

// Helper functions for commands::prune
fn re_retriever(limit: u64) -> GetMessages {
    GetMessages::default()
        .limit(limit)
}

fn be_retriever(id: MessageId, limit: u64) -> GetMessages {
    GetMessages::default()
        .before(id)
        .limit(limit)
}

fn get_filter(input: String, guild_id: GuildId) -> Box<FnMut(&Message) -> bool> {
    match input.as_str() {
        "bot" => Box::new(|m| m.author.bot),
        "mention" => Box::new(|m| !m.mentions.is_empty() && m.mention_everyone),
        "attachment" => Box::new(|m| !m.attachments.is_empty()),
        "!pin" => Box::new(|m| !m.pinned),
        _ => {
            match parse_user(input.to_string(), guild_id) {
                Some((user_id, _)) => {
                    Box::new(move |m| m.author.id == user_id)
                },
                None => {
                    Box::new(|m| true)
                },
            }
        },
    }
}

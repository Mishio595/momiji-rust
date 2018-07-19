use std::collections::HashMap;
use std::str::FromStr;
use serenity::CACHE;
use serenity::prelude::*;
use serenity::model::Permissions;
use serenity::model::id::*;
use serenity::model::guild::Role;
use serenity::model::channel::{Message, PermissionOverwrite, PermissionOverwriteType};
use serenity::client::bridge::gateway::ShardId;
use serenity::builder::GetMessages;
use sysinfo;
use sysinfo::{ProcessExt, SystemExt};
use sys_info;
use rand::prelude::*;
use ::core::utils::*;
use ::core::model::*;
use ::core::consts::*;
use chrono::Utc;
use regex::Regex;

// Rank 0

command!(bot_info(ctx, message, _args) {
    let mut data = ctx.data.lock();
    let cache = CACHE.read();
    let shard_count = cache.shard_count;
    let owner = data.get::<Owner>().expect("Failed to get owner").get()?;
    let sys = sysinfo::System::new();
    let process = sys.get_process(sysinfo::get_current_pid()).unwrap();

    message.channel_id.send_message(|m| m
        .embed(|e| e
            .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
            .field("Owner", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
            .field("Links", "[Momiji's House](https://discord.gg/YYdpsNc)\n[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)\n[Github](https://github.com/Mishio595/momiji-rust)\n[Patreon](https://www.patreon.com/momijibot)", true)
            .field("Counts", format!("Guilds: {}\nShards: {}", cache.guilds.len(), shard_count), false)
            .field("System Info", format!("OS: {} {}\nUptime: {}",
                sys_info::os_type().unwrap(),
                sys_info::os_release().unwrap(),
                seconds_to_hrtime(sys.get_uptime() as usize)), true)
            .field("Process Info", format!("Memory Usage: {} mB\nCPU Usage {}%\nUptime: {}",
                process.memory()/1000, // convert to mB
                (process.cpu_usage()*100.0).round()/100.0, // round to 2 decimals
                seconds_to_hrtime((sys.get_uptime() - process.start_time()) as usize)), true)
            .thumbnail(&cache.user.avatar_url().unwrap_or(cache.user.default_avatar_url()))
            .colour(Colours::Main.val())
        ))?;
});

command!(cat(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<ApiClient>().expect("Failed to get API Client").cat() {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .image(res.file)))?;
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
                .image(res.message)))?;
    };
});

command!(dad_joke(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Ok(res) = data.get::<ApiClient>().expect("Failed to get API Client").joke() {
        message.channel_id.say(res)?;
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
                ))?;
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
    )?;
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
        ))?;
    };
});

command!(prefix(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let settings = db.get_guild(message.guild_id.unwrap().0 as i64)?;
    message.channel_id.say(format!("The prefix for this guild is `{}`", settings.prefix))?;
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
                    seconds_to_hrtime(dur as usize)))?;
            },
            Err(why) => {
                message.channel_id.say(format!("Sorry, I couldn't make the reminder. Here's why: {:?}", why))?;
            },
        }
    } else {
        message.channel_id.say("Sorry, I wasn't able to find a timer there. Make sure you to add `/t time_resolvable` after your reminder text.")?;
    }
});

command!(asr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let roles = db.get_roles(guild_id.0 as i64)?;
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
            .colour(member.colour().unwrap())))?;
});

command!(rsr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut member = message.member().unwrap();
    let roles = db.get_roles(guild_id.0 as i64)?;
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
            .colour(member.colour().unwrap())))?;
});

command!(lsr(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let roles = db.get_roles(guild_id.0 as i64)?;
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
        let mut des = val.iter().map(|c| RoleId(*c as u64).find().unwrap().name).collect::<Vec<String>>();
        des.sort();
        fields.push((key, des.join("\n"), true));
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Self Roles")
            .fields(fields)
            .colour(Colours::Main.val())
    ))?;
});

command!(role_info(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    if let Some((role_id, role)) = parse_role(args.rest().to_string(), guild_id) {
        let role_data = match db.get_role(role_id.0 as i64, guild_id.0 as i64) {
            Ok(r) => Some(r),
            _ => None,
        };
        let mut fields = vec![
            ("Name", role.name.clone(), true),
            ("ID", format!("{}", role_id.0), true),
            ("Hex", format!("#{}", role.colour.hex()), true),
            ("Hoisted", String::from(if role.hoist { "Yes" } else { "No" }), true),
            ("Mentionable", String::from(if role.mentionable { "Yes" } else { "No" }), true),
            ("Position", format!("{}", role.position), true),
        ];
        match role_data {
            Some(r) => {
                fields.push(("Self Assignable", String::from("Yes"), true));
                if !r.aliases.is_empty() {
                    fields.push(("Self Role Aliases", r.aliases.join(", "), true));
                }
            },
            None => {
                fields.push(("Self Assignable", String::from("No"), true));
            }
        }
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .thumbnail(format!("https://www.colorhexa.com/{}.png", role.colour.hex().to_lowercase()))
                .colour(role.colour)
                .fields(fields)
        ))?;
    };
});

command!(roll(_ctx, message, args) {
    let re = Regex::new(r"(?P<count>\d{0,2})d?(?P<sides>\d{0,2})").unwrap();
    let expr = args.single::<String>().unwrap_or(String::new());
    let caps = re.captures(expr.as_str()).unwrap();
    let count: u32 = caps["count"].parse().unwrap_or(1);
    let sides: u32 = caps["sides"].parse().unwrap_or(6);
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
        ))?;
    }
});

command!(server_info(_ctx, message, args) {
    use serenity::model::channel::ChannelType::*;
    use serenity::model::user::OnlineStatus::*;

    let switches = get_switches(args.full().to_string());
    let g = match switches.get("rest") {
        Some(s) => {
            let (id, lock) = parse_guild(s.to_string()).unwrap_or((message.guild_id.unwrap(), message.guild().unwrap()));
            Some(lock)
        },
        None => message.guild(),
    };
    if let Some(guild_lock) = g {
        let guild = guild_lock.read().clone();
        match switches.get("roles") {
            None => {
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
                        .thumbnail(guild.icon_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()))
                        .color(Colours::Main.val())
                        .field("ID", guild.id, true)
                        .field("Name", &guild.name, true)
                        .field("Owner", guild.owner_id.mention(), true)
                        .field("Region", guild.region, true)
                        .field(format!("Channels [{}]", guild.channels.len()), format!("Categories: {}\nText: {}\nVoice: {}", channels.2, channels.0, channels.1), true)
                        .field(format!("Members [{}/{}]", members.2, guild.members.len()), format!("Humans: {}\nBots: {}", members.0, members.1), true)
                        .field("Created", guild.id.created_at().format("%a, %d %h %Y @ %H:%M:%S").to_string(), false)
                        .field("Roles", guild.roles.len(), true)
                        .field("Emojis", guild.emojis.len(), true)
                        .title(guild.name)
                ))?;
            },
            Some(_) => {
                let mut roles_raw = guild.roles.values().collect::<Vec<&Role>>();
                roles_raw.sort_by(|a, b| b.position.cmp(&a.position));
                let roles = roles_raw.iter().map(|e| e.name.clone()).collect::<Vec<String>>();
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .title(format!("Roles for {}. Count: {}", guild.name, roles.len()))
                        .description(roles.join("\n"))
                        .colour(Colours::Blue.val())
                ))?;
            },
        }
    }
});

// TODO add fuzzy matching
command!(tag_single(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.rest().to_string();
    match db.get_tag(guild_id.0 as i64, tag_input.clone()) {
        Ok(tag) => { message.channel_id.say(tag.data)?; },
        Err(why) => { message.channel_id.say("No tag by that name")?; },
    }
});

command!(tag_add(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    let value = args.rest().to_string();
    match db.new_tag(message.author.id.0 as i64, guild_id.0 as i64, tag_input.clone(), value) {
        Ok(tag) => { message.channel_id.say(format!("Successfully created tag `{}`", tag.name))?; },
        Err(why) => { message.channel_id.say(format!("Failed to create tag `{}`. Here's why: {:?}", tag_input, why))?; },
    }
});

// TODO add mod/admin checks and ownership check
command!(tag_del(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    match db.del_tag(guild_id.0 as i64, tag_input.clone()) {
        Ok(tag) => { message.channel_id.say(format!("Successfully deleted tag `{}`", tag.name))?; },
        Err(why) => { message.channel_id.say(format!("Failed to create tag `{}`. Here's why: {:?}", tag_input, why))?; },
    }
});

// TODO add mod/admin check
command!(tag_edit(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    let value = args.rest().to_string();
    let mut tag = db.get_tag(guild_id.0 as i64, tag_input.clone())?;
    if message.author.id.0 as i64 == tag.author {
        tag.data = value.clone();
        match db.update_tag(guild_id.0 as i64, tag_input.clone(), tag) {
            Ok(t) => { message.channel_id.say(format!("Successfully edited tag `{}`", t.name))?; },
            Err(why) => { message.channel_id.say(format!("Failed to edit tag `{}`. Here's why: {:?}", tag_input, why))?; },
        }
    } else {
        message.channel_id.say("You do not own this tag and do not have permissions to edit it")?;
    }
});

command!(urban(ctx, message, args) {
    let data = ctx.data.lock();
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
                        .field("Tags", res.tags.iter().map(|t| { String::from("#")+t }).collect::<Vec<String>>().join(", "), false)
                ))?;
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
                ))?;
            }
        }
    };
});

command!(user_info(ctx, message, args) {
    if let Some(guild_lock) = message.guild() {
        let data = ctx.data.lock();
        let db = data.get::<DB>().unwrap().lock();
        let guild = guild_lock.read();
        let guild_data = db.get_guild(guild.id.0 as i64)?;
        let (user, member) = match parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
            Some((id, member)) => (id.get().unwrap(), member),
            None => (message.author.clone(), message.member().unwrap().clone()),
        };
        let user_data = db.get_user(user.id.0 as i64, guild.id.0 as i64)?;
        let mut roles = member.roles.iter().map(|c| c.find().unwrap().name).collect::<Vec<String>>();
        roles.sort();
        let dates = format!("Created: {}\nJoined: {}{}",
            user.created_at().format("%a, %d %h %Y @ %H:%M:%S").to_string(),
            member.joined_at.unwrap().format("%a, %d %h %Y @ %H:%M:%S").to_string(),
            if guild_data.premium {
                let r = user_data.registered;
                if r.is_some() { format!("\nRegistered: {}", r.unwrap().format("%a, %d %h %Y @ %H:%M:%S").to_string()) }
                else { String::new() }}
            else { String::new() }
        );
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .colour(member.colour().unwrap())
                .thumbnail(user.face())
                .title(&user.tag())
                .field("ID", user.id, true)
                .field("Mention", user.mention(), true)
                .field("Nickname", member.display_name().into_owned(), true)
                .field("Dates", dates, false)
                .field(format!("Roles [{}]", member.roles.len()), roles.join(", "), false)
        ))?;
    };
});

command!(weather(ctx, message, args) {
});

command!(xp(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let user_data = db.get_user(message.author.id.0 as i64, guild_id.0 as i64)?;
    message.channel_id.say(format!("Your current XP is {}", user_data.xp))?;
});

// Rank 1

//TODO obtain data safely
command!(mod_info(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    let cases = db.get_cases(user_id.0 as i64, guild_id.0 as i64)?;
    let case_fmt = cases.iter().map(|c| format!("Type: {}\nModerator: {}\nTimestamp: {}", c.casetype, c.moderator, c.timestamp)).collect::<Vec<String>>().join("\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Moderator info")
            .field("Watchlist", { if user.watchlist { "Yes" } else { "No" } }, false)
            .field("Cases", case_fmt, false)
    ))?;
});

command!(mute(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let (_, mut member) = parse_user(args.single::<String>().unwrap(), guild.id).unwrap();
    let temp = member.clone();
    let user = temp.user.read();
    let guild_data = db.get_guild(guild.id.0 as i64)?;
    if guild_data.mute_setup {
        let switches = get_switches(args.rest().to_string());
        let time = match switches.get("t") {
            Some(s) => hrtime_to_seconds(s.clone()),
            None => 0,
        };
        let reason = match switches.get("r") {
            Some(s) => s.clone(),
            None => String::new(),
        };
        let mute_role = guild.roles.values().find(|e| e.name.to_lowercase() == "muted").unwrap();
        if member.roles.contains(&mute_role.id) {
            message.channel_id.say("Member already muted.")?;
        } else {
            if let Ok(_) = member.add_role(mute_role) {
                let case = db.new_case(user.id.0 as i64, guild.id.0 as i64, "Mute".to_string(), message.author.id.0 as i64)?;
                let mut fields = Vec::new();
                fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                if time != 0 {
                    let tc = data.get::<TC>().unwrap().lock();
                    tc.request(format!("UNMUTE||{}||{}||{}||{}||{}||{}",
                        user.id.0,
                        guild.id.0,
                        mute_role.id.0,
                        if guild_data.modlog && guild_data.modlog_channel > 0 {
                            guild_data.modlog_channel
                        } else { message.channel_id.0 as i64 },
                        time,
                        case.id), time as u64);
                    fields.push(("Duration", seconds_to_hrtime(time as usize), true));
                }
                if !reason.is_empty() {
                    fields.push(("Reason", reason.to_string(), true));
                }
                if guild_data.modlog && guild_data.modlog_channel > 0 {
                    let channel = ChannelId(guild_data.modlog_channel as u64);
                    channel.send_message(|m| m
                        .embed(|e| e
                            .title("Member Muted")
                            .colour(Colours::Blue.val())
                            .fields(fields)
                    ))?;
                } else {
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Member Muted")
                            .colour(Colours::Blue.val())
                            .fields(fields)
                    ))?;
                }
            }
        }
    } else {
        message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
    }
});

command!(unmute(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let (_, mut member) = parse_user(args.single::<String>().unwrap(), guild.id).unwrap();
    let temp = member.clone();
    let user = temp.user.read();
    let guild_data = db.get_guild(guild.id.0 as i64)?;
    if guild_data.mute_setup {
        let mute_role = guild.roles.values().find(|e| e.name.to_lowercase() == "muted").unwrap();
        let mut fields = Vec::new();
        fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
        fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
        if member.roles.contains(&mute_role.id) {
            if let Ok(_) = member.remove_role(mute_role) {
                if guild_data.modlog && guild_data.modlog_channel > 0 {
                    let channel = ChannelId(guild_data.modlog_channel as u64);
                    channel.send_message(|m| m
                        .embed(|e| e
                            .title("Member Unmuted")
                            .colour(Colours::Blue.val())
                            .fields(fields)
                    ))?;
                } else {
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Member Unmuted")
                            .colour(Colours::Blue.val())
                            .fields(fields)
                    ))?;
                }
            }
        } else {
            message.channel_id.say("Member was not muted.")?;
        }
    } else {
        message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
    }
});

command!(note_add(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let note = String::from(args.rest());
    match db.new_note(user.0 as i64, message.guild_id.unwrap().0 as i64, note, message.author.id.0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Added note `{}`.", data.note))?; },
        Err(why) => { message.channel_id.say(format!("Failed to add note. Reason: {:?}", why))?; },
    }
});

command!(note_del(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let index = args.single::<i32>().unwrap_or(0);
    match db.del_note(index, user.0 as i64, message.guild_id.unwrap().0 as i64) {
        Ok(data) => { message.channel_id.say(format!("Deleted note `{}`.", data))?; },
        Err(why) => { message.channel_id.say(format!("Failed to delete note. Reason: {:?}", why))?; },
    }
});

command!(note_list(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let user = user_id.get().unwrap();
    let notes = db.get_notes(user_id.0 as i64, message.guild_id.unwrap().0 as i64)?;
    let notes_fmt = notes.iter().map(|n| format!("{}", n)).collect::<Vec<String>>().join("\n\n");
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(Colours::Main.val())
            .title(format!("Notes for {}", user.tag()))
            .description(notes_fmt)
    ))?;
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
            .colour(member.colour().unwrap())
    ))?;
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
            .colour(member.colour().unwrap())
    ))?;
});

//TODO make not shit
command!(role_colour(_ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let (_, mut role) = parse_role(args.single::<String>().unwrap(), guild_id).unwrap();
    let colour_as_hex = args.single::<String>().unwrap();
    let colour = u64::from_str_radix(colour_as_hex.as_str(), 16).unwrap();
    if let Ok(_) = role.edit(|r| r.colour(colour)) {
        message.channel_id.say("Colour changed successfully.")?;
    }
});

command!(watchlist_add(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id, _) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    user_data.watchlist = true;
    match db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data) {
        Ok(_) => { message.channel_id.say(format!("Set {} to watchlist status.", user_id.get().unwrap().tag()))?; },
        Err(_) => { message.channel_id.say("Failed to set watchlist status")?; },
    }
});

command!(watchlist_del(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let (user_id,_) = parse_user(args.single::<String>().unwrap(), guild_id).unwrap();
    let mut user_data = db.get_user(user_id.0 as i64, guild_id.0 as i64)?;
    user_data.watchlist = true;
    match db.update_user(user_id.0 as i64, guild_id.0 as i64, user_data) {
        Ok(_) => { message.channel_id.say(format!("Unset {} from watchlist status.", user_id.get().unwrap().tag()))?; },
        Err(_) => { message.channel_id.say("Failed to unset watchlist status")?; },
    }
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
            .colour(Colours::Main.val())
    ))?;
});

// Rank 2

command!(config_raw(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
    message.channel_id.say(format!("{:?}", guild_data))?;
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
    ))?;
});

command!(config_prefix(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let pre = args.single::<String>().unwrap();
    guild_data.prefix = pre;
    match db.update_guild(guild_id.0 as i64, guild_data) {
        Ok(guild) => {
            message.channel_id.say(format!("Set prefix to {}", guild.prefix))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to change prefix")?;
        },
    };
});

command!(config_autorole(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_admin(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_mod(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_audit(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_modlog(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_welcome(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
        "type" => {
            guild_data.welcome_type = val.to_string();
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

command!(config_introduction(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        },
        Err(_) => {
            message.channel_id.say("Failed to update database")?;
        },
    }
});

// TODO add hackban and ignore lists views
command!(hackban(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
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
                ))?;
            },
            Err(why) =>{
                message.channel_id.say("Failed to add hackban")?;
            },
        };
    } else {
        guild_data.hackbans.retain(|e| *e != user_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("Removed {} from the hackban list",
                    user_id.0
                ))?;
            },
            Err(why) =>{
                message.channel_id.say("Failed to remove hackban")?;
            },
        };
    }
});

// TODO rewrite as group {add, remove, list}
command!(ignore(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let (channel_id, channel) = parse_channel(args.full().to_string(), guild_id).unwrap();
    if !guild_data.ignored_channels.contains(&(channel_id.0 as i64)) {
        guild_data.ignored_channels.push(channel_id.0 as i64);
        match db.update_guild(guild_id.0 as i64, guild_data) {
            Ok(guild) => {
                message.channel_id.say(format!("I will now ignore messages in {}",
                    channel.name
                ))?;
            },
            Err(why) =>{
                message.channel_id.say("Failed to add channel to ignore list")?;
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
    let switches = get_switches(args.full().to_string());
    let rest = switches.get("rest").unwrap();
    let (role_id, role) = parse_role(rest.clone(), guild_id).expect("Failed to parse role");
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
            ))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to add role: {:?}", why))?;
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
            message.channel_id.say(format!("Successfully deleted role {}", data))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to delete role: {:?}", why))?;
        },
    };
});

command!(esr(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let switches = get_switches(args.full().to_string());
    let rest = switches.get("rest").unwrap();
    let (role_id, role) = parse_role(rest.clone(), guild_id).expect("Failed to parse role");
    let category = match switches.get("c") {
        Some(s) => Some(s.clone()),
        None => None,
    };
    let aliases = match switches.get("a") {
        Some(s) => Some(s.split(",").map(|c| c.trim().to_string().to_lowercase()).collect::<Vec<String>>()),
        None => None,
    };
    let mut role = db.get_role(role_id.0 as i64, guild_id.0 as i64)?;
    if let Some(s) = category { role.category = s; }
    if let Some(mut a) = aliases {
        match switches.get("replace") {
            Some(_) => { role.aliases = a; },
            None => { role.aliases.append(&mut a); },
        }
    }
    match db.update_role(role_id.0 as i64, guild_id.0 as i64, role) {
        Ok(data) => {
            message.channel_id.say(format!("Successfully update role {} in category {} {}",
                data.id,
                data.category,
                if !data.aliases.is_empty() {
                    format!("with aliases {}", data.aliases.join(","))
                } else {
                    String::new()
                }
            ))?;
        },
        Err(why) => {
            message.channel_id.say(format!("Failed to edit role: {:?}", why))?;
        },
    };
});

command!(prune(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().expect("Failed to get DB").lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
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
            ))?;
        } else {
            message.channel_id.say(format!("Pruned {} message!", num_del))?;
        }
    }
});

command!(test_welcome(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let guild_data = db.get_guild(guild_id.0 as i64)?;
    let member = message.member().unwrap();
    if guild_data.welcome {
        let channel = ChannelId(guild_data.welcome_channel as u64);
        if guild_data.welcome_type.as_str() == "embed" {
            send_welcome_embed(guild_data.welcome_message, &member, channel)?;
        } else {
            channel.say(parse_welcome_items(guild_data.welcome_message, &member))?;
        }
    }
});

command!(setup_mute(ctx, message, _args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let guild_id = message.guild_id.unwrap();
    let lock = message.guild().unwrap();
    let guild = lock.read();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    let mute_role = match guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
        Some(role) => role.clone(),
        None => {
            message.channel_id.say("Role `Muted` created")?;
            guild.create_role(|r| r.name("Muted")).unwrap()
        },
    };
    let allow = Permissions::empty();
    let deny = Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS | Permissions::SPEAK;
    let overwrite = PermissionOverwrite {
        allow,
        deny,
        kind: PermissionOverwriteType::Role(mute_role.id),
    };
    for channel in guild.channels.values() {
        let mut channel = channel.read();
        channel.create_permission(&overwrite)?;
    }
    guild_data.mute_setup = true;
    db.update_guild(guild.id.0 as i64, guild_data)?;
    message.channel_id.say(format!("Setup permissions for {} channels.", guild.channels.len()))?;
});

// Rank 4

command!(git(_ctx, message, args) {
});

command!(log(_ctx, message, _args) {
    use std::path::Path;
    message.channel_id.send_files(vec![Path::new("./log.txt")], |m| m)?;
});

command!(premium(ctx, message, args) {
    let data = ctx.data.lock();
    let db = data.get::<DB>().unwrap().lock();
    let g = args.single::<String>().unwrap();
    let op = args.single::<String>().unwrap();
    let (guild_id, guild) = parse_guild(g).unwrap();
    let mut guild_data = db.get_guild(guild_id.0 as i64)?;
    match op.to_lowercase().as_str() {
        "enable" => { guild_data.premium = true; },
        "disable" => { guild_data.premium = false; },
        "set" => { guild_data.premium_tier = args.single::<i16>().unwrap(); },
        _ => {},
    }
    db.update_guild(guild_id.0 as i64, guild_data)?;
    message.channel_id.say("Command complete")?;
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
                    Box::new(|_| true)
                },
            }
        },
    }
}

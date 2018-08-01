use core::utils::*;
use core::model::*;
use core::consts::DB as db;
use core::consts::*;
use core::colours;
use serenity::CACHE;
use serenity::prelude::*;
use serenity::model::id::*;
use serenity::model::guild::Role;
use serenity::client::bridge::gateway::ShardId;
use sysinfo;
use sysinfo::{ProcessExt, SystemExt};
use sys_info;
use rand::prelude::*;
use chrono::Utc;
use regex::Regex;
use fuzzy_match::fuzzy_match;
use fuzzy_match::algorithms::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use forecast::Icon::*;

lazy_static! {
    static ref DICE_MATCH: Regex = Regex::new(r"(?P<count>\d+)d?(?P<sides>\d*)").expect("Failed to create Regex");
}

// Rank 0

command!(bot_info(ctx, message, _args) {
    let mut data = ctx.data.lock();
    let cache = CACHE.read();
    let shard_count = cache.shard_count;
    let owner = data.get::<Owner>().expect("Failed to get owner").get()?;
    let sys = sysinfo::System::new();
    if let Some(process) = sys.get_process(sysinfo::get_current_pid()) {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
                .field("Owner", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Momiji's House](https://discord.gg/YYdpsNc)\n[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)\n[Github](https://github.com/Mishio595/momiji-rust)\n[Patreon](https://www.patreon.com/momijibot)", true)
                .field("Counts", format!("Guilds: {}\nShards: {}", cache.guilds.len(), shard_count), false)
                .field("System Info", format!("OS: {} {}\nUptime: {}",
                    sys_info::os_type().unwrap_or(String::from("OS Not Found")),
                    sys_info::os_release().unwrap_or(String::from("Release Not Found")),
                    seconds_to_hrtime(sys.get_uptime() as usize)), true)
                .field("Process Info", format!("Memory Usage: {} mB\nCPU Usage {}%\nUptime: {}",
                    process.memory()/1000, // convert to mB
                    (process.cpu_usage()*100.0).round()/100.0, // round to 2 decimals
                    seconds_to_hrtime((sys.get_uptime() - process.start_time()) as usize)), true)
                .thumbnail(&cache.user.avatar_url().unwrap_or(cache.user.default_avatar_url()))
                .colour(*colours::MAIN)
        ))?;
    } else {
        message.channel_id.send_message(|m| m
            .embed(|e| e
                .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
                .field("Owner", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Momiji's House](https://discord.gg/YYdpsNc)\n[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)\n[Github](https://github.com/Mishio595/momiji-rust)\n[Patreon](https://www.patreon.com/momijibot)", true)
                .field("Counts", format!("Guilds: {}\nShards: {}", cache.guilds.len(), shard_count), false)
                .thumbnail(&cache.user.avatar_url().unwrap_or(cache.user.default_avatar_url()))
                .colour(*colours::MAIN)
        ))?;
    }
});

command!(cat(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Some(api) = data.get::<ApiClient>() {
        match api.cat() {
            Ok(res) => {
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .image(res.file)
                ))?;
            },
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
});

/* TODO add these in once I get good tools for it
command!(color(ctx, message, args) {
});

command!(danbooru(ctx, message, args) {
});
*/

command!(dog(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Some(api) = data.get::<ApiClient>() {
        match api.dog() {
            Ok(res) => {
                message.channel_id.send_message(|m| m
                    .embed(|e| e
                        .image(res.message)
                ))?;
            },
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
});

command!(dad_joke(ctx, message, _args) {
    let mut data = ctx.data.lock();
    if let Some(api) = data.get::<ApiClient>() {
        match api.joke() {
            Ok(res) => {
                message.channel_id.say(res)?;
            },
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
});

command!(e621(ctx, message, args) {
    let mut data = ctx.data.lock();
    message.channel_id.broadcast_typing()?;
    if let Some(api) = data.get::<ApiClient>() {
        match api.furry(args.full(), 1) {
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
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
});

command!(anime_search(ctx, message, args) {
    use kitsu::model::Status::*;
    let data = ctx.data.lock();
    message.channel_id.broadcast_typing()?;
    if let Some(api) = data.get::<ApiClient>() {
        match  api.anime(args.full()) {
            Ok(res) => {
                if let Some(anime) = res.data.first() {
                    let status = match anime.attributes.status {
                        Some(stat) => { match stat {
                            Current => "Current",
                            Finished => "Complete",
                            TBA => "To Be Announced",
                            Unreleased => "Unreleased",
                            Upcoming => "Upcoming",
                        }},
                        None => "Status Not Found",
                    };
                    let cover_url = match anime.attributes.cover_image.clone() {
                        Some(cover) => { match cover.original {
                            Some(url) => url,
                            None => String::new(),
                        }},
                        None => String::new(),
                    };
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title(format!("**{}**", anime.attributes.canonical_title.clone()))
                            .url(anime.url())
                            .description(format!("{}\n\n{}\n**Score:** {}\n**Status:** {}",
                                anime.attributes.synopsis,
                                if let Some(count) = anime.attributes.episode_count {
                                    let mut out = format!("**Episodes:** {}", count);
                                    if let Some(length) = anime.attributes.episode_length {
                                        out.push_str(format!(" ({} min/ep)", length).as_str());
                                    }
                                    out
                                } else { String::from("Episode Information Not Found") },
                                anime.attributes.average_rating.clone().unwrap_or(String::from("Not Found")),
                                status
                            ))
                            .thumbnail(cover_url)
                            .colour(*colours::MAIN)
                    ))?;
                }
            },
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
});

command!(manga_search(ctx, message, args) {
    use kitsu::model::Status::*;
    let data = ctx.data.lock();
    message.channel_id.broadcast_typing()?;
    if let Some(api) = data.get::<ApiClient>() {
        match api.manga(args.full()) {
            Ok(res) => {
                if let Some(manga) = res.data.first() {
                    let status = match manga.attributes.status {
                        Some(stat) => { match stat {
                            Current => "Current",
                            Finished => "Complete",
                            TBA => "To Be Announced",
                            Unreleased => "Unreleased",
                            Upcoming => "Upcoming",
                        }},
                        None => "Status Not Found",
                    };
                    let cover_url = match manga.attributes.cover_image.clone() {
                        Some(cover) => { match cover.original {
                            Some(url) => url,
                            None => String::new(),
                        }},
                        None => String::new(),
                    };
                    message.channel_id.send_message(|m| m
                        .embed(|e| e
                            .title(format!("**{}**", manga.attributes.canonical_title.clone()))
                            .url(manga.url())
                            .description(format!("{}\n\n**Volumes:** {}\n**Chapters:** {}\n**Score:** {}\n**Status:** {}",
                                manga.attributes.synopsis,
                                manga.attributes.volume_count.map_or(String::from("Not Found"), |count| format!("{}", count)),
                                manga.attributes.chapter_count.map_or(String::from("Not Found"), |count| format!("{}", count)),
                                manga.attributes.average_rating.clone().unwrap_or(String::from("Not Found")),
                                status
                            ))
                            .thumbnail(cover_url)
                            .colour(*colours::MAIN)
                    ))?;
                }
            },
            Err(why) => {
                failed!(API_GET_FAIL, why);
                message.channel_id.say(format!("Something went wrong while fetching the data.\nError: {}", why))?;
            },
        }
    } else { failed!(API_FAIL); }
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
            .colour(*colours::MAIN)
            .description(format!("**Time:** {}\n**Date:** {}\n**Timezone:** UTC{}", time, date, datetime.timezone()))
    ))?;
});

command!(ping(ctx, message, _args) {
    let data = ctx.data.lock();
    let mut lat = 0;
    if let Some(sm_lock) = data.get::<SerenityShardManager>() {
        let sm = sm_lock.lock();
        let runners = sm.runners.lock();
        if let Some(shard_runner) = runners.get(&ShardId(ctx.shard_id)) {
            if let Some(la) = shard_runner.latency {
                lat = la.as_secs() as u32 + la.subsec_millis();
            }
        }
    }
    if let Ok(mut m) = message.channel_id.send_message(|m| m.embed(|e| e.title("Pong!"))) {
        let t = m.timestamp.timestamp_millis() - message.timestamp.timestamp_millis();
        m.edit(|m| m.embed(|e| e
            .title("Pong!")
            .description(format!("**Shard Latency:** {}\n**Response Time:** {} ms", if lat==0 { String::from("Failed to retrieve") } else { format!("{} ms", lat) }, t))
            .colour(*colours::MAIN)
        ))?;
    };
});

command!(prefix(ctx, message, _args) {
    let data = ctx.data.lock();
    if let Some(guild_id) = message.guild_id {
        if let Ok(settings) = db.get_guild(guild_id.0 as i64) {
            message.channel_id.say(format!("The prefix for this guild is `{}`", settings.prefix))?;
        } else {
            message.channel_id.say("Failed to get guild data.")?;
        }
    }
});

command!(remind(ctx, message, args) {
    let data = ctx.data.lock();
    if let Some(tc_lock) = data.get::<TC>() {
        let tc = tc_lock.lock();
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
                        seconds_to_hrtime(dur as usize)
                    ))?;
                },
                Err(why) => {
                    message.channel_id.say(format!("Sorry, I couldn't make the reminder. Here's why: {:?}", why))?;
                },
            }
        } else {
            message.channel_id.say("Sorry, I wasn't able to find a timer there. Make sure you to add `/t time_resolvable` after your reminder text.")?;
        }
    } else { failed!(TC_FAIL); }
});

command!(asr(ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        if let Some(mut member) = message.member() {
            match db.get_roles(guild_id.0 as i64) {
                Ok(roles) => {
                    if !roles.is_empty() {
                        let list = args.rest().split(",").map(|s| s.trim().to_string());
                        let mut to_add = Vec::new();
                        let mut failed = Vec::new();
                        let role_names = roles.iter().filter_map(|r| match RoleId(r.id as u64).find() {
                            Some(role) => Some(role.clone()),
                            None => None,
                        }).collect::<Vec<Role>>();
                        for r1 in list {
                            if let Some((r, r2)) = parse_role(r1.clone(), guild_id) {
                                if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                    to_add.push(r);
                                } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                            } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
                                to_add.push(RoleId(roles[i].id as u64));
                            } else {
                                failed.push(format!("Failed to find match \"{}\". {}", r1,
                                    if let Some(i) = fuzzy_match(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                        let ref val = role_names[i];
                                        format!("Closest match: {}", val.name.clone())
                                    } else { String::new() }
                                ));
                            }
                        }
                        for (i, role_id) in to_add.clone().iter().enumerate() {
                            if member.roles.contains(role_id) {
                                to_add.remove(i);
                                failed.push(format!("You already have {}", match role_names.iter().find(|r| &r.id == role_id) {
                                    Some(s) => s.name.clone(),
                                    None => format!("{}", role_id.0),
                                }));
                            }
                            if let Err(_) = member.add_role(*role_id) {
                                to_add.remove(i);
                                failed.push(format!("Failed to add {}", match role_names.iter().find(|r| &r.id == role_id) {
                                    Some(s) => s.name.clone(),
                                    None => format!("{}", role_id.0),
                                }));
                            };
                        }
                        let mut fields = Vec::new();
                        if !to_add.is_empty() {
                            fields.push(("Added Roles", format!("{}", to_add.iter().filter_map(|r| match r.find() {
                                Some(r) => Some(r.name.clone()),
                                None => None,
                            }).collect::<Vec<String>>().join("\n")), false));
                        }
                        if !failed.is_empty() {
                            fields.push(("Failed to Add", format!("{}", failed.join("\n")), false));
                        }
                        message.channel_id.send_message(|m| m
                            .embed(|e| e
                                .title("Add Self Role Summary")
                                .fields(fields)
                                .colour(member.colour().unwrap_or(*colours::GREEN))
                        ))?;
                    } else {
                        message.channel_id.say("There are no self roles.")?;
                    }
                },
                Err(why) => { failed!(DB_ROLES_FAIL, why); } // This branch should never occur
            }
        } else { failed!(MEMBER_FAIL); }
    } else { failed!(GUILDID_FAIL); }
});

command!(rsr(ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        if let Some(mut member) = message.member() {
            match db.get_roles(guild_id.0 as i64) {
                Ok(roles) => {
                    if !roles.is_empty() {
                        let list = args.rest().split(",").map(|s| s.trim().to_string());
                        let mut to_remove = Vec::new();
                        let mut failed = Vec::new();
                        let role_names = roles.iter().filter_map(|r| match RoleId(r.id as u64).find() {
                            Some(role) => Some(role.clone()),
                            None => None,
                        }).collect::<Vec<Role>>();
                        for r1 in list {
                            if let Some((r, r2)) = parse_role(r1.clone(), guild_id) {
                                if let Some(_) = roles.iter().find(|e| e.id == r.0 as i64) {
                                    to_remove.push(r);
                                } else { failed.push(format!("{} is a role, but it isn't self-assignable", r2.name)); }
                            } else if let Some(i) = roles.iter().position(|r| r.aliases.contains(&r1)) {
                                to_remove.push(RoleId(roles[i].id as u64));
                            } else {
                                failed.push(format!("Failed to find match \"{}\". {}", r1,
                                    if let Some(i) = fuzzy_match(&r1, role_names.iter().enumerate().map(|(i,r)| (r.name.as_str(), i)).collect()) {
                                        let ref val = role_names[i];
                                        format!("Closest match: {}", val.name.clone())
                                    } else { String::new() }
                                ));
                            }
                        }
                        for (i, role_id) in to_remove.clone().iter().enumerate() {
                            if !member.roles.contains(role_id) {
                                to_remove.remove(i);
                                failed.push(format!("You already have {}", match role_names.iter().find(|r| &r.id == role_id) {
                                    Some(s) => s.name.clone(),
                                    None => format!("{}", role_id.0),
                                }));
                            }
                            if let Err(_) = member.remove_role(*role_id) {
                                to_remove.remove(i);
                                failed.push(format!("Failed to remove {}", match role_names.iter().find(|r| &r.id == role_id) {
                                    Some(s) => s.name.clone(),
                                    None => format!("{}", role_id.0),
                                }));
                            };
                        }
                        let mut fields = Vec::new();
                        if !to_remove.is_empty() {
                            fields.push(("Added Roles", format!("{}", to_remove.iter().filter_map(|r| match r.find() {
                                Some(r) => Some(r.name.clone()),
                                None => None,
                            }).collect::<Vec<String>>().join("\n")), false));
                        }
                        if !failed.is_empty() {
                            fields.push(("Failed to Remove", format!("{}", failed.join("\n")), false));
                        }
                        message.channel_id.send_message(|m| m
                            .embed(|e| e
                                .title("Remove Self Role Summary")
                                .fields(fields)
                                .colour(member.colour().unwrap_or(*colours::RED))
                        ))?;
                    } else {
                        message.channel_id.say("There are no self roles.")?;
                    }
                },
                Err(why) => { failed!(DB_ROLES_FAIL, why); } // This branch should never occur
            }
        } else { failed!(MEMBER_FAIL); }
    } else { failed!(GUILDID_FAIL); }
});

// TODO view a single category
command!(lsr(ctx, message, args) {
    if let Some(guild_id) = message.guild_id {
        match db.get_roles(guild_id.0 as i64) {
            Ok(roles) => {
                if !roles.is_empty() {
                    if args.is_empty() {
                        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
                        for role in roles.iter() {
                            match RoleId(role.id as u64).find() {
                                Some(r) => {
                                    map.entry(role.category.clone()).or_insert(Vec::new()).push(r.name);
                                },
                                None => {
                                    // Clean up roles that don't exist
                                    db.del_role(role.id, guild_id.0 as i64)?;
                                },
                            }
                        }
                        let mut fields = Vec::new();
                        for (key, val) in map.iter_mut() {
                            val.sort();
                            fields.push((key, val.join("\n"), true));
                        }
                        message.channel_id.send_message(|m| m
                            .embed(|e| e
                                .title("Self Roles")
                                .fields(fields)
                                .colour(*colours::MAIN)
                        ))?;
                    } else {
                        // single cat
                    }
                } else {
                    message.channel_id.say("There are no self roles.")?;
                }
            },
            Err(why) => { failed!(DB_ROLES_FAIL, why); } // This branch should never happen
        }
    } else { failed!(GUILDID_FAIL); }
});

command!(role_info(ctx, message, args) {
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

// TODO eval expressions such as "2d10 + 5"
command!(roll(_ctx, message, args) {
    let expr = args.single::<String>().unwrap_or(String::new());
    let caps = DICE_MATCH.captures(expr.as_str()).unwrap();
    let count: u32 = caps["count"].parse().unwrap_or(1);
    let sides: u32 = caps["sides"].parse().unwrap_or(6);
    if count > 0 && count <= 1000 {
        if sides > 0 && sides <= 100 {
            let mut total = 0;
            for _ in 1..&count+1 {
                let r = thread_rng().gen_range(1,&sides+1);
                total += r;
            }
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .colour(*colours::MAIN)
                    .field(format!("{} 🎲 [1-{}]", count, sides), format!("You rolled {}", total), true)
            ))?;
        } else { message.channel_id.say("Sides out of bounds")?; }
    } else { message.channel_id.say("Count out of bounds")?; }
});

command!(server_info(_ctx, message, args) {
    use serenity::model::channel::ChannelType::*;
    use serenity::model::user::OnlineStatus::*;

    let switches = get_switches(args.full().to_string());
    let g = match switches.get("rest") {
        Some(s) => {
            let (_, lock) = parse_guild(s.to_string()).unwrap_or((message.guild_id.unwrap(), message.guild().unwrap()));
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
                        .color(*colours::MAIN)
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
                        .colour(*colours::BLUE)
                ))?;
            },
        }
    }
});

command!(tag_list(ctx, message, _args) {
    if let Some(guild_id) = message.guild_id {
        match db.get_tags(guild_id.0 as i64) {
            Ok(tags) => {
                message.channel_id.say(tags.iter().map(|e| e.name.as_str()).collect::<Vec<&str>>().join("\n"))?;
            },
            Err(why) => {},
        }
    }
});

command!(tag_single(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.full().trim().to_string();
    let tags = db.get_tags(guild_id.0 as i64)?;
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
});

command!(tag_add(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    let value = args.rest().to_string();
    match db.new_tag(message.author.id.0 as i64, guild_id.0 as i64, tag_input.clone(), value) {
        Ok(tag) => { message.channel_id.say(format!("Successfully created tag `{}`", tag.name))?; },
        Err(why) => { message.channel_id.say(format!("Failed to create tag `{}`. Here's why: {:?}", tag_input, why))?; },
    }
});

command!(tag_del(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    if let Ok(tag) = db.get_tag(guild_id.0 as i64, tag_input.clone()) {
        let mut rank_check = false;
        if let Ok(guild_data) = db.get_guild(guild_id.0 as i64) {
            if let Ok(member) = guild_id.member(&message.author.id) {
                if check_rank(guild_data.admin_roles, &member.roles) { rank_check = true }
                if check_rank(guild_data.mod_roles, &member.roles) { rank_check = true }
            }
        }
        if message.author.id.0 as i64 == tag.author || rank_check {
            match db.del_tag(guild_id.0 as i64, tag_input.clone()) {
                Ok(tag) => { message.channel_id.say(format!("Successfully deleted tag `{}`", tag.name))?; },
                Err(why) => { message.channel_id.say(format!("Failed to delete tag `{}`. Here's why: {:?}", tag_input, why))?; },
            }
        } else { message.channel_id.say("You must own this tag in order to delete it.")?; }
    } else { message.channel_id.say("Tag not found.")?; }
});

command!(tag_edit(ctx, message, args) {
    let guild_id = message.guild_id.unwrap();
    let tag_input = args.single_quoted::<String>().unwrap();
    let value = args.rest().to_string();
    if let Ok(mut tag) = db.get_tag(guild_id.0 as i64, tag_input.clone()) {
        if message.author.id.0 as i64 == tag.author {
            tag.data = value.clone();
            match db.update_tag(guild_id.0 as i64, tag_input.clone(), tag) {
                Ok(t) => { message.channel_id.say(format!("Successfully edited tag `{}`", t.name))?; },
                Err(why) => { message.channel_id.say(format!("Failed to edit tag `{}`. Here's why: {:?}", tag_input, why))?; },
            }
        } else { message.channel_id.say("You must own this tag to edit it.")?; }
    } else { message.channel_id.say("Tag not found.")?; }
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
                        .colour(*colours::MAIN)
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
                        .colour(*colours::MAIN)
                ))?;
            }
        }
    };
});

command!(user_info(ctx, message, args) {
    if let Some(guild_lock) = message.guild() {
        let guild = guild_lock.read();
        let premium = match db.get_premium(guild.id.0 as i64) {
            Ok(_) => true,
            Err(_) => false,
        };
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
            if premium {
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

// TODO fix float math on pressure
command!(weather(ctx, message, args) {
    let mut data = ctx.data.lock();
    let loc = args.full();
    message.channel_id.broadcast_typing()?;
    if let Some((city_info, res)) = data.get::<ApiClient>().expect("Failed to get API Client").weather(loc) {
        if let Ok(body) = res {
            let current = body.currently.unwrap();
            let daily = &body.daily.unwrap().data[0];
            let temp = current.temperature.unwrap();
            let temp_high = current.temperature_high.unwrap_or(daily.temperature_high.unwrap());
            let temp_low = current.temperature_low.unwrap_or(daily.temperature_low.unwrap());
            let feels_like = current.apparent_temperature.unwrap();
            let wind = current.wind_speed.unwrap();
            let visi = current.visibility.unwrap();
            let pressure = current.pressure.unwrap();
            let humidity = current.humidity.unwrap()*100.0;
            let icon = match current.icon {
                Some(ic) => {
                    match ic {
                        ClearDay => "The sky is clear",
                        ClearNight => "The sky is clear",
                        Rain => "It is raining",
                        Snow => "It is snowing",
                        Sleet => "It is sleeting",
                        Wind => "It is windy",
                        Fog => "It is foggy",
                        Cloudy => "The sky is cloudy",
                        PartlyCloudyDay => "The sky is partly cloudy",
                        PartlyCloudyNight => "The sky is partly cloudy",
                        Hail => "It is hailing",
                        Thunderstorm => "There is a thunderstorm",
                        Tornado => "There is a tornado",
                    }
                },
                None => "The sky is clear",
            };
            message.channel_id.send_message(|m| m
                .embed(|e| e
                    .title(format!("Weather in {}", city_info))
                    .description(format!("_It is currently **{}°C** with wind of **{} mph** making it feel like **{}°C**. {} with a visibility of about **{} mi**._",
                        temp,
                        wind,
                        feels_like,
                        icon,
                        visi
                    ))
                    .field("Temperature", format!("Current: **{}°C**\nLow/High: **{}°C / {}°C**", temp, temp_low, temp_high), true)
                    .field("Wind Chill", format!("Feels Like: **{}°C**\nWind Speed: **{} mph**", feels_like, wind), true)
                    .field("Atmosphere", format!("Humidity: **{}%**\nPressure: **{} mb**", humidity, pressure), true)
                    .colour(*colours::MAIN)
                    .timestamp(now!())
                    .footer(|f| f.text("Forecast by Dark Sky"))
            ))?;
        }
    }
});

command!(xp(ctx, message, _args) {
    let guild_id = message.guild_id.unwrap();
    let user_data = db.get_user(message.author.id.0 as i64, guild_id.0 as i64)?;
    message.channel_id.say(format!("Your current XP is {}", user_data.xp))?;
});
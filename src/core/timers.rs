use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::utils::*;
use serenity::model::channel::Channel;
use serenity::model::id::*;
use serenity::prelude::{Mentionable, Mutex};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{
    Sender,
    channel
};

use std::thread;
use std::time::Duration;

fn reminder(channel_id: ChannelId, user_id: UserId, dur: String, reminder: &String, id: i32) {
    let check = match channel_id.to_channel() {
        Ok(ch) => { match ch {
            Channel::Private(_) => true,
            _ => false,
        }},
        _ => false,
    };
    check_error!(channel_id.send_message(|m| m
        .content(match check {
            false => user_id.mention(),
            true => String::new(),
        })
        .embed(|e| e
            .title(format!("Reminder from {} ago", dur))
            .colour(*colours::MAIN)
            .description(reminder)
    )));
    check_error!(db.del_timer(id));
}

fn unmute(user_id: UserId, guild_id: GuildId, channel_id: ChannelId, role_id: RoleId) {
    match user_id.to_user() {
        Ok(user) => {
            if let Ok(mut member) = guild_id.member(user_id) {
                if let Ok(_) = member.remove_role(role_id) {
                    check_error!(channel_id.send_message(|m| m
                        .embed(|e| e
                            .title("Member Unmuted Automatically")
                            .colour(*colours::BLUE)
                            .field("Member", format!("{}\n{}", user.tag(), user_id.0), true)
                    )));
                }
            }
        },
        Err(why) => { failed!(USER_FAIL, why); }
    }
}

fn cooldown(user_id: UserId, guild_id: GuildId, mrole_id: RoleId, crole_id: RoleId) {
    if let Ok(mut member) = guild_id.member(user_id) {
        check_error!(member.add_role(mrole_id));
        check_error!(member.remove_role(crole_id));
        debug!("Member removed from cooldown. User ID: {:?}, Guild: {:?}", user_id, guild_id);
    }
}

pub struct TimerClient(Arc<Mutex<Sender<bool>>>);

impl TimerClient {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let tx = Arc::new(Mutex::new(tx));
        let mtx = tx.clone();
        thread::spawn(move || {
            if let Ok(_) = rx.recv() {
                thread::spawn(move || {
                    loop {
                        match db.get_earliest_timer() {
                            Ok(timer) => {
                                let dur = (timer.starttime - timer.endtime) as u64;
                                let itx = mtx.clone();
                                let cond = Arc::new(AtomicBool::new(true));
                                let mc = cond.clone();
                                thread::spawn(move || {
                                    thread::sleep(Duration::from_secs(dur));
                                    if mc.load(Ordering::Relaxed) {
                                        let _ = itx.lock().send(true);
                                    }
                                });
                                if let Ok(opt) = rx.recv() {
                                    match opt {
                                        false => {
                                            cond.store(false, Ordering::Relaxed);
                                            continue;
                                        },
                                        true => {
                                            let parts = timer.data.split("||").map(|s| s.to_string()).collect::<Vec<String>>();
                                            match parts[0].as_str() {
                                                "REMINDER" => {
                                                    // type, channel_id, user_id, dur, reminder
                                                    let cid = ChannelId::from_str(parts[1].as_str()).ok();
                                                    let uid = UserId::from_str(parts[2].as_str()).ok();
                                                    let dur = seconds_to_hrtime(parts[3].parse::<usize>().unwrap_or(0));
                                                    let rem = &parts[4];
                                                    let id = parts[5].parse::<i32>().unwrap_or(0);
                                                    match (cid, uid) {
                                                        (Some(cid), Some(uid)) => { reminder(cid, uid, dur, rem, id); },
                                                        _ => (),
                                                    }
                                                },
                                                "UNMUTE" => {
                                                    // type, user_id, guild_id, mute_role, channel_id, dur
                                                    let uid = UserId::from_str(parts[1].as_str()).ok();
                                                    let gid = match parts[2].parse::<u64>() {
                                                        Ok(g) => Some(GuildId(g)),
                                                        _ => None,
                                                    };
                                                    let rid = RoleId::from_str(parts[3].as_str()).ok();
                                                    let cid = ChannelId::from_str(parts[4].as_str()).ok();
                                                    match (uid, gid, cid, rid) {
                                                        (Some(u), Some(g), Some(c), Some(r)) => { unmute(u,g,c,r); },
                                                        _ => (),
                                                    }
                                                },
                                                "COOLDOWN" => {
                                                    // type, user_id, guild_id, member_role_id, cooldown_role_id
                                                    let uid = UserId::from_str(parts[1].as_str()).ok();
                                                    let gid = match parts[2].parse::<u64>() {
                                                        Ok(g) => Some(GuildId(g)),
                                                        _ => None,
                                                    };
                                                    let mrid = RoleId::from_str(parts[3].as_str()).ok();
                                                    let crid = RoleId::from_str(parts[4].as_str()).ok();
                                                    match (uid, gid, mrid, crid) {
                                                        (Some(u), Some(g), Some(m), Some(c)) => { cooldown(u,g,m,c); },
                                                        _ => (),
                                                    }
                                                },
                                                _ => {},
                                            }
                                            check_error!(db.del_timer(timer.id));
                                        },
                                    }
                                }
                            },
                            Err(why) => {
                                debug!("{:?}", why);
                                use diesel::result::Error::*;
                                match why {
                                    NotFound => { let _ = rx.recv(); },
                                    _ => ()
                                }
                            },
                        }
                    }
                });
            }
        });
        TimerClient(tx)
    }

    pub fn request(&self) {
        let _ = self.0.lock().send(false);
    }
}

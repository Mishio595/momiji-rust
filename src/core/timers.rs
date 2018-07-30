use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};
use threadpool::ThreadPool;
use std::time::Duration;
use std::str::FromStr;
use chrono::Utc;
use serenity::prelude::Mutex;
use serenity::model::id::*;
use serenity::model::channel::Channel;
use core::utils::*;
use core::consts::DB as db;
use core::consts::*;
use core::colours;

pub struct TimerClient {
    pub recv: Arc<Mutex<Receiver<String>>>,
    pub sender: Arc<Mutex<Sender<String>>>,
    pub pool: ThreadPool,
}

impl TimerClient {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let tc = TimerClient {
            recv: Arc::new(Mutex::new(rx)),
            sender: Arc::new(Mutex::new(tx)),
            pool: ThreadPool::new(5),
        };
        let rec = Arc::clone(&tc.recv);
        tc.pool.execute(move || {
            loop {
                match rec.lock().recv() {
                    Ok(data) => {
                        let parts = data.split("||").map(|s| s.to_string()).collect::<Vec<String>>();
                        match parts[0].as_str() {
                            "REMINDER" => {
                                // type, channel_id, user_id, dur, reminder, id
                                if let Ok(channel_id) = ChannelId::from_str(parts[1].as_str()) {
                                    let check = match channel_id.get() {
                                        Ok(ch) => { match ch {
                                            Channel::Private(_) => true,
                                            _ => false,
                                        }},
                                        _ => false,
                                    };
                                    check_error!(channel_id.send_message(|m| m
                                        .content(if !check { format!("<@{}>", parts[2]) } else { String::new() })
                                        .embed(|e| e
                                            .title(format!("Reminder from {} ago", seconds_to_hrtime(parts[3].parse::<usize>().unwrap_or(0))))
                                            .colour(*colours::MAIN)
                                            .description(&parts[4])
                                    )));
                                    check_error!(db.del_timer(parts[5].parse::<i32>().unwrap_or(0)));
                                }
                            },
                            "UNMUTE" => {
                                // type, user_id, guild_id, mute_role, channel_id, dur, id
                                if let Ok(user_id) = UserId::from_str(parts[1].as_str()) {
                                    match user_id.get() {
                                        Ok(user) => {
                                            if let Ok(guild_id) = parts[2].parse::<u64>() {
                                                let guild_id = GuildId(guild_id);
                                                if let Ok(role_id) = RoleId::from_str(parts[3].as_str()) {
                                                    if let Ok(channel_id) = ChannelId::from_str(parts[4].as_str()) {
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
                                                    }
                                                }
                                            }
                                        },
                                        Err(why) => { failed!(USER_FAIL, why); }
                                    }
                                }
                            },
                            "COOLDOWN" => {
                                // type, user_id, guild_id, member_role_id
                                if let Ok(user_id) = UserId::from_str(parts[1].as_str()) {
                                    if let Ok(guild_id) = parts[2].parse::<u64>() {
                                        let guild_id = GuildId(guild_id);
                                        let member_role_id = RoleId::from_str(parts[3].as_str()).expect("Failed to build RoleId");
                                        let cooldown_role_id = RoleId::from_str(parts[4].as_str()).expect("Failed to build RoleId");
                                        if let Ok(mut member) = guild_id.member(user_id) {
                                            if let Ok(_) = member.add_role(member_role_id) {
                                                if let Ok(_) = member.remove_role(cooldown_role_id) {
                                                    info!("Member removed from cooldown. User ID: {:?}, Guild: {:?}", user_id, guild_id);
                                                } else { warn!("Failed to remove cooldown role"); }
                                            } else { warn!("Failed to add member role"); }
                                        }
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    Err(_) => {},
                }
            }
        });
        tc
    }

    pub fn request(&self, data: String, time: u64) {
        let tx = Arc::clone(&self.sender);
        self.pool.execute(move || {
            thread::sleep(Duration::from_secs(time));
            check_error!(tx.lock().send(data));
        });
    }

    pub fn load(&self) {
        let timers = db.get_timers().unwrap();
        for timer in timers.iter() {
            if let Some(dur) = (timer.endtime as u64).checked_sub(Utc::now().timestamp() as u64) {
                let mut data = timer.data.clone();
                data.push_str(format!("||{}", timer.id).as_str());
                let tx = Arc::clone(&self.sender);
                self.pool.execute(move || {
                    thread::sleep(Duration::from_secs(dur));
                    check_error!(tx.lock().send(data));
                });
            } else {
                let mut data = timer.data.clone();
                data.push_str(format!("||{}", timer.id).as_str());
                let tx = Arc::clone(&self.sender);
                check_error!(tx.lock().send(data));
            }
        }
    }
}

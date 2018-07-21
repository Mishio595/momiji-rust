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
use core::colours;

pub struct TimerClient {
    pub recv: Arc<Mutex<Receiver<String>>>,
    pub sender: Arc<Mutex<Sender<String>>>,
    pub pool: ThreadPool,
    pub db: Arc<Mutex<::db::Database>>,
}

impl TimerClient {
    pub fn new(db: Arc<Mutex<::db::Database>>) -> Self {
        let (tx, rx) = channel();
        let tc = TimerClient {
            recv: Arc::new(Mutex::new(rx)),
            sender: Arc::new(Mutex::new(tx)),
            pool: ThreadPool::new(5),
            db: Arc::clone(&db),
        };
        let rec = Arc::clone(&tc.recv);
        tc.pool.execute(move || {
            loop {
                match rec.lock().recv() {
                    Ok(data) => {
                        let parts = data.split("||").map(|s| s.to_string()).collect::<Vec<String>>();
                        if parts[0] == "REMINDER" {
                            // type, channel_id, user_id, dur, reminder, id
                            let channel_id = ChannelId::from_str(parts[1].as_str()).expect("Failed to build ChannelId from string");
                            let check = match channel_id.get() {
                                Ok(ch) => { match ch {
                                    Channel::Private(_) => true,
                                    _ => false,
                                }}
                                _ => false,
                            };
                            channel_id.send_message(|m| m
                                .content(if !check { format!("<@{}>", parts[2]) } else { String::new() })
                                .embed(|e| e
                                    .title(format!("Reminder from {} ago", seconds_to_hrtime(parts[3].parse::<usize>().unwrap())))
                                    .colour(*colours::MAIN)
                                    .description(&parts[4])))
                                .expect("Failed to send message");
                        db.lock().del_timer(parts[5].parse::<i32>().unwrap()).expect("Failed to delete timer");
                        } else if parts[0] == "UNMUTE" {
                            // type, user_id, guild_id, mute_role, channel_id, dur, id
                            let user_id = UserId::from_str(parts[1].as_str()).expect("Failed to build UserId");
                            let user = user_id.get().unwrap();
                            let guild_id = GuildId(parts[2].parse::<u64>().expect("Failed to build GuildId"));
                            let role_id = RoleId::from_str(parts[3].as_str()).expect("Failed to build RoleId");
                            let channel_id = ChannelId::from_str(parts[4].as_str()).expect("Failed to build ChannelId");
                            if let Ok(mut member) = guild_id.member(user_id) {
                                if let Ok(_) = member.remove_role(role_id) {
                                    channel_id.send_message(|m| m
                                        .embed(|e| e
                                            .title("Member Unmuted Automatically")
                                            .colour(*colours::BLUE)
                                            .field("Member", format!("{}\n{}", user.tag(), user_id.0), true)
                                    )).expect("Failed to send message");
                                }
                            }
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
            tx.lock().send(data).unwrap();
        });
    }

    pub fn load(&self) {
        let db = self.db.lock();
        let timers = db.get_timers().unwrap();
        for timer in timers.iter() {
            if let Some(dur) = (timer.endtime as u64).checked_sub(Utc::now().timestamp() as u64) {
                let mut data = timer.data.clone();
                data.push_str(format!("||{}", timer.id).as_str());
                let tx = Arc::clone(&self.sender);
                self.pool.execute(move || {
                    thread::sleep(Duration::from_secs(dur));
                    tx.lock().send(data).unwrap();
                });
            } else {
                let mut data = timer.data.clone();
                data.push_str(format!("||{}", timer.id).as_str());
                let tx = Arc::clone(&self.sender);
                tx.lock().send(data).unwrap();
                db.del_timer(timer.id).expect("Failed to delete timer");
            }
        }
    }
}

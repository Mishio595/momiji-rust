use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};
use threadpool::ThreadPool;
use std::time::Duration;
use std::str::FromStr;
use serenity::prelude::Mutex;
use serenity::model::id::*;
use core::utils::*;
use core::consts::Colours;

pub struct TimerClient {
    pub recv: Arc<Mutex<Receiver<String>>>,
    pub sender: Arc<Mutex<Sender<String>>>,
    pub pool: ThreadPool,
    db: Arc<Mutex<::db::Database>>,
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
                        // type, guild_id, user_id, dur, reminder, id
                        let parts = data.split("||").map(|s| s.to_string()).collect::<Vec<String>>();
                        if parts[0] == "REMINDER" {
                            let user = UserId::from_str(parts[2].as_str()).expect("Failed to build UserId from string").get().expect("Failed to find user");
                            user.dm(|m| m
                                .embed(|e| e
                                    .title(format!("Reminder from {} ago", seconds_to_hrtime(parts[3].parse::<usize>().unwrap())))
                                    .colour(Colours::Main.val())
                                    .description(&parts[4])))
                                .expect("Failed to DM user");
                        db.lock().del_timer(parts[5].parse::<i32>().unwrap());
                        } else if parts[0] == "UNMUTE" {
                            // TODO write unmute
                        }
                    },
                    Err(why) => {},
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
}

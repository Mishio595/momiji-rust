use core::consts::DB as db;
use core::utils::parse_guild;
use std::path::Path;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
};
use serenity::model::channel::Message;
use serenity::prelude::Context;

pub struct Log;
impl Command for Log {
    fn execute(&self, _: &mut Context, message: &Message, _: Args) -> Result<(), CommandError> {
        message.channel_id.send_files(vec![Path::new("./output.log")], |m| m)?;
        Ok(())
    }
}

pub struct Premium;
impl Command for Premium {
    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        let op = args.single::<String>()?;
        let g = args.single_quoted::<String>()?;
        if let Some((guild_id, guild_lock)) = parse_guild(g) {
            let guild = guild_lock.read();
            match op.to_lowercase().as_str() {
                "enable" => {
                    if let Ok(_) = db.new_premium(guild_id.0 as i64) {
                        message.channel_id.say(format!("{} is now premium!", guild.name))?;
                    }
                },
                "disable" => {
                    if let Ok(_) = db.del_premium(guild_id.0 as i64) {
                        message.channel_id.say(format!("{} is no longer premium.", guild.name))?;
                    }
                },
                "set" => {
                    let mut prem = db.get_premium(guild_id.0 as i64)?;
                    prem.tier = args.single::<i32>()?;
                    let pr = db.update_premium(guild_id.0 as i64, prem)?;
                    message.channel_id.say(format!("Updated premium tier for {} to {}.", guild.name, pr.tier))?;
                },
                "show" => {
                    if let Ok(mut prem) = db.get_premium(guild_id.0 as i64) {
                        // TODO add impl Display for PremiumSettings
                        message.channel_id.say(format!("{:?}", prem))?;
                    }
                },
                _ => {},
            }
        }
        Ok(())
    }
}

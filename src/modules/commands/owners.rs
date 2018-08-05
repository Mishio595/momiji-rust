use core::consts::DB as db;
use core::utils::parse_guild;
use std::path::Path;

// Rank 4
/*
command!(git(_ctx, message, args) {
});*/

command!(log(_ctx, message, _args) {
    message.channel_id.send_files(vec![Path::new("./log.txt")], |m| m)?;
});

command!(set_premium(_ctx, message, args) {
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
});

/*command!(restart(_ctx, message, _args) {
});*/

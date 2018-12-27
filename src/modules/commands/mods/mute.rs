use chrono::Utc;
use core::colours;
use core::consts::*;
use core::consts::DB as db;
use core::model::TC;
use core::utils::*;
use serenity::builder::CreateMessage;
use serenity::framework::standard::{
    Args,
    Command,
    CommandError,
    CommandOptions
};
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::model::Permissions;
use serenity::prelude::Context;
use std::sync::Arc;

pub struct Mute;
impl Command for Mute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Mute a user. Can provide an optional reason and time.".to_string()),
            usage: Some("<user_resolvable> [/t time] [/r reason]".to_string()),
            example: Some("@Adelyn /t 1day /r spam".to_string()),
            required_permissions: Permissions::MANAGE_ROLES | Permissions::MUTE_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, ctx: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_lock) = message.guild() {
            let guild = {
                guild_lock.read().clone()
            };
            if let Some((_, mut member)) = parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
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
                    if let Some(mute_role) = guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                        if member.roles.contains(&mute_role.id) {
                            message.channel_id.say("Member already muted.")?;
                        } else {
                            member.add_role(mute_role)?;
                            let user = {
                                member.user.read().clone()
                            };
                            let case = db.new_case(user.id.0 as i64, guild.id.0 as i64, "Mute".to_string(), Some(reason.clone()), message.author.id.0 as i64)?;
                            let mut fields = Vec::new();
                            fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                            fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                            if time != 0 {
                                let data = ctx.data.lock();
                                if let Some(tc_lock) = data.get::<TC>() {
                                    let tc = tc_lock.lock();
                                    let data = format!("UNMUTE||{}||{}||{}||{}||{}||{}",
                                        user.id.0,
                                        guild.id.0,
                                        mute_role.id.0,
                                        if guild_data.modlog && guild_data.modlog_channel > 0 {
                                            guild_data.modlog_channel
                                        } else { message.channel_id.0 as i64 },
                                        time,
                                        case.id);
                                    let start_time = Utc::now().timestamp();
                                    let end_time = start_time + time;
                                    db.new_timer(start_time, end_time, data)?;
                                    tc.request();
                                    fields.push(("Duration", seconds_to_hrtime(time as usize), true));
                                } else {
                                    message.channel_id.say("Something went wrong with the timer.")?;
                                }
                            }
                            if !reason.is_empty() {
                                fields.push(("Reason", reason.to_string(), true));
                            }
                            let response = CreateMessage::default()
                                .embed(|e| e
                                    .title("Member Muted")
                                    .colour(*colours::BLUE)
                                    .fields(fields)
                                    .timestamp(now!()));

                            if guild_data.modlog && guild_data.modlog_channel > 0 {
                                let channel = ChannelId(guild_data.modlog_channel as u64);
                                channel.send_message(|_| response)?;
                            } else {
                                message.channel_id.send_message(|_| response)?;
                            }
                        }
                    } else { message.channel_id.say("No mute role")?; }
                } else {
                    message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
                }
            } else { message.channel_id.say("I couldn't find that user.")?; }
        } else { failed!(GUILD_FAIL); }
        Ok(())
    }
}

pub struct Unmute;
impl Command for Unmute {
    fn options(&self) -> Arc<CommandOptions> {
        let default = CommandOptions::default();
        let options = CommandOptions {
            desc: Some("Unmute a user.".to_string()),
            usage: Some("<user_resolvable>".to_string()),
            example: Some("@Adelyn".to_string()),
            required_permissions: Permissions::MANAGE_ROLES | Permissions::MUTE_MEMBERS,
            ..default
        };
        Arc::new(options)
    }

    fn execute(&self, _: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
        if let Some(guild_lock) = message.guild() {
            let guild = {
                guild_lock.read().clone()
            };
            if let Some((_, mut member)) = parse_user(args.single::<String>().unwrap_or(String::new()), guild.id) {
                let guild_data = db.get_guild(guild.id.0 as i64)?;
                if guild_data.mute_setup {
                    if let Some(mute_role) = guild.roles.values().find(|e| e.name.to_lowercase() == "muted") {
                        let mut fields = Vec::new();
                        let user = {
                            member.user.read().clone()
                        };
                        fields.push(("User", format!("{}\n{}", user.tag(), user.id.0), true));
                        fields.push(("Moderator", format!("{}\n{}", message.author.tag(), message.author.id.0), true));
                        let response = CreateMessage::default()
                            .embed(|e| e
                                .title("Member Unmuted")
                                .colour(*colours::BLUE)
                                .fields(fields)
                                .timestamp(now!()));

                        if member.roles.contains(&mute_role.id) {
                            member.remove_role(mute_role)?;
                            if guild_data.modlog && guild_data.modlog_channel > 0 {
                                let channel = ChannelId(guild_data.modlog_channel as u64);
                                channel.send_message(|_| response)?;
                            } else {
                                message.channel_id.send_message(|_| response)?;
                            }
                        } else {
                            message.channel_id.say("Member was not muted.")?;
                        }
                    } else { message.channel_id.say("No mute role")?; }
                } else {
                    message.channel_id.say("Please run `setup` before using this command. Without it, muting may not work right.")?;
                }
            } else { message.channel_id.say("I couldn't find that user.")?; }
        } else { failed!(GUILD_FAIL); }
        Ok(())
    }
}
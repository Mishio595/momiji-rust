// Unfinished, DO NOT USE

use chrono::{DateTime, Duration, Utc};
use log::debug;
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};
use twilight_model::channel::ChannelType;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Timer {
    Cooldown {
        user: UserId,
        guild: GuildId,
        duration: Duration,
        start: DateTime<Utc>,
    },
    Mute {
        user: UserId,
        guild: GuildId,
        mute_role: RoleId,
        duration: Duration,
        start: DateTime<Utc>,
    },
    Reminder {
        channel: ChannelId,
        recipient: UserId,
        message: String,
        duration: Duration,
        start: DateTime<Utc>,
    },
}
pub(crate) struct TimerClient {
    timers: Arc<HashSet<Timer>>,
}
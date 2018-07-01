use serenity::prelude::*;
use serenity::CACHE;
use serenity::model::{
    id::*,
    channel::*,
    guild::*,
    user::*,
    event::*,
    gateway::{Ready, Game}
};
use std::sync::Arc;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.name);
    }

    fn cached(&self, ctx: Context, guilds: Vec<GuildId>) {
        let mut data = ctx.data.lock();
        let cache = CACHE.read();
        let api = data.get::<::ApiClient>().unwrap();
        let guild_count = guilds.len();
        api.stats_update(cache.user.id.0, guild_count);
        ctx.set_game(Game::listening(&format!("{} guilds | m!help", guild_count)));
    }

    fn message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {

    }

    fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, ids: Vec<MessageId>) {

    }

    fn message_update(&self, ctx: Context, data: MessageUpdateEvent) {

    }

    fn presence_update(&self, ctx: Context, data: PresenceUpdateEvent) {

    }

    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {

    }

    fn guild_delete(&self, ctx: Context, partial_guild: PartialGuild, guild: Option<Arc<RwLock<Guild>>>) {

    }

    fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {

    }

    fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member: Option<Member>) {

    }

    fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {

    }

    fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, user: User) {

    }

    fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, user: User) {

    }
}

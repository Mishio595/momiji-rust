use serenity::CACHE;
use serenity::prelude::*;
use serenity::model::channel::*;
use serenity::model::id::*;
use serenity::utils::*;
use chrono::offset::Utc;
use sys_info;
use procinfo;

// Rank 0

command!(bot_info(ctx, message, _args) {
    let mut data = ctx.data.lock();
    let cache = CACHE.read();
    let shard_count = match data.get::<::SerenityShardManager>() {
        Some(s) => s.lock().shards_instantiated().len(),
        None => {
            error!("Unable to get the shard manager!");
            0
        },
    };
    if let Err(why) = message.channel_id.send_message(|m| m
        .embed(|e| e
            .description("Hi! I'm Momiji, a general purpose bot created in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity).")
            .field("Guilds", cache.guilds.len(), true)
            .field("Shards", shard_count, true)
            .field("Owner", data.get::<::Owner>().unwrap().get().unwrap().tag(), true)
            .field("Support Server", "[Momiji's House](https://discord.gg/YYdpsNc)", true)
            .field("Invite Me!", "[Invite](https://discordapp.com/oauth2/authorize/?permissions=335670488&scope=bot&client_id=345316276098433025)", true)
            .field("Contribute", "[Github](https://github.com/Mishio595/momiji)\n[Patreon](https://www.patreon.com/momijibot)", true)
            .thumbnail(&cache.user.avatar_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()))
            //.timestamp(Utc::now())
            .colour(Colour::new(6138367))
        )
    ) {
        error!("Failed to send message: {:?}", why);
    };
});

command!(cat(ctx, message, args) {
});

command!(color(ctx, message, args) {
});

command!(danbooru(ctx, message, args) {
});

command!(dog(ctx, message, args) {
});

command!(e621(ctx, message, args) {
});

command!(dad_joke(ctx, message, args) {
});

command!(anime(ctx, message, args) {
});

command!(manga(ctx, message, args) {
});

command!(nerdy_info(_ctx, message, _args) {
    if let Err(why) = message.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Nerdy  Info")
            .colour(Colour::new(6138367))
            .field("OS", format!("{} {}", sys_info::os_type().unwrap(), sys_info::os_release().unwrap()), false)
            .field("CPU", format!("**Cores**: {}\n**Speed**: {} MHz", sys_info::cpu_num().unwrap(), sys_info::cpu_speed().unwrap()), false)
            .field("Memory", format!("{} KiB", procinfo::pid::statm_self().unwrap().size), false)
    )) {
        error!("Failed to send message: {:?}", why);
    };
});

command!(ping(_ctx, message, _args) {
    if let Ok(mut m) = message.channel_id.say("Pong!") {
        let t = m.timestamp.timestamp_millis() - message.timestamp.timestamp_millis();
        let _ = m.edit(|m| m.content(format!("Pong! `{} ms`", t)));
    };
});

command!(prefix(ctx, message, args) {
});

command!(remind(ctx, message, args) {
});

command!(asr(ctx, message, args) {
});

command!(rsr(ctx, message, args) {
});

command!(lsr(ctx, message, args) {
});

command!(role_info(ctx, message, args) {
    //TODO: parsing
    let role = RoleId(parse_role(&args.single::<String>().unwrap()[..]).unwrap()).find().unwrap();
    if let Err(why) = message.channel_id.send_message(|m| m
        .embed(|e| e
            .colour(role.colour)
            .field("Name", role.name, true)
    )) {

    };
});

command!(roll(ctx, message, args) {
});

command!(server_info(ctx, message, args) {
});

command!(tag(ctx, message, args) {
});

command!(time(ctx, message, args) {
});

command!(urban(ctx, message, args) {
});

command!(user_info(ctx, message, args) {
});

command!(weather(ctx, message, args) {
});

// Rank 1

command!(mod_info(ctx, message, args) {
});

command!(mute(ctx, message, args) {
});

command!(unmute(ctx, message, args) {
});

command!(notes(ctx, message, args) {
});

command!(register(ctx, message, args) {
});

command!(ar(ctx, message, args) {
});

command!(rr(ctx, message, args) {
});

command!(role_color(ctx, message, args) {
});

command!(watchlist(ctx, message, args) {
});

// Rank 2

command!(config(ctx, message, args) {
});

command!(hackban(ctx, message, args) {
});

command!(ignore(ctx, message, args) {
});

command!(csr(ctx, message, args) {
});

command!(dsr(ctx, message, args) {
});

command!(prune(ctx, message, args) {
});

command!(test(ctx, message, args) {
});

// Rank 3

command!(setup_mute(ctx, message, args) {
});

// Rank 4

command!(git(ctx, message, args) {
});

command!(log(ctx, message, args) {
});

command!(restart(ctx, message, args) {
});

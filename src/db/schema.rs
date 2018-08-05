table! {
    cases (id, user_id, guild_id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        casetype -> Text,
        moderator -> Int8,
        timestamp -> Timestamptz,
    }
}

table! {
    guilds (id) {
        id -> Int8,
        admin_roles -> Array<Int8>,
        audit -> Bool,
        audit_channel -> Int8,
        audit_threshold -> Int2,
        autorole -> Bool,
        autoroles -> Array<Int8>,
        ignored_channels -> Array<Int8>,
        ignore_level -> Int2,
        introduction -> Bool,
        introduction_channel -> Int8,
        introduction_message -> Text,
        introduction_type -> Text,
        mod_roles -> Array<Int8>,
        modlog -> Bool,
        modlog_channel -> Int8,
        mute_setup -> Bool,
        prefix -> Text,
        welcome -> Bool,
        welcome_channel -> Int8,
        welcome_message -> Text,
        welcome_type -> Text,
        commands -> Array<Text>,
        logging -> Array<Text>,
        hackbans -> Array<Int8>,
    }
}

table! {
    notes (id, user_id, guild_id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        note -> Text,
        moderator -> Int8,
        timestamp -> Timestamptz,
    }
}

table! {
    premium (id) {
        id -> Int8,
        tier -> Int4,
        register_member_role -> Nullable<Int8>,
        register_cooldown_role -> Nullable<Int8>,
        register_cooldown_duration -> Nullable<Int4>,
        cooldown_restricted_roles -> Array<Int8>,
    }
}

table! {
    roles (id, guild_id) {
        id -> Int8,
        guild_id -> Int8,
        category -> Text,
        aliases -> Array<Text>,
    }
}

table! {
    tags (guild_id, name) {
        author -> Int8,
        guild_id -> Int8,
        name -> Text,
        data -> Text,
    }
}

table! {
    timers (id) {
        id -> Int4,
        starttime -> Int8,
        endtime -> Int8,
        data -> Text,
    }
}

table! {
    users (id, guild_id) {
        id -> Int8,
        guild_id -> Int8,
        username -> Text,
        nickname -> Text,
        roles -> Array<Int8>,
        watchlist -> Bool,
        xp -> Int8,
        last_message -> Timestamptz,
        registered -> Nullable<Timestamptz>,
    }
}

allow_tables_to_appear_in_same_query!(
    cases,
    guilds,
    notes,
    premium,
    roles,
    tags,
    timers,
    users,
);

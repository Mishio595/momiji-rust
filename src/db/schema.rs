table! {
    guilds (id) {
        id -> Int8,
        admin_roles -> Array<Int8>,
        audit -> Bool,
        audit_channel -> Int8,
        audit_threshold -> Int2,
        autorole -> Bool,
        autoroles -> Array<Int8>,
        ignore_level -> Int2,
        introduction -> Bool,
        introduction_channel -> Int8,
        introduction_message -> Text,
        mod_roles -> Array<Int8>,
        modlog -> Bool,
        modlog_channel -> Int8,
        mute_setup -> Bool,
        prefix -> Text,
        welcome -> Bool,
        welcome_channel -> Int8,
        welcome_message -> Text,
        premium -> Bool,
        premium_tier -> Int2,
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
    roles (role_id, guild_id) {
        role_id -> Int8,
        guild_id -> Int8,
        category -> Text,
        aliases -> Array<Text>,
    }
}

table! {
    users (id, guild_id) {
        id -> Int8,
        guild_id -> Int8,
        username -> Text,
        nickname -> Text,
        roles -> Array<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    guilds,
    notes,
    roles,
    users,
);

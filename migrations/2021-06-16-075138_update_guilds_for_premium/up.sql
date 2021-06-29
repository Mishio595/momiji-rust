ALTER TABLE guilds
ADD COLUMN register_member_role BIGINT;

ALTER TABLE guilds
ADD COLUMN register_cooldown_role BIGINT;

ALTER TABLE guilds
ADD COLUMN register_cooldown_duration INT;

ALTER TABLE guilds
ADD COLUMN cooldown_restricted_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[];
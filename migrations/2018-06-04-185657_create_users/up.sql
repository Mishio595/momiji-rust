CREATE TABLE users (
	id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	username TEXT NOT NULL DEFAULT '',
	nickname TEXT NOT NULL DEFAULT '',
	roles TEXT [] NOT NULL DEFAULT array[]::text[],
	PRIMARY KEY(id, guild_id)
)

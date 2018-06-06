CREATE TABLE users (
	id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	username TEXT NOT NULL DEFAULT '',
	nickname TEXT NOT NULL DEFAULT '',
	roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	watchlist BOOL NOT NULL DEFAULT 'f',
	PRIMARY KEY(id, guild_id)
)

CREATE TABLE users (
	id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	username TEXT NOT NULL DEFAULT '',
	nickname TEXT NOT NULL DEFAULT '',
	roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	watchlist BOOL NOT NULL DEFAULT 'f',
	xp BIGINT NOT NULL DEFAULT 0,
	last_message TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
	PRIMARY KEY(id, guild_id)
)

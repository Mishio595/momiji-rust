CREATE TABLE tags (
	author BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	name TEXT NOT NULL,
	data TEXT NOT NULL,
	PRIMARY KEY(guild_id, name)
)

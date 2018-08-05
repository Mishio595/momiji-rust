CREATE TABLE hackbans(
	id BIGINT,
	guild_id BIGINT,
	reason TEXT,
	PRIMARY KEY(id, guild_id)
)

CREATE TABLE cases (
	id BIGINT,
	guild_id BIGINT,
	casetype TEXT NOT NULL,
	moderator BIGINT NOT NULL,
	timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
	PRIMARY KEY(id, guild_id)
)

CREATE TABLE notes (
	user_id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	index SERIAL NOT NULL,
	note TEXT NOT NULL,
	moderator BIGINT NOT NULL,
	timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
	PRIMARY KEY(user_id, guild_id)
)

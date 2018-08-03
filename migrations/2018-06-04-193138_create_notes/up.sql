CREATE TABLE notes (
	index SERIAL NOT NULL,
	user_id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	note TEXT NOT NULL,
	moderator BIGINT NOT NULL,
	timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
	PRIMARY KEY(index, user_id, guild_id)
)

CREATE TABLE roles (
	role_id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	category TEXT NOT NULL DEFAULT 'Default',
	aliases TEXT [] NOT NULL DEFAULT array[]::text[],
	PRIMARY KEY(role_id, guild_id)
)

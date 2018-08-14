CREATE TABLE roles (
	id BIGINT NOT NULL,
	guild_id BIGINT NOT NULL,
	category TEXT NOT NULL DEFAULT 'Default',
	aliases TEXT [] NOT NULL DEFAULT array[]::text[],
	required_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	forbidden_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	PRIMARY KEY(id, guild_id)
)

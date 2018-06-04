CREATE TABLE guilds (
	id BIGINT PRIMARY KEY,
	admin_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	audit BOOL NOT NULL DEFAULT 'f',
	audit_channel BIGINT NOT NULL DEFAULT 0,
	audit_threshold SMALLINT NOT NULL DEFAULT 0,
	autorole BOOL NOT NULL DEFAULT 'f',
	autoroles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	ignore_level SMALLINT NOT NULL DEFAULT 3,
	introduction BOOL NOT NULL DEFAULT 'f',
	introduction_channel BIGINT NOT NULL DEFAULT 0,
	introduction_message TEXT NOT NULL DEFAULT '',
	mod_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[],
	modlog BOOL NOT NULL DEFAULT 'f',
	modlog_channel BIGINT NOT NULL DEFAULT 0,
	mute_setup BOOL NOT NULL DEFAULT 'f',
	prefix TEXT NOT NULL DEFAULT 'm!',
	welcome BOOL NOT NULL DEFAULT 'f',
	welcome_channel BIGINT NOT NULL DEFAULT 0,
	welcome_message TEXT NOT NULL DEFAULT '',
	premium BOOL NOT NULL DEFAULT 'f',
	premium_tier SMALLINT NOT NULL DEFAULT 0
)


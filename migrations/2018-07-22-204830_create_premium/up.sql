CREATE TABLE premium (
	id BIGINT PRIMARY KEY,
	tier INT NOT NULL DEFAULT 0,
	register_member_role BIGINT,
	register_cooldown_role BIGINT,
	register_cooldown_duration INT,
	cooldown_restricted_roles BIGINT [] NOT NULL DEFAULT array[]::bigint[]
)

CREATE TABLE timers (
	id SERIAL PRIMARY KEY,
	starttime INT NOT NULL,
	endtime INT NOT NULL,
	data TEXT NOT NULL
)

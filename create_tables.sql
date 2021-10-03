CREATE TABLE responses (
	response_id UUID PRIMARY KEY,
	user_id UUID,
	submit_time TIMESTAMP NOT NULL,
	content JSON NOT NULL
);
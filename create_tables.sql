CREATE TABLE responses (
	response_id UUID PRIMARY KEY,
	user_id UUID,
	content JSON NOT NULL
);
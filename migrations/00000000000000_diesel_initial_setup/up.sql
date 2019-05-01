-- Your SQL goes here
CREATE TABLE tickets
(
    id VARCHAR(255) PRIMARY KEY,
    departure_code VARCHAR(255) NOT NULL,
    arrival_code VARCHAR(255) NOT NULL,
    departure_time INTEGER NOT NULL,
    arrival_time INTEGER NOT NULL,
    price INTEGER NOT NULL
)
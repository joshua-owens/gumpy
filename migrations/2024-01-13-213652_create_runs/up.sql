CREATE TABLE runs
(
    id         SERIAL PRIMARY KEY,
    distance   DECIMAL NOT NULL,
    duration INTERVAL NOT NULL,
    date       DATE    NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
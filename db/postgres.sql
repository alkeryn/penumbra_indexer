DROP TABLE IF EXISTS blocks;

CREATE TABLE IF NOT EXISTS blocks (
    id BIGINT PRIMARY KEY,  -- using BIGINT for usize compatibility
    data JSONB NOT NULL     -- JSONB is more efficient than JSON type
);

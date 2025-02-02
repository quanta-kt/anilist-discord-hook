CREATE TABLE LastPublishedTimestamp(
    last_published_timestamp TIMESTAMP NOT NULL,

    -- Allow only a single row in this table
    id SMALLINT
        CONSTRAINT ensure_singleton CHECK (id = 0)
        UNIQUE
        DEFAULT 0
)

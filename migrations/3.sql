CREATE TABLE upscales(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    file_id TEXT NOT NULL,
    url TEXT NOT NULL,
    width SMALLINT NOT NULL,
    height SMALLINT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    generate_media_dto JSONB,
    seed VARCHAR(255),
    source VARCHAR(255) NOT NULL,
    model VARCHAR(255),
    created_at BIGINT NOT NULL
);
CREATE TABLE users(
    id VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    username_key VARCHAR(255) NOT NULL UNIQUE,
    displayname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_key VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE devices(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    messaging_token TEXT,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE posts(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    media JSONB,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE media(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    file_id TEXT NOT NULL,
    url TEXT NOT NULL,
    width SMALLINT NOT NULL,
    height SMALLINT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    source VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE generate_media_requests(
    id VARCHAR(255) PRIMARY KEY,
    status VARCHAR(255) NOT NULL,
    generate_media_dto JSONB NOT NULL,
    created_at BIGINT NOT NULL
);
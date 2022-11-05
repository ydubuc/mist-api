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
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE posts(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    media_url TEXT,
    media_width: SMALLINT,
    media_height: SMALLINT,
    media_mime_type: VARCHAR(255),
    media_source: VARCHAR(255),
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE media(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    width SMALLINT NOT NULL,
    height SMALLINT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    source VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL
);
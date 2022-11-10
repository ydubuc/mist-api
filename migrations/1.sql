CREATE TABLE users(
    id VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    username_key VARCHAR(255) NOT NULL UNIQUE,
    displayname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_key VARCHAR(255) NOT NULL UNIQUE,
    avatar_url TEXT,
    password_hash VARCHAR(255) NOT NULL,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX users_updated_at ON users (updated_at);
CREATE INDEX users_created_at ON users (created_at);

CREATE TABLE devices(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    messaging_token TEXT,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX devices_updated_at ON devices (updated_at);
CREATE INDEX devices_created_at ON devices (created_at);

CREATE TABLE posts(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    media JSONB,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX posts_updated_at ON posts (updated_at);
CREATE INDEX posts_created_at ON posts (created_at);

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

CREATE INDEX media_mime_type ON media (mime_type);
CREATE INDEX media_created_at ON media (created_at);

CREATE TABLE generate_media_requests(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(255) NOT NULL,
    generate_media_dto JSONB NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX generate_media_requests_status ON generate_media_requests (status);
CREATE INDEX generate_media_requests_created_at ON generate_media_requests (created_at);
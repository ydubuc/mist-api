CREATE TABLE users(
    id VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    username_key VARCHAR(255) NOT NULL UNIQUE,
    displayname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_key VARCHAR(255) NOT NULL UNIQUE,
    email_pending VARCHAR(255),
    avatar_url TEXT,
    password_hash VARCHAR(255) NOT NULL,
    roles TEXT [],
    ink INTEGER NOT NULL,
    ink_sum INTEGER NOT NULL,
    ink_pending INTEGER NOT NULL,
    delete_pending BOOLEAN NOT NULL,
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
    roles TEXT [],
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX devices_user_id ON devices (user_id);
CREATE INDEX devices_updated_at ON devices (updated_at);
CREATE INDEX devices_created_at ON devices (created_at);

CREATE TABLE posts(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    media JSONB,
    generate_media_dto JSONB,
    published BOOLEAN NOT NULL DEFAULT TRUE,
    reports_count SMALLINT NOT NULL,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX posts_user_id ON posts (user_id);
CREATE INDEX posts_published ON posts (published);
CREATE INDEX posts_updated_at ON posts (updated_at);
CREATE INDEX posts_created_at ON posts (created_at);

CREATE TABLE posts_reports(
    id VARCHAR(510) PRIMARY KEY,
    post_id VARCHAR(255) REFERENCES posts(id) ON DELETE CASCADE,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE media(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    file_id TEXT NOT NULL,
    post_id VARCHAR(255),
    url TEXT NOT NULL,
    width SMALLINT NOT NULL,
    height SMALLINT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    generate_media_dto JSONB,
    seed VARCHAR(255),
    source VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX media_user_id ON media (user_id);
CREATE INDEX media_source ON media (source);
CREATE INDEX media_created_at ON media (created_at);

CREATE TABLE generate_media_requests(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(255) NOT NULL,
    generate_media_dto JSONB NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX generate_media_requests_user_id ON generate_media_requests (user_id);
CREATE INDEX generate_media_requests_status ON generate_media_requests (status);
CREATE INDEX generate_media_requests_created_at ON generate_media_requests (created_at);

CREATE TABLE transactions(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE follows(
    id TEXT PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    follows_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    followed_at BIGINT NOT NULL
);

CREATE INDEX follows_follows_id ON follows (follows_id);
CREATE INDEX follows_followed_at ON follows (followed_at);

CREATE TABLE blocks(
    id TEXT PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    blocked_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    blocked_at BIGINT NOT NULL
);

CREATE INDEX blocks_user_id ON blocks (user_id);
CREATE INDEX blocks_blocked_id ON blocks (blocked_id);
CREATE INDEX blocks_blocked_at ON blocks (blocked_at);
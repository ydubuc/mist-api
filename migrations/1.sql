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

CREATE INDEX users_updated_at_desc ON users (updated_at DESC);
CREATE INDEX users_created_at_desc ON users (created_at DESC);

CREATE TABLE devices(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    messaging_token TEXT,
    roles TEXT [],
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX devices_user_id_asc ON devices (user_id ASC);
CREATE INDEX devices_updated_at_desc ON devices (updated_at DESC);
CREATE INDEX devices_created_at_desc ON devices (created_at DESC);

CREATE TABLE posts(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT,
    media JSONB,
    generate_media_dto JSONB,
    published BOOLEAN NOT NULL DEFAULT TRUE,
    featured BOOLEAN NOT NULL DEFAULT FALSE,
    reports_count SMALLINT NOT NULL,
    updated_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX posts_user_id_ASC ON posts (user_id ASC);
CREATE INDEX posts_updated_at_asc ON posts (updated_at ASC);
CREATE INDEX posts_updated_at_desc ON posts (updated_at DESC);
CREATE INDEX posts_created_at_asc ON posts (created_at ASC);
CREATE INDEX posts_created_at_desc ON posts (created_at DESC);

CREATE TABLE posts_reports(
    id VARCHAR(255) PRIMARY KEY,
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
    model VARCHAR(255),
    created_at BIGINT NOT NULL
);

CREATE INDEX media_user_id_asc ON media (user_id ASC);
CREATE INDEX media_source_asc ON media (source ASC);
CREATE INDEX media_created_at_desc ON media (created_at DESC);

CREATE TABLE generate_media_requests(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(255) NOT NULL,
    generate_media_dto JSONB NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX generate_media_requests_user_id_asc ON generate_media_requests (user_id ASC);
CREATE INDEX generate_media_requests_status_asc ON generate_media_requests (status ASC);
CREATE INDEX generate_media_requests_created_at_desc ON generate_media_requests (created_at DESC);

CREATE TABLE transactions(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE follows(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    follows_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    followed_at BIGINT NOT NULL
);

CREATE INDEX follows_followed_at_desc ON follows (followed_at DESC);

CREATE TABLE blocks(
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    blocked_id VARCHAR(255) REFERENCES users(id) ON DELETE CASCADE,
    blocked_at BIGINT NOT NULL
);

CREATE INDEX blocks_blocked_at_desc ON blocks (blocked_at DESC);



-- UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_2_1"') WHERE source = 'mist_stability';
-- UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE source = 'labml';
-- UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE source = 'stable_horde';
-- UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"dalle"') WHERE source = 'dalle';

-- UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_2_1"') WHERE generate_media_dto->>'generator' = 'mist_stability';
-- UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE generate_media_dto->>'generator' = 'labml';
-- UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE generate_media_dto->>'generator' = 'stable_horde';
-- UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"dalle"') WHERE generate_media_dto->>'generator' = 'dalle';
-- UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE generate_media_dto->>'generator' = 'dream';


UPDATE media SET model = 'stable_diffusion_2_1' WHERE source = 'mist_stability';
UPDATE media SET source = 'mist' WHERE source = 'mist_stability';

UPDATE media SET model = 'stable_diffusion_1_5' WHERE source = 'stable_horde';

UPDATE media SET model = 'dalle' WHERE source = 'dalle';
UPDATE media SET model = 'stable_diffusion_2_1' WHERE source = 'mist_stability';
UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_2_1"') WHERE source = 'mist_stability';
UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_2_1"') WHERE generate_media_dto->>'generator' = 'mist_stability';
UPDATE generate_media_requests SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_2_1"') WHERE generate_media_dto->>'generator' = 'mist_stability';

UPDATE media SET source = 'mist' WHERE source = 'mist_stability';
UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{generator}', '"mist"') WHERE generate_media_dto->>'generator' = 'mist_stability';
UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{generator}', '"mist"') WHERE generate_media_dto->>'generator' = 'mist_stability';
UPDATE generate_media_requests SET generate_media_dto = jsonb_set(generate_media_dto, '{generator}', '"mist"') WHERE generate_media_dto->>'generator' = 'mist_stability';

UPDATE media SET model = 'stable_diffusion_1_5' WHERE source = 'stable_horde';
UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE source = 'stable_horde';
UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE generate_media_dto->>'generator' = 'stable_horde';
UPDATE generate_media_requests SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"stable_diffusion_1_5"') WHERE generate_media_dto->>'generator' = 'stable_horde';

UPDATE media SET model = 'dalle' WHERE source = 'dalle';
UPDATE media SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"dalle"') WHERE source = 'dalle';
UPDATE posts SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"dalle"') WHERE generate_media_dto->>'generator' = 'dalle';
UPDATE generate_media_requests SET generate_media_dto = jsonb_set(generate_media_dto, '{model}', '"dalle"') WHERE generate_media_dto->>'generator' = 'dalle';
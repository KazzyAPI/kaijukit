-- Content kinds beyond pages: custom content carries its own type string, smart links a URL.
ALTER TABLE content ADD COLUMN custom_type text;
ALTER TABLE content ADD COLUMN embed_url text;
CREATE INDEX content_custom_type_idx ON content (custom_type, space_id) WHERE type = 'custom';

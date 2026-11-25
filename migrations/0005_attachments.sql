-- An attachment is a content row of type 'attachment' (title = filename); its container is a page.
-- Per-version file metadata lives in attachment_versions; blobs are addressed by sha256.
CREATE TABLE attachments (
    content_id   bigint PRIMARY KEY REFERENCES content (id) ON DELETE CASCADE,
    container_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    file_id      text NOT NULL DEFAULT gen_random_uuid()::text
);
CREATE INDEX attachments_container_idx ON attachments (container_id);

CREATE TABLE attachment_versions (
    content_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    number     integer NOT NULL,
    blob_key   text NOT NULL,
    media_type text NOT NULL,
    file_size  bigint NOT NULL,
    comment    text NOT NULL DEFAULT '',
    PRIMARY KEY (content_id, number)
);
CREATE INDEX attachment_versions_blob_idx ON attachment_versions (blob_key);

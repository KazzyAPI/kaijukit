-- Comments are content rows of type 'comment' (versions live in content_versions); this table
-- holds what is specific to them. Deleting a container or a parent comment deletes the thread.
CREATE TABLE comments (
    content_id         bigint PRIMARY KEY REFERENCES content (id) ON DELETE CASCADE,
    kind               text NOT NULL CHECK (kind IN ('footer', 'inline')),
    container_id       bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    parent_id          bigint REFERENCES comments (content_id) ON DELETE CASCADE,
    marker_ref         text,
    original_selection text,
    resolution         text NOT NULL DEFAULT 'open' CHECK (resolution IN ('open', 'reopened', 'resolved', 'dangling')),
    resolved_by        bigint REFERENCES users (id),
    resolved_at        timestamptz
);
CREATE INDEX comments_container_idx ON comments (container_id, kind, content_id);
CREATE INDEX comments_parent_idx ON comments (parent_id, content_id);

CREATE FUNCTION kk_drop_comment_content() RETURNS trigger AS $$
BEGIN
    DELETE FROM content WHERE id = OLD.content_id;
    RETURN NULL;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER comments_drop_content AFTER DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION kk_drop_comment_content();

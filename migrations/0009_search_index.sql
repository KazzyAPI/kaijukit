-- Stored search document per content row: plain text of the current version, its modified time
-- and a tsvector with a GIN index, kept current by triggers in the writing transaction.
CREATE TABLE content_search (
    content_id bigint PRIMARY KEY REFERENCES content (id) ON DELETE CASCADE,
    modified   timestamptz NOT NULL,
    txt        text NOT NULL,
    tsv        tsvector NOT NULL
);
CREATE INDEX content_search_tsv_idx ON content_search USING gin (tsv);

CREATE FUNCTION kk_reindex(cid bigint) RETURNS void LANGUAGE sql AS $$
    INSERT INTO content_search (content_id, modified, txt, tsv)
    SELECT c.id, v.created_at, t.txt, to_tsvector('english', c.title || ' ' || t.txt)
    FROM content c
    JOIN content_versions v ON v.content_id = c.id AND v.number = c.version_number
    CROSS JOIN LATERAL (
        SELECT COALESCE(string_agg(x #>> '{}', ' '), '') AS txt
        FROM jsonb_path_query(v.body, 'strict $.**.text') x) t
    WHERE c.id = cid
    ON CONFLICT (content_id) DO UPDATE
        SET modified = EXCLUDED.modified, txt = EXCLUDED.txt, tsv = EXCLUDED.tsv
$$;

CREATE FUNCTION kk_reindex_version() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN PERFORM kk_reindex(NEW.content_id); RETURN NULL; END $$;
CREATE FUNCTION kk_reindex_content() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN PERFORM kk_reindex(NEW.id); RETURN NULL; END $$;

CREATE TRIGGER content_versions_reindex AFTER INSERT OR UPDATE OF body, created_at
    ON content_versions FOR EACH ROW EXECUTE FUNCTION kk_reindex_version();
CREATE TRIGGER content_reindex AFTER UPDATE OF version_number, title
    ON content FOR EACH ROW EXECUTE FUNCTION kk_reindex_content();

SELECT kk_reindex(id) FROM content;

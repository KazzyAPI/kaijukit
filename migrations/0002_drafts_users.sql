-- Unpublished edits of a published page; the content row keeps the current version.
CREATE TABLE content_drafts (
    content_id bigint PRIMARY KEY REFERENCES content (id) ON DELETE CASCADE,
    title      text NOT NULL,
    body       jsonb NOT NULL,
    author_id  bigint NOT NULL REFERENCES users (id),
    updated_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE users ADD COLUMN account_type text NOT NULL DEFAULT 'atlassian';

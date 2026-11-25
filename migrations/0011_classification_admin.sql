-- Classification levels (site configuration), per-content and per-space-default assignments, admin keys.
CREATE TABLE classification_levels (
    id          text PRIMARY KEY,
    position    integer NOT NULL,
    name        text NOT NULL,
    description text NOT NULL DEFAULT '',
    guideline   text NOT NULL DEFAULT '',
    color       text NOT NULL DEFAULT 'GREY',
    status      text NOT NULL DEFAULT 'PUBLISHED',
    is_default  boolean NOT NULL DEFAULT false
);
CREATE UNIQUE INDEX classification_levels_default_idx ON classification_levels (is_default) WHERE is_default;
INSERT INTO classification_levels (id, position, name, description, guideline, color, is_default) VALUES
    ('kk-level-public', 1, 'Public', 'Safe to share with anyone', '', 'GREEN', true),
    ('kk-level-internal', 2, 'Internal', 'For people inside the organization', '', 'BLUE', false),
    ('kk-level-confidential', 3, 'Confidential', 'Limited to those who need it', '', 'ORANGE', false);

CREATE TABLE content_classification (
    content_id bigint PRIMARY KEY REFERENCES content (id) ON DELETE CASCADE,
    level_id   text NOT NULL REFERENCES classification_levels (id)
);
CREATE TABLE space_classification (
    space_id bigint PRIMARY KEY REFERENCES spaces (id) ON DELETE CASCADE,
    level_id text NOT NULL REFERENCES classification_levels (id)
);

CREATE TABLE admin_keys (
    user_id    bigint PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    expires_at timestamptz NOT NULL
);

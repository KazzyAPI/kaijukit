-- One sequence for every id so ids are unique across entity types.
CREATE SEQUENCE kk_id_seq;

CREATE TABLE users (
    id           bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    email        text NOT NULL,
    display_name text NOT NULL,
    is_admin     boolean NOT NULL DEFAULT false,
    created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX users_email_key ON users (lower(email));

CREATE TABLE api_tokens (
    id         bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    user_id    bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    label      text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE spaces (
    id          bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    key         text NOT NULL,
    name        text NOT NULL,
    type        text NOT NULL DEFAULT 'global',
    status      text NOT NULL DEFAULT 'current',
    description text NOT NULL DEFAULT '',
    author_id   bigint NOT NULL REFERENCES users (id),
    homepage_id bigint,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX spaces_key_key ON spaces (upper(key));

-- Pages, folders, whiteboards, ... share one table keyed by type.
-- trashed_with is the id of the content whose trashing trashed this row (restore/purge unit).
CREATE TABLE content (
    id             bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    type           text NOT NULL,
    status         text NOT NULL,
    space_id       bigint NOT NULL REFERENCES spaces (id),
    parent_id      bigint REFERENCES content (id),
    position       integer NOT NULL DEFAULT 0,
    title          text NOT NULL,
    author_id      bigint NOT NULL REFERENCES users (id),
    version_number integer NOT NULL DEFAULT 1,
    trashed_with   bigint,
    created_at     timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX content_parent_idx ON content (parent_id, position, id);
CREATE INDEX content_space_idx ON content (space_id, id);
CREATE INDEX content_trashed_with_idx ON content (trashed_with) WHERE trashed_with IS NOT NULL;
CREATE UNIQUE INDEX content_page_title_key ON content (space_id, lower(title))
    WHERE type = 'page' AND status IN ('current', 'draft', 'archived') AND title <> '';

-- Canonical body is ADF (jsonb); other representations are derived on read.
CREATE TABLE content_versions (
    content_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    number     integer NOT NULL,
    title      text NOT NULL,
    body       jsonb NOT NULL,
    author_id  bigint NOT NULL REFERENCES users (id),
    message    text NOT NULL DEFAULT '',
    minor_edit boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (content_id, number)
);

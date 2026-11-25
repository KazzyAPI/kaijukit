-- Page templates: global (space_id NULL) or per space. The body is ADF like page bodies.
CREATE TABLE templates (
    id          bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    space_id    bigint REFERENCES spaces (id) ON DELETE CASCADE,
    name        text NOT NULL,
    description text NOT NULL DEFAULT '',
    labels      jsonb NOT NULL DEFAULT '[]',
    body        jsonb NOT NULL,
    author_id   bigint NOT NULL REFERENCES users (id),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX templates_space_idx ON templates (space_id, id);

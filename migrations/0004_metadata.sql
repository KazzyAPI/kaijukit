CREATE TABLE labels (
    id     bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    prefix text NOT NULL,
    name   text NOT NULL,
    UNIQUE (prefix, name)
);

CREATE TABLE content_labels (
    content_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    label_id   bigint NOT NULL REFERENCES labels (id),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (content_id, label_id)
);
CREATE INDEX content_labels_label_idx ON content_labels (label_id);

CREATE TABLE space_labels (
    space_id   bigint NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    label_id   bigint NOT NULL REFERENCES labels (id),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, label_id)
);

CREATE TABLE content_properties (
    id             bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    owner_id       bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    key            text NOT NULL,
    value          jsonb NOT NULL,
    version_number integer NOT NULL DEFAULT 1,
    message        text NOT NULL DEFAULT '',
    created_by     bigint NOT NULL REFERENCES users (id),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_by     bigint NOT NULL REFERENCES users (id),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (owner_id, key)
);

CREATE TABLE space_properties (
    id             bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    owner_id       bigint NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    key            text NOT NULL,
    value          jsonb NOT NULL,
    version_number integer NOT NULL DEFAULT 1,
    message        text NOT NULL DEFAULT '',
    created_by     bigint NOT NULL REFERENCES users (id),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_by     bigint NOT NULL REFERENCES users (id),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (owner_id, key)
);

CREATE TABLE app_properties (
    key        text PRIMARY KEY,
    value      jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE likes (
    content_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    user_id    bigint NOT NULL REFERENCES users (id),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (content_id, user_id)
);

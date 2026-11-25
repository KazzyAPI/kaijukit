-- Rows mirror taskItem nodes of a published page body; the body stays the source of truth.
CREATE TABLE tasks (
    id           bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    page_id      bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    space_id     bigint NOT NULL REFERENCES spaces (id),
    local_id     text NOT NULL,
    status       text NOT NULL DEFAULT 'incomplete',
    body         jsonb NOT NULL,
    blank        boolean NOT NULL,
    created_by   bigint NOT NULL REFERENCES users (id),
    assigned_to  bigint REFERENCES users (id),
    completed_by bigint REFERENCES users (id),
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    due_at       timestamptz,
    completed_at timestamptz,
    UNIQUE (page_id, local_id)
);
CREATE INDEX tasks_space_idx ON tasks (space_id);

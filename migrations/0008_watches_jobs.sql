-- Watches, in-app notifications and the background job queue.
CREATE TABLE watches (
    user_id     bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_type text NOT NULL CHECK (target_type IN ('content', 'space')),
    target_id   bigint NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, target_type, target_id)
);
CREATE INDEX watches_target_idx ON watches (target_type, target_id);

CREATE TABLE notifications (
    id         bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    user_id    bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content_id bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    kind       text NOT NULL,
    actor_id   bigint REFERENCES users (id) ON DELETE SET NULL,
    read       boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX notifications_user_idx ON notifications (user_id, id DESC);

-- state: queued -> running -> done, or back to queued with a later run_at, or failed.
CREATE TABLE jobs (
    id         bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    kind       text NOT NULL,
    payload    jsonb NOT NULL,
    state      text NOT NULL DEFAULT 'queued' CHECK (state IN ('queued', 'done', 'failed')),
    attempts   integer NOT NULL DEFAULT 0,
    run_at     timestamptz NOT NULL DEFAULT now(),
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX jobs_due_idx ON jobs (run_at) WHERE state = 'queued';

-- Long-running task records. Work runs inline today, so a task is finished when it is created.
CREATE TABLE long_tasks (
    id          bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    kind        text NOT NULL,
    user_id     bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    successful  boolean NOT NULL DEFAULT true,
    messages    jsonb NOT NULL DEFAULT '[]',
    created_at  timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz NOT NULL DEFAULT now()
);

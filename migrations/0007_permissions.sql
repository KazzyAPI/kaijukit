-- Groups, space permission grants and content restrictions.
CREATE TABLE groups (
    id         bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    name       text NOT NULL,
    -- the everyone group: every user is a member, membership rows are not used
    everyone   boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX groups_name_key ON groups (lower(name));

CREATE TABLE group_members (
    group_id bigint NOT NULL REFERENCES groups (id) ON DELETE CASCADE,
    user_id  bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, user_id)
);
CREATE INDEX group_members_user_idx ON group_members (user_id);

INSERT INTO groups (name, everyone) VALUES ('kk-users', true);

CREATE TABLE space_permissions (
    id           bigint PRIMARY KEY DEFAULT nextval('kk_id_seq'),
    space_id     bigint NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    subject_type text NOT NULL CHECK (subject_type IN ('user', 'group')),
    subject_id   bigint NOT NULL,
    operation    text NOT NULL,
    target       text NOT NULL,
    UNIQUE (space_id, subject_type, subject_id, operation, target)
);

CREATE TABLE content_restrictions (
    content_id   bigint NOT NULL REFERENCES content (id) ON DELETE CASCADE,
    operation    text NOT NULL CHECK (operation IN ('read', 'update')),
    subject_type text NOT NULL CHECK (subject_type IN ('user', 'group')),
    subject_id   bigint NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    created_by   bigint REFERENCES users (id) ON DELETE SET NULL,
    PRIMARY KEY (content_id, operation, subject_type, subject_id)
);

-- Spaces that predate permissions: the everyone group gets the defaults, the author everything.
INSERT INTO space_permissions (space_id, subject_type, subject_id, operation, target)
SELECT s.id, 'group', g.id, p.op, p.target
FROM spaces s CROSS JOIN groups g
CROSS JOIN (VALUES ('read', 'space'), ('create', 'page'), ('create', 'blogpost'), ('create', 'comment'),
    ('create', 'attachment'), ('delete', 'comment'), ('delete', 'attachment')) AS p (op, target)
WHERE g.everyone;

INSERT INTO space_permissions (space_id, subject_type, subject_id, operation, target)
SELECT s.id, 'user', s.author_id, p.op, p.target
FROM spaces s
CROSS JOIN (VALUES ('read', 'space'), ('create', 'page'), ('create', 'blogpost'), ('create', 'comment'),
    ('create', 'attachment'), ('delete', 'page'), ('delete', 'blogpost'), ('delete', 'comment'),
    ('delete', 'attachment'), ('delete', 'space'), ('export', 'space'), ('administer', 'space'),
    ('archive', 'page'), ('restrict_content', 'space')) AS p (op, target);

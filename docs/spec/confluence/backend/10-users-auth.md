# 10 Users, groups, auth

Last reviewed: 2026-11-24

Structure: [v1 Users](../../../parity/confluence-api/v1/users.md), [v1 Group](../../../parity/confluence-api/v1/group.md), [v2 User](../../../parity/confluence-api/v2/user.md), [v1 User properties](../../../parity/confluence-api/v1/user-properties.md).

- **Identity:** `accountId` opaque string per person; stable across email changes; `accountType` `atlassian` (human), `app`, `customer`. Users can be **unknown/anonymous** (`anonymous` account for unauthenticated; "unknown" placeholder for deleted users with the reference's display name `[V]` replaced by our wording).
- **Privacy:** email, full name visibility follow the user's profile visibility settings; API returns `email` empty/redacted when hidden; `publicName` is the display fallback `[V]`.
- **Directory:** users and groups come from the site directory. Sources: local, OIDC/SAML JIT provisioning, SCIM v2 (provisioning API `[V]` compat), LDAP later. Group membership drives global/space permissions.
- **Auth for the UI:** local password (+ optional 2FA TOTP `[V]`), OIDC, SAML 2.0 SSO, session cookies, "remember me", logout everywhere, session timeout settings.
- **Auth for APIs:** API tokens (user-created, revocable, shown once, label, expiry), OAuth 2.0 apps (client credentials registry, authorization code + refresh, scopes, consent screen, revocation), personal access tokens `[V]`. Basic auth password is the API token, never the account password.
- **Bulk user lookups:** v2 user-by-ids, v1 bulk lookup, current user, user watch status, user content/groups membership.
- **User deletion/deactivation:** content authored remains attributed; display shows deactivated marker; mentions render as unknown `[V]`.
- **Personal space:** created on first use or admin action, key `~{accountId}` `[V]`.
- **Profile fields:** displayName, picture, position, department, location, timezone, locale; extended via user properties.
- **External collaborators/customers** `[V]`: restricted account type seeing only shared spaces.

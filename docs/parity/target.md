# Parity target

Last reviewed: 2026-11-24

Parity is measured against ONE pinned reference. Change it only via PR that re-reviews every parity doc.

| Field | Value |
|---|---|
| Product | Jira Software (issues, workflows, JQL, permissions, REST API) |
| Edition | Cloud (Premium not required) — **TBD: confirm** |
| API version | REST API v3 — **TBD: pin the OpenAPI spec snapshot date** |
| Reference instance | **TBD:** free-tier cloud site owned by the maintainers |
| Docs/wiki target | Confluence Cloud — deferred, last in order of work |

## Open decision
Cloud vs Data Center. Cloud has a public OpenAPI spec (easier to conform to) but changes continuously, so we pin a dated snapshot. Data Center has versioned releases but a smaller public spec.

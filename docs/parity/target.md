# Parity target

Last reviewed: 2026-11-24

Parity is measured against ONE pinned reference. Change it only via PR that re-reviews every parity doc.

| Field | Value |
|---|---|
| Product | Jira Software (issues, workflows, JQL, permissions, REST API) |
| Edition | Cloud (Premium not required) — **TBD: confirm** |
| API version | REST API v3 — Confluence v1+v2 OpenAPI snapshots pinned 2026-11-24 (sha256 in `scripts/confluence_inventory.py`); Jira snapshot still TBD |
| Reference instance | **TBD:** free-tier cloud site owned by the maintainers |
| Docs/wiki target | **Confluence Cloud — now the first subsystem.** Specs: [confluence-parity.md](confluence-parity.md), [v2 inventory](confluence-rest-v2.md), [v1 inventory](confluence-rest-v1.md). API snapshot pinned by sha256 in `scripts/confluence_inventory.py`. |

## Open decision
Cloud vs Data Center. Cloud has a public OpenAPI spec (easier to conform to) but changes continuously, so we pin a dated snapshot. Data Center has versioned releases but a smaller public spec.

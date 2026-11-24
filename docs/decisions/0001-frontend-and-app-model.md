# ADR 0001: Atlaskit as frontend base; app model; Kubernetes

Last reviewed: 2026-11-24 · status: proposed (needs legal sign-off on item 1)

## Evidence (npm registry, checked 2026-11-24)

| Package | License field | LICENSE file |
|---|---|---|
| `@atlaskit/button`, `icon`, `tokens`, `renderer`, `editor-core`, `adf-schema`, `editor-prosemirror` | Apache-2.0 | Apache-2.0 (checked `icon`, `editor-core`) |
| `@forge/react` (Forge UI Kit) | "SEE LICENSE" | Use only under the Atlassian Developer Terms; not open source |

## Decisions

1. **Use Atlaskit (the design system and editor packages) as our frontend base.** The code is Apache-2.0, same as ours. This gives us the real editor, renderer, ADF schema, tokens and component set the reference product is built from, which is the cheapest route to editor and ADF parity. Apache-2.0 grants no trademark rights: we ship no Atlassian logos/marks and the naming policy applies to every string we render, including strings inside dependencies (audit them). **Action:** get a legal review of using Atlaskit outside Atlassian products (design-guideline terms, any package-level notices) before the first release.
2. **Do not use `@forge/react` (UI Kit).** Its license is restrictive and it only works inside the Forge runtime. UI Kit components are thin wrappers over Atlaskit, so using Atlaskit directly gives the same components. A UI Kit compatibility layer is only needed if Forge apps are supported (decision 3).
3. **App model:** REST + OAuth + API tokens + event webhooks now. Connect-compatible app layer (descriptor, `installed` callback, JWT, iframe modules) is phase 2. **Forge runtime is out of scope**; we do not rebuild the hosted serverless runtime. We may later offer our own extension model.
4. **Deployment:** Kubernetes first (Helm chart, Postgres, object storage, worker Deployments for jobs, collab service with sticky routing). Docker Compose only for local development.

## Consequences and risks

- The editor stack changes from "Tiptap + schema generated from ADF" to **Atlaskit editor + renderer**. Frontend specs F04/F08 updated accordingly.
- **Collaboration conflict:** Atlaskit's collaborative editing (`collab-provider`) speaks Atlassian's own collab protocol, not Yjs. Options: (a) implement a server for that protocol (it must be specified from observed client behavior; the client package is public, but the server contract is not documented), (b) bridge to Yjs/yrs with a custom provider. Decide before building realtime; until then realtime stays last.
- Atlaskit expects backing services (media/file store, emoji, mentions, smart-link metadata, collab, analytics, feature flags). Each is a service we implement; they are listed in backend specs 07, 08, 11 and 13.
- Large dependency tree, version churn (majors are frequent): pin exact versions, upgrade deliberately.
- Atlaskit is built for Atlassian's own product assumptions; some packages may reference internal-only behavior. Prototype before committing: render one page of ADF and open one editor on a stub backend (spike, exit criterion for this ADR).
- We still must not copy UI wording or assets from Atlassian products; Atlaskit's bundled i18n strings must pass `scripts/check-naming.sh`-equivalent scanning for banned names.

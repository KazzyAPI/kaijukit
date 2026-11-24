# ADR 0002: Own the whole stack; kaijukit UI kit; data-model assumptions

Last reviewed: 2026-11-24 · status: accepted (supersedes the Atlaskit decision in 0001; its app-model and Kubernetes decisions stand)

## Decisions

1. **We rebuild everything, backend and frontend.** No runtime dependency on Atlassian packages. We ship our own component kit, `@kaijukit/ui`, and our own editor and renderer, `@kaijukit/editor`, named and branded kaijukit throughout (package names, CSS class prefixes `kk-`, `data-kk-*` attributes, token names `--kk-*`, docs).
2. **Parity target for the kit is behavior and capability, not code.** For each component in the reference design system's public documentation, `@kaijukit/ui` provides an equivalent: same role, same states, same keyboard and accessibility behavior, comparable props. Inventory and per-component status: `docs/parity/ui-kit.md`. Visual styling is our own (see UI parity requirement 3).
3. **Editor:** ProseMirror (+ Tiptap where useful) with a schema generated from the public ADF JSON schema, so content round-trips with the reference. Yjs/yrs for collaboration. This restores the original plan; the 0001 collab-protocol problem disappears.
4. **Data models are ours.** The reference's internal schema is not public and we do not try to match it. We match only what is visible on the wire: API paths, fields, types, enums, status codes, error shapes, ids, pagination, plus the body formats. Where the reference gives no visible signal, pick the simplest sound design (Postgres, 64-bit ids, JSONB for open-ended data) and note the assumption in the relevant spec. `docs/spec/confluence/backend/01-data-model.md` lists reference-visible fields; table layout is not a parity concern.
5. **App model, deployment:** unchanged from 0001. REST/OAuth/tokens/webhooks now, Connect-compatible layer later, Forge runtime out of scope, Kubernetes first.
6. **Atlassian packages as a reference only:** reading their public docs and published schemas (OpenAPI, ADF) is allowed by the clean-room policy. We do not copy their component code or strings. If a future spike shows an Atlaskit package would save substantial time, reopen with a new ADR (license evidence is in 0001).

## Consequences

- More frontend work (component kit, editor behaviors) than reusing Atlaskit; mitigated by the per-node editor test tables in F04 as acceptance criteria.
- No hidden dependency on Atlassian services or release cadence; naming policy covers all shipped strings.

# Clean-room policy

Last reviewed: 2026-11-24

## Allowed
- Public documentation, public OpenAPI specs, public developer docs.
- Observed behavior of a reference instance you are licensed to use (recorded as request/response pairs in `conformance/`).
- Public standards (OIDC, SAML, OpenAPI, JSON Schema).

## Dependencies
- Apache-2.0 (or otherwise compatible) packages from public registries may be used as dependencies after a license check recorded in `docs/decisions/`. Their license does not waive the rest of this policy (no marks, no copied product copy).

## Not allowed
- Decompiling, disassembling or deobfuscating any vendor binary or client bundle.
- Leaked, stolen or otherwise non-public source code, or anyone's notes derived from it.
- Reading vendor source you have access to under NDA or an employer relationship.
- Copying vendor UI assets, icons, copy text or documentation prose. Specs are written in our own words.
- Bypassing terms of service or rate limits on a reference instance.

## Rule of thumb
Every spec entry cites a public URL or a conformance capture. No citation, no entry.

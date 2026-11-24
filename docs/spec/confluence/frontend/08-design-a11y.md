# F08 Design system, accessibility, i18n, responsive, performance

Last reviewed: 2026-11-24

## Design system (ours)

- Tokens: color (light/dark themes, semantic roles: surface, text, border, brand, success/warn/danger/info, selection, highlight for comments), spacing 4px scale, radii, elevation, typography (system font stack; headings H1-H6 scale, body 14px with 1.43 line height as density target `[V]`, code mono), motion (reduce-motion respected), z-index scale. Tokens are CSS variables; themes selectable per user (light, dark, auto) and per-site accents.
- Component library (React + our own primitives): Button, IconButton, Dropdown/Menu, Select, Combobox/Picker (user, space, page, label), Dialog/Modal, Popover, Tooltip, Flag/Toast, Banner, Tabs, Tree, Table, Pagination, Avatar/AvatarGroup, Badge, Lozenge, Checkbox/Radio/Toggle, TextField/TextArea, DatePicker, EmptyState, Skeleton, Spinner, Progress, Breadcrumbs, Sidebar, Resizable panel, DropZone, FormField with validation. Each with states: default, hover, focus-visible, active, disabled, loading, error.
- Iconography: our own icon set; no copied glyphs.
- Layout density parity: sidebar width, top bar height (~56px `[V]`), content max width, editor toolbar height, comment panel width kept within +/-10% of reference measurements captured in `conformance/ui/layout/`.

## Accessibility
- WCAG 2.1 AA minimum. Landmarks (banner, navigation, main, complementary), skip link, focus-visible, logical focus order, focus trap + return in dialogs, `aria-live` for flags/status/presence, keyboard-only operation for tree, menus, pickers, editor toolbar, tables, drag-and-drop alternatives, color is never the only indicator, contrast >= 4.5:1, reduced motion, 200% zoom reflow, screen-reader labels for icon buttons. Automated axe checks in Playwright on every captured screen plus manual SR passes (VoiceOver, NVDA) on editor and tree before `done`.

## Internationalization
- All strings via message catalogs (ICU MessageFormat); our own wording. Locales: reference's list `[V]` (>= en, de, fr, es, pt-BR, it, nl, ja, ko, zh-CN, zh-TW, ru, pl, cs, hu, da, nb, sv, fi, tr). RTL layouts mirror (sidebar, toolbar, breadcrumbs, editor direction per block). Dates/times/numbers via `Intl` with user time zone and locale preferences. Pluralization rules. User-generated content language independent.

## Responsive and mobile web
- Breakpoints `[V]`: >=1200 desktop, 768-1199 tablet (sidebar collapsed by default, overlay), <768 mobile (top bar compresses to menu, search icon, create fab `[V]`, tree in drawer, tables scroll, layouts stack, comment panel -> bottom sheet, editor toolbar scrolls horizontally, touch targets >= 44px). Native app is out of scope; mobile web must support viewing, commenting, light editing.
- Print stylesheet; email-safe HTML for notifications.

## Performance budgets
- LCP < 2.5s on page view (warm cache) for a 1000-word page, TTI < 3.5s; route transitions < 300ms with skeletons; editor ready < 2s after route; tree of 10k nodes smooth (virtualized); search typeahead < 300ms server time; JS initial bundle budget `[ours]` < 300kB gz excluding editor chunk (lazy loaded); WASM core loaded lazily with streaming compile and cached.

## Frontend architecture constraints
- **Base:** Atlaskit packages (Apache-2.0) for components, tokens, editor, renderer and ADF schema, per [ADR 0001](../../../decisions/0001-frontend-and-app-model.md). Our own component library below is only for gaps Atlaskit does not cover. If the ADR is rejected, fall back to the own-library plan in this file.
- React + TypeScript + Vite; routing via a client router; data via typed client generated from our OpenAPI; server state cache (query library); editor in a separate lazy chunk; core WASM crates for CQL parse/validate, ADF validation/conversion, permission hints (UI affordance only; server authoritative).
- No business logic divergence: any ADF/storage/CQL logic used in UI comes from the shared Rust core.
- Error boundary per region; offline detection; feature flags per site setting for collaborative editing.

## Test strategy
- Playwright E2E with reference captures (screens, ARIA snapshots, scripted flows) in `conformance/ui/`; visual regression is structure/ARIA-based and layout-measure-based, not pixel-based (we intentionally do not clone the pixels); axe per screen; component tests for the library; ADF round-trip tests from backend corpus.

# Coding Conventions

**Analysis Date:** 2026-05-22

## Naming Patterns

**Files (Rust):**
- Domain modules under `src/openhuman/<domain>/` with per-file role: `mod.rs` (exports only), `ops.rs` (operations), `store.rs` (persistence), `types.rs` (domain types), `schemas.rs` (controller schemas + `handle_*`), `rpc.rs` (RPC handlers), `bus.rs` (event-bus subscribers).
- New functionality MUST live in a domain subdirectory. Do NOT add standalone `*.rs` at `src/openhuman/` root (`dev_paths.rs`, `util.rs` are grandfathered, not a template).

**Files (Frontend):**
- React components: PascalCase `Foo.tsx` co-located with `Foo.test.tsx`.
- Services as singletons under `app/src/services/` (camelCase, e.g. `coreRpcClient.ts`).
- Redux slices under `app/src/store/` (camelCase slice names).

**JSON-RPC methods:** `openhuman.<namespace>_<function>` (e.g. `openhuman.cron_create`).

**Event-bus native handlers:** method key `"<domain>.<verb>"`.

**Event-bus subscribers:** `<Purpose>Subscriber` with `name()` returning `"<domain>::<purpose>"`.

## Code Style

**Formatting:**
- Frontend: Prettier (run `pnpm format` / `pnpm format:check`).
- Rust: `cargo fmt` (also wired into `pnpm format`).

**Linting:**
- ESLint with `--cache` (`pnpm lint`).
- Husky pre-push hook runs `pnpm rust:check` (Tauri shell `cargo check`). Use `--no-verify` only for pre-existing breakage unrelated to your change; call it out in the PR body.

**Type-check:** `pnpm typecheck` (alias `pnpm compile`) → `tsc --noEmit` in `app/`.

## File Size

- Soft cap ~500 lines. Split growing modules. Keep `mod.rs` export-focused; operational code lives in sibling files.

## Rust Core Patterns

**RpcOutcome<T> contract** (see [`AGENTS.md`](../../AGENTS.md)):
- RPC controller handlers return `RpcOutcome<T>` so success payloads, structured errors, and audit metadata stay aligned across CLI + JSON-RPC + socket dispatch.

**Controller-only RPC exposure:**
- Expose features via the controller registry in each domain's `schemas.rs` (`schemas`, `all_controller_schemas`, `all_registered_controllers`, `handle_*`).
- Wire exports into `src/core/all.rs`.
- Do NOT add domain branches in `src/core/cli.rs` or `src/core/jsonrpc.rs`. Do NOT add domain logic to `src/core/`.

**Schema contract:**
- Shared types in `src/core/types.rs` / `src/core/mod.rs` (`ControllerSchema`, `FieldSchema`, `TypeSchema`).
- Per-domain `schemas.rs` re-exports `all_controller_schemas as all_<domain>_controller_schemas` and `all_registered_controllers as all_<domain>_registered_controllers` from `mod.rs`.

**Event bus** (`src/core/event_bus/`):
- Use module-level singleton API only: `init_global`, `publish_global`, `subscribe_global`, `register_native_global`, `request_native_global`. Never construct `EventBus` / `NativeRegistry` directly outside tests.
- Native request/response types: owned fields, `Arc`s, channels — not borrows. `Send + 'static`. Not `Serialize`.
- Domains in scope: `agent`, `memory`, `channel`, `cron`, `skill`, `tool`, `webhook`, `system`.
- `DomainEvent` is `#[non_exhaustive]`; extend the `domain()` match when adding variants.

**Adding events:** extend `DomainEvent` → update `domain()` → add subscribers in `<domain>/bus.rs` → register at startup → publish via `publish_global`.

**Adding native handlers:** define typed req/resp in the domain → register at startup keyed by `"<domain>.<verb>"` → callers use `request_native_global`.

**Skills runtime:** QuickJS/`rquickjs` removed. `src/openhuman/skills/` is metadata-only (`ops_create`, `ops_discover`, `ops_install`, `ops_parse`, `inject`, `schemas`, `types`). Do not reintroduce a JS skill runtime.

## Frontend Patterns

**No dynamic imports** in production `app/src` code:
- Static `import` / `import type` only.
- Forbidden: `import()`, `React.lazy(() => import(...))`, `await import(...)`.
- For heavy optional paths: static import + `try/catch` or runtime guard at the call site.
- Exceptions: Vitest harness (`*.test.ts`, `__tests__/`, `app/src/test/setup.ts`), ambient `typeof import('…')` in `.d.ts`, config files (e.g. `tailwind.config.js` JSDoc).

**Config gateway:**
- `app/src/utils/config.ts` is the ONLY place that reads `import.meta.env` / `VITE_*`. All other code reads from re-exports.

**Tauri environment guard:**
- Use `isTauri()` from `app/src/services/webviewAccountService.ts` or wrap `invoke(...)` in `try/catch`.
- Do NOT check `window.__TAURI__` directly — it's not present at module load and bypasses the wrapper contract.

**Core RPC bridge:**
- Use `invoke('core_rpc_relay', ...)` via `coreRpcClient` — avoids CORS preflight that raw `fetch()` would trigger.

**State management:**
- Prefer Redux Toolkit slices over ad-hoc `localStorage`. Exception: ephemeral UI state (e.g. upsell dismiss flags).
- Auth tokens live in the in-process core, NOT in `redux-persist`.

**Tailwind tokens:**
- Centralized in `app/tailwind.config.js` (ocean primary `#4A83DD`, sage/amber/coral semantics, Inter + Cabinet Grotesk + JetBrains Mono, custom radii/spacing/shadows). Do not invent ad-hoc tokens — extend the config.

## CEF Child Webviews

**No new JS injection** into `acct_*` provider webviews (`app/src-tauri/src/webview_accounts/`):
- Do NOT add new `.js` files under `webview_accounts/`.
- Do NOT extend `build_init_script` / `RUNTIME_JS`.
- Do NOT dispatch scripts via CDP `Page.addScriptToEvaluateOnNewDocument` / `Runtime.evaluate` for these webviews.
- New behavior goes in: CEF handlers (`on_navigation`, `on_new_window`, `LoadHandler::OnLoadStart`, `CefRequestHandler::*`), CDP from the scanner side (`*_scanner/` modules), Rust-side IPC hooks.
- Audit new Tauri plugins for default JS injection (e.g. `tauri-plugin-opener`'s `init-iife.js` — disable with `.open_js_links_on_click(false)`).
- Legacy injection for `gmail`, `linkedin`, `google-meet` is grandfathered but should shrink, not grow.

## Import Organization

**Frontend:** static `import` only (see above). Path aliases per `app/tsconfig.json` / Vite resolver.

**Rust:** standard `use` ordering; `cargo fmt` enforces.

## Error Handling

**Rust:** Return `RpcOutcome<T>` from controllers; structured error variants carry audit metadata. Domain logic uses `Result<T, E>` with domain-specific error types.

**Frontend:** Wrap Tauri `invoke` in `try/catch`. Surface failures via snackbars / Sentry (`Sentry.ErrorBoundary` at provider root).

## Logging

**Mandatory verbose diagnostics** on new/changed flows:
- Rust: `log` / `tracing` at `debug` / `trace`.
- Frontend: namespaced `debug` + dev-only detail.
- Stable grep prefixes: `[domain]`, `[rpc]`, `[ui-flow]`.
- Include correlation fields: request IDs, method names, entity IDs.
- Log entry/exit, branches, external calls, retries/timeouts, state transitions, errors.
- NEVER log secrets or full PII — redact.
- Changes lacking diagnostic logging are incomplete.

## Function & Module Design

**Functions:** single sharp responsibility (Unix style).

**Modules:** compose through clear boundaries; light `mod.rs`; behavior in sibling files.

**Exports:** domain `mod.rs` re-exports only public surface (`all_controller_schemas`, `all_registered_controllers`, public types).

## Documentation

- New/changed behavior ships with matching rustdoc / code comments.
- Update `AGENTS.md` or architecture docs (`gitbooks/developing/`) when rules or user-visible behavior change.
- Update `src/openhuman/about_app/` when adding/removing/renaming a user-facing feature.

## Git Workflow

- **Never write code on `main`.** Always: `git fetch upstream && git checkout -b <branch> upstream/main`.
- Issues and PRs filed against upstream **[tinyhumansai/openhuman](https://github.com/tinyhumansai/openhuman)** (not a fork).
- Templates: `.github/ISSUE_TEMPLATE/feature.md`, `.github/ISSUE_TEMPLATE/bug.md`, `.github/PULL_REQUEST_TEMPLATE.md`.
- PRs target `main`.
- Push branches to `origin` (the fork, `senamakel/openhuman`), NEVER to `upstream`. Treat `upstream` as fetch-only.
- Open PRs against `tinyhumansai/openhuman:main` with `--head senamakel:<branch>`.
- When asked to push or open a PR, resolve blockers and push — don't prompt. If pre-push hook fails on unrelated pre-existing breakage, push with `--no-verify` and call it out in the PR body.

## Pre-merge Checklist

For code changes:
- `pnpm format:check` (Prettier + `cargo fmt --check`).
- `pnpm lint`.
- `pnpm typecheck` in `app/`.
- `cargo check` for changed Rust crates (`Cargo.toml` and `app/src-tauri/Cargo.toml`).
- Vitest + relevant Rust tests passing.
- Coverage on changed lines ≥ 80% (see `TESTING.md`).

---

*Convention analysis: 2026-05-22*

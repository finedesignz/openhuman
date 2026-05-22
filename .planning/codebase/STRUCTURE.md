# Codebase Structure

**Analysis Date:** 2026-05-22

## Directory Layout

```
openhuman/
├── src/                          # Rust crate `openhuman` + `openhuman-core` bin
│   ├── main.rs                   # CLI entry (openhuman-core)
│   ├── bin/                      # slack-backfill, gmail-backfill-3d helpers
│   ├── core/                     # Transport: Axum/JSON-RPC/CLI/event bus
│   └── openhuman/                # Domain logic (one folder per domain)
├── app/                          # pnpm workspace `openhuman-app`
│   ├── src/                      # Vite + React UI
│   └── src-tauri/                # Tauri v2 desktop host (Rust)
├── tests/                        # Rust integration tests (json_rpc_e2e, etc.)
├── scripts/                      # Mock API, dotenv loader, debug runners
├── docs/                         # Deep internals (memory pipeline, sentry)
├── gitbooks/developing/          # Public contributor docs (authoritative)
├── packages/                     # Workspace packages
├── examples/                     # Example integrations
├── remotion/                     # Remotion video tooling
├── design-previews/              # Design artifacts
├── e2e/                          # docker-compose for Linux E2E on macOS
├── .planning/                    # GSD planning artifacts (this map lives here)
├── Cargo.toml                    # Root core crate manifest
├── package.json                  # Root (openhuman-repo, private, pnpm)
├── pnpm-workspace.yaml           # Workspace definition
├── AGENTS.md                     # RPC controller patterns, RpcOutcome contract
└── CLAUDE.md                     # Authoritative repo guide for agents
```

## Directory Purposes

**`src/core/`** — Transport only.
- Files: `all.rs` (controller registry), `all_tests.rs`, `auth.rs`, `autocomplete_cli_adapter.rs`, `cli.rs`, `cli_tests.rs`, `dispatch.rs`, `jsonrpc.rs`, `jsonrpc_cors_tests.rs`, `jsonrpc_tests.rs`, `legacy_aliases.rs`, `logging.rs`, `memory_cli.rs`, `mod.rs`, `observability.rs`, `rpc_log.rs`, `shutdown.rs`, `socketio.rs`, `types.rs`, `agent_cli.rs`.
- Subdirs: `event_bus/` (`bus.rs`, `events.rs`, `events_tests.rs`, `mod.rs`, `native_request.rs`, `native_request_tests.rs`, `subscriber.rs`, `testing.rs`, `tracing.rs`, `README.md`).

**`src/openhuman/`** — Domains. Each domain follows the convention:
- `mod.rs` — exports only, light
- `schemas.rs` — `ControllerSchema`s + `all_registered_controllers()`
- `rpc.rs` — `handle_*` JSON-RPC entry points returning `RpcOutcome<T>`
- `ops.rs` — domain operations (business logic)
- `store.rs` — persistence
- `types.rs` — domain types
- `bus.rs` (optional) — event bus subscribers (`<Purpose>Subscriber`)

**`app/src/`** — React UI.
**`app/src-tauri/src/`** — Tauri host modules.

## Domains under `src/openhuman/`

`about_app`, `accessibility`, `agent`, `agent_experience`, `agent_tool_policy`, `app_state`, `approval`, `audio_toolkit`, `autocomplete`, `billing`, `channels`, `composio`, `config`, `connectivity`, `context`, `cost`, `credentials`, `cron`, `desktop_companion`, `doctor`, `embeddings`, `encryption`, `health`, `heartbeat`, `http_host`, `inference`, `integrations`, `javascript`, `learning`, `mcp_client`, `mcp_clients`, `mcp_server`, `meet`, `meet_agent`, `memory`, `migration`, `migrations`, `notifications`, `overlay`, `people`, `prompt_injection`, `provider_surfaces`, `redirect_links`, `referral`, `routing`, `runtime_node`, `runtime_python`, `scheduler_gate`, `screen_intelligence`, `security`, `service`, `skills` (metadata-only — QuickJS runtime removed), `socket`, `subconscious`, `team`, `test_support`, `text_input`, `threads`, `todos`, `tokenjuice`, `tool_registry`, `tool_timeout`, `tools`, `tree_summarizer`, `update`, `vault`, `voice`, `wallet`, `webhooks`, `webview_accounts`, `webview_apis`, `webview_notifications`, `whatsapp_data`, `workspace`.

Grandfathered single-file modules at this level (do **not** add new ones): `dev_paths.rs`, `util.rs`.

### Inference domain (`src/openhuman/inference/`)

- Top level: `device.rs`, `model_context.rs`, `model_ids.rs`, `mod.rs`, `ops.rs`, `ops_tests.rs`, `parse.rs`, `paths.rs`, `presets.rs`, `presets_tests.rs`, `schemas.rs`, `schemas_tests.rs`, `sentiment.rs`, `types.rs`.
- Subdirs: `http/`, `local/`, `openai_oauth/`, `voice/`, `provider/`.
- **`provider/`** — pluggable LLM backends:
  - `traits.rs` — `InferenceProvider` trait (factory string grammar lives here)
  - `factory.rs` / `factory_test.rs` — parses `openhuman` | `ollama:<model>` | `<slug>:<model>` | `claude-code:<model>`
  - `openhuman_backend.rs`, `compatible*.rs` (OpenAI-compat — `compatible.rs`, `compatible_dump.rs`, `compatible_parse.rs`, `compatible_stream.rs`, `compatible_tests.rs`, `compatible_types.rs`)
  - `reliable.rs` / `reliable_tests.rs`, `router.rs` / `router_test.rs`
  - `billing_error.rs`, `config_rejection.rs`, `ops.rs`, `schemas.rs`, `temperature.rs`, `thread_context.rs`, `traits_tests.rs`
  - **`claude_code/`** (new on this branch — Phase 1 scaffold for Claude Code CLI provider): `auth.rs`, `driver.rs`, `event_mapper.rs`, `input_builder.rs`, `mod.rs`, `session_store.rs`, `stream_parser.rs`, `types.rs`, `version_check.rs`.

## Tauri shell modules (`app/src-tauri/src/`)

Top-level files: `lib.rs`, `main.rs`, `cef_preflight.rs`, `cef_profile.rs`, `companion_commands.rs`, `core_process.rs`, `core_process_tests.rs`, `core_rpc.rs`, `dictation_hotkeys.rs`, `file_logging.rs`, `mascot_native_window.rs`, `mcp_commands.rs`, `process_kill.rs`, `process_recovery.rs`, `window_state.rs`.

Submodules:
- `cdp/` — Chrome DevTools Protocol client
- `discord_scanner/`, `gmessages_scanner/`, `imessage_scanner/`, `meet_scanner/`, `slack_scanner/`, `telegram_scanner/`, `whatsapp_scanner/` — per-provider native scanners (CDP-driven; no JS injection)
- `fake_camera/`, `meet_audio/`, `meet_call/`, `meet_video/`, `screen_capture/` — media
- `native_notifications/`, `notification_settings/` — OS notification surface
- `webview_accounts/`, `webview_apis/` — child CEF webview infrastructure

## React UI (`app/src/`)

Top-level: `App.tsx`, `AppRoutes.tsx`, `App.css`, `index.css`, `index.html`, `main.tsx`, `polyfills.ts`, `SOUL.md`, `vite-env.d.ts`.

Subdirs:
- `__tests__/`, `assets/`, `chat/`, `components/`, `constants/`, `features/`, `hooks/`, `lib/` (includes `lib/mcp/`, `lib/ai/`), `mascot/`, `overlay/`, `pages/`, `providers/`, `services/`, `store/`, `styles/`, `test/`, `types/`, `utils/`.

### Redux store (`app/src/store/`)

`index.ts`, `hooks.ts`, `resetActions.ts`, `userScopedStorage.ts`, plus slices:
`accountsSlice.ts`, `agentProfileSlice.ts`, `channelConnectionsSlice.ts`, `chatRuntimeSlice.ts`, `companionSlice.ts`, `connectivitySlice.ts` (+ `connectivitySelectors.ts`), `coreModeSlice.ts`, `deepLinkAuthState.ts`, `localeSlice.ts`, `mascotSlice.ts`, `notificationSlice.ts`, `providerSurfaceSlice.ts`, `socketSlice.ts` (+ `socketSelectors.ts`), `themeSlice.ts`, `threadSlice.ts`. Tests under `__tests__/` and `*.test.ts` co-located.

### Services (`app/src/services/`)

Singletons including `apiClient`, `socketService`, `coreRpcClient`, `coreCommandClient`, `chatService`, `analytics`, `notificationService`, `webviewAccountService`, `daemonHealthService`, plus domain `api/*` clients.

## Key File Locations

**Entry Points:**
- `src/main.rs` — `openhuman-core` CLI binary
- `app/src-tauri/src/main.rs` — Tauri host entry
- `app/src/main.tsx` — React entry → `App.tsx`

**Configuration:**
- `.env.example`, `app/.env.example` — env templates
- `app/src/utils/config.ts` — centralized `VITE_*` reader (never read `import.meta.env` elsewhere)
- `src/openhuman/config/schema/types.rs` — Rust TOML config schema
- `src/openhuman/config/schema/load.rs` — env override loader

**Core Logic:**
- `src/core/all.rs` — controller registry wiring
- `src/core/jsonrpc.rs` — Axum router (`/`, `/health`, `/schema`, `/events`, `/events/webhooks`, `/rpc`, `/ws/dictation`, `/auth/telegram`, `/v1/*`)
- `src/core/event_bus/mod.rs` — singleton init + `publish_global` / `subscribe_global` / `register_native_global` / `request_native_global`
- `src/openhuman/inference/provider/factory.rs` — provider factory string grammar
- `src/openhuman/inference/provider/claude_code/driver.rs` — new Claude Code CLI provider driver

**Testing:**
- `tests/json_rpc_e2e.rs` — Rust JSON-RPC E2E
- `app/test/vitest.config.ts` — Vitest config
- `app/test/wdio.conf.ts` — WDIO E2E config
- `app/test/e2e/specs/*.spec.ts` — desktop E2E specs
- `scripts/mock-api-server.mjs`, `scripts/mock-api-core.mjs` — shared mock backend
- `scripts/test-rust-with-mock.sh` — cargo test wrapper

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (one concept per file)
- React components: `PascalCase.tsx`
- Slices: `<feature>Slice.ts`; selectors `<feature>Selectors.ts`
- Tests: co-located `*.test.ts(x)` (Vitest); Rust `mod_tests.rs` siblings
- E2E specs: `*.spec.ts` under `app/test/e2e/specs/`

**Directories:**
- Rust domain folders: `snake_case`
- React feature folders: `camelCase` or `PascalCase` matching dominant export

**JSON-RPC methods:** `openhuman.<namespace>_<function>` (e.g. `openhuman.cron_list`).

## Where to Add New Code

**New Rust domain:**
- Create `src/openhuman/<domain>/` with `mod.rs`, `schemas.rs`, `rpc.rs`, `ops.rs`, `types.rs`
- Export `all_controller_schemas as all_<domain>_controller_schemas` and `all_registered_controllers as all_<domain>_registered_controllers` from `mod.rs`
- Wire into `src/core/all.rs`
- Do **not** add to `src/core/cli.rs` or `src/core/jsonrpc.rs`

**New JSON-RPC method on existing domain:**
- Add `ControllerSchema` to `<domain>/schemas.rs`
- Add `handle_<method>` to `<domain>/rpc.rs` returning `RpcOutcome<T>`
- Include in `all_registered_controllers()`

**New inference provider:**
- Add module under `src/openhuman/inference/provider/<name>/`
- Implement the `InferenceProvider` trait from `traits.rs`
- Register in `src/openhuman/inference/provider/factory.rs` with a factory-string prefix

**New event bus event:**
- Add variant to `DomainEvent` in `src/core/event_bus/events.rs` (extend `domain()` match)
- Create `<domain>/bus.rs` with a `<Purpose>Subscriber` impl
- Register at startup; publish via `publish_global`

**New typed native request:**
- Define request/response types in the domain (owned, `Send + 'static`, not `Serialize`)
- Register at startup with `register_native_global("<domain>.<verb>", handler)`
- Callers use `request_native_global`

**New React screen:**
- Component under `app/src/pages/<Feature>/` or `app/src/features/<feature>/`
- Route added in `app/src/AppRoutes.tsx`
- State (if cross-screen) in `app/src/store/<feature>Slice.ts`
- Backend access via `coreRpcClient` (never raw `fetch`)

**New Tauri IPC command:**
- File under `app/src-tauri/src/<module>.rs`
- Register in `app/src-tauri/src/lib.rs` invoke handler
- Audit any plugin for JS injection before adding

**New tests:**
- Vitest: co-located `*.test.tsx` under `app/src/**`
- Rust unit: `mod_tests.rs` next to module
- Rust integration: `tests/<name>.rs`
- E2E: `app/test/e2e/specs/<name>.spec.ts` using helpers in `app/test/e2e/helpers/`

**Utilities:**
- TS shared helpers: `app/src/utils/`
- Rust shared types: `src/core/types.rs` (transport) or `src/openhuman/<domain>/types.rs` (domain)

## Special Directories

**`target/`:**
- Purpose: Rust build artifacts
- Generated: Yes · Committed: No

**`node_modules/`:**
- Purpose: pnpm install output
- Generated: Yes · Committed: No

**`app/src-tauri/vendor/tauri-cef/`:**
- Purpose: Vendored CEF-aware `tauri-cli` (required — stock CLI produces broken bundles)
- Generated: No · Committed: Yes

**`.planning/`:**
- Purpose: GSD planning artifacts (this codebase map, phase plans, etc.)
- Generated: By GSD commands · Committed: Yes

**`docs/`:**
- Purpose: Deep internal docs (memory pipeline excalidraws, Sentry, etc.)
- Generated: No · Committed: Yes

**`gitbooks/developing/`:**
- Purpose: Authoritative contributor docs — architecture, frontend, Tauri shell, agent harness, E2E testing, CEF, testing strategy, observability
- Generated: No · Committed: Yes

---

*Structure analysis: 2026-05-22*

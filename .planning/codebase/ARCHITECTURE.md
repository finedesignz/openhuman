<!-- refreshed: 2026-05-22 -->
# Architecture

**Analysis Date:** 2026-05-22

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                  Tauri Desktop Host (app/src-tauri)                  │
│   Window/IPC/lifecycle · CEF webviews · native scanners · hotkeys    │
│   `app/src-tauri/src/lib.rs` · `core_process.rs` · `core_rpc.rs`     │
└──────────────┬──────────────────────────────────────┬────────────────┘
               │ tauri::invoke (`core_rpc_relay`)     │ spawns in-process
               ▼                                       ▼
┌──────────────────────────────────┐   ┌──────────────────────────────────┐
│  React UI  (app/src)             │   │  Rust Core (in-process tokio)    │
│  Vite + React + Redux Toolkit    │   │  Axum HTTP server bound to       │
│  `App.tsx` provider chain        │◀──│  127.0.0.1:<port>; bearer auth   │
│  `services/coreRpcClient.ts`     │   │  via `OPENHUMAN_CORE_TOKEN`      │
└──────────────────────────────────┘   │  `src/core/jsonrpc.rs`           │
                                       └──────────────┬───────────────────┘
                                                      │
            ┌─────────────────────────────────────────┼─────────────────────────┐
            ▼                                         ▼                         ▼
┌──────────────────────────┐   ┌──────────────────────────────┐   ┌──────────────────────────┐
│ Controller Registry      │   │ Event Bus (singleton)        │   │ Domains                  │
│ `src/core/all.rs`        │   │ `src/core/event_bus/`        │   │ `src/openhuman/<dom>/`   │
│ RegisteredController +   │   │ DomainEvent pub/sub +        │   │ rpc.rs · ops.rs ·        │
│ per-domain `schemas.rs`  │   │ NativeRegistry req/resp      │   │ schemas.rs · store.rs    │
└──────────────────────────┘   └──────────────────────────────┘   └──────────────────────────┘
                                                      │
                                                      ▼
                                       ┌──────────────────────────────────┐
                                       │ Persistence / external services  │
                                       │ workspace dir, OpenAI-compat,    │
                                       │ Composio, OAuth, providers       │
                                       └──────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Tauri host | Window, OS IPC, CEF webviews, native scanners, spawns core | `app/src-tauri/src/lib.rs` |
| Core process handle | Lifecycle of in-process core tokio task; bearer mint; PID-safe restart | `app/src-tauri/src/core_process.rs` |
| Core RPC relay | Frontend `invoke('core_rpc_relay', …)` → HTTP to embedded server | `app/src-tauri/src/core_rpc.rs` |
| Axum JSON-RPC server | HTTP transport: REST + JSON-RPC + WS + OpenAI-compat | `src/core/jsonrpc.rs` |
| Controller registry | Declarative schemas + handler dispatch for every RPC method | `src/core/all.rs` |
| Event bus | Typed pub/sub + native req/resp singletons | `src/core/event_bus/` |
| Frontend RPC client | TS client over `core_rpc_relay` | `app/src/services/coreRpcClient.ts` |
| Redux store | UI state, persisted slices, hooks | `app/src/store/index.ts` |
| Inference provider trait | Pluggable LLM backends; factory string grammar | `src/openhuman/inference/provider/traits.rs` |

## Pattern Overview

**Overall:** In-process core with HTTP boundary. Tauri shell is delivery; Rust core is authoritative; React UI presents.

**Key Characteristics:**
- Single binary per desktop install — no sidecar (removed PR #1061). Core runs as a tokio task inside the Tauri host.
- HTTP-over-loopback boundary with per-launch hex bearer (`OPENHUMAN_CORE_TOKEN`) preserves a clean transport contract while avoiding process management.
- Controller registry is the only path features take to reach CLI + JSON-RPC; no manual branches in `src/core/cli.rs` / `src/core/jsonrpc.rs`.
- Domain code lives in `src/openhuman/<domain>/`; transport stays in `src/core/`.
- Event bus is the seam for cross-domain coupling (typed pub/sub + native typed request/response — no JSON in-process).

## Layers

**React UI (`app/src/`):**
- Purpose: Screens, navigation, presentation
- Location: `app/src/`
- Contains: Components, Redux slices, services, hooks
- Depends on: Tauri IPC (`@tauri-apps/api`), `coreRpcClient`, `socketService`
- Used by: end user via Tauri WebView

**Tauri shell (`app/src-tauri/`):**
- Purpose: Desktop host — windows, OS hooks, CEF webviews, native scanners
- Location: `app/src-tauri/src/`
- Contains: IPC commands, core lifecycle, per-provider CDP scanners
- Depends on: `openhuman-core` crate (linked in-process)
- Used by: UI via `invoke(...)`

**Core transport (`src/core/`):**
- Purpose: HTTP/JSON-RPC/CLI/socket transport, controller dispatch, event bus
- Location: `src/core/`
- Contains: Axum router, controller registry, event bus, socket.io, observability
- Depends on: domain modules under `src/openhuman/`
- Used by: Tauri shell (in-process), `openhuman-core` CLI

**Core domains (`src/openhuman/`):**
- Purpose: Business logic — agent, memory, channels, cron, integrations, inference, …
- Location: `src/openhuman/<domain>/`
- Contains: `mod.rs` (exports only), `rpc.rs`, `schemas.rs`, `ops.rs`, `store.rs`, `types.rs`
- Depends on: other domains via event bus, persistence layer
- Used by: controller registry (`src/core/all.rs`)

## Data Flow

### Primary Request Path (UI → Core RPC)

1. React component calls `coreRpcClient.invoke('openhuman.<ns>_<fn>', params)` (`app/src/services/coreRpcClient.ts`).
2. Client invokes Tauri command `core_rpc_relay` (`app/src-tauri/src/core_rpc.rs`) — chosen over `fetch` to bypass CORS preflight.
3. Tauri shell POSTs to `http://127.0.0.1:<port>/rpc` with bearer header from `OPENHUMAN_CORE_TOKEN`.
4. Axum handler in `src/core/jsonrpc.rs` (`rpc_handler`, line ~601) validates bearer and dispatches to the controller registry.
5. `src/core/all.rs` resolves method → `RegisteredController` → domain `handle_*` in `src/openhuman/<domain>/schemas.rs`.
6. Domain `rpc.rs` returns `RpcOutcome<T>`; JSON-RPC envelope is serialized back.

### Event Path (cross-domain)

1. Producer calls `publish_global(DomainEvent::…)` (`src/core/event_bus/bus.rs`).
2. Subscribers registered at boot (e.g. `cron/bus.rs`, `webhooks/bus.rs`, `channels/bus.rs`) receive on filtered broadcast channels.
3. For typed 1:1 dispatch, callers use `request_native_global("<domain>.<verb>", req)` against `NativeRegistry`.

### Realtime Socket Path

1. Server side: `src/core/socketio.rs` exposes Socket.IO; MCP transport lives in `src/openhuman/mcp_server/` and `src/openhuman/mcp_client/`.
2. UI side: `app/src/services/socketService.ts` connects; `SocketProvider` in `app/src/providers/` exposes context; `socketSlice` mirrors connection state in Redux.
3. Dual-socket contract: changes to realtime protocol must keep `socketService` and MCP transport aligned (see `gitbooks/developing/architecture.md`).

**State Management:**
- Redux Toolkit with redux-persist (allowlisted slices). Auth tokens are **not** persisted in redux — they live in the in-process core, fetched on boot via `fetchCoreAppSnapshot()`.

## Key Abstractions

**RegisteredController:**
- Purpose: Single source of truth for a JSON-RPC method (name, schema, handler)
- Examples: `src/openhuman/cron/schemas.rs`, `src/openhuman/agent/schemas.rs`
- Pattern: Domain `schemas.rs` exports `all_controller_schemas()` + `all_registered_controllers()`; wired into `src/core/all.rs`.

**DomainEvent:**
- Purpose: Typed cross-module pub/sub envelope
- Examples: `src/core/event_bus/events.rs`
- Pattern: `#[non_exhaustive]` enum with `domain()` matcher; subscribers filter by domain.

**NativeRegistry:**
- Purpose: Typed 1:1 request/response between domains without serialization
- Examples: `src/core/event_bus/native_request.rs`
- Pattern: Register by method string; payloads pass `Send + 'static` trait objects, channels, `Arc`s.

**InferenceProvider trait:**
- Purpose: Pluggable LLM backends (openhuman backend, OpenAI-compatible, Ollama, Claude Code CLI)
- Examples: `src/openhuman/inference/provider/traits.rs`
- Pattern: Factory string grammar parsed in `src/openhuman/inference/provider/factory.rs` — `openhuman` | `ollama:<model>` | `<slug>:<model>` | `claude-code:<model>` (new on this branch).

**Frontend Provider Chain:**
- Purpose: Composable React context hierarchy
- Examples: `app/src/App.tsx`
- Pattern: `Sentry.ErrorBoundary` → `Redux Provider` → `PersistGate` (`PersistRehydrationScreen`) → `BootCheckGate` → `CoreStateProvider` → `SocketProvider` → `ChatRuntimeProvider` → `HashRouter` → `CommandProvider` → `ServiceBlockingGate` → `AppShell`.

## Entry Points

**Tauri host:**
- Location: `app/src-tauri/src/main.rs` → `lib.rs`
- Triggers: OS launches `.app` / `.exe`
- Responsibilities: Build tauri::Builder, register IPC commands, spawn `CoreProcessHandle`, open windows

**Core CLI / server:**
- Location: `src/main.rs` (`openhuman-core` binary) — wraps `src/core/cli.rs`
- Triggers: Spawned in-process by Tauri (default) or run standalone for debug (`./target/debug/openhuman-core serve`)
- Responsibilities: Init logging, load config, start Axum server, controller dispatch

**HTTP routes (`src/core/jsonrpc.rs` ~line 596):**
- `/` — root
- `/health` — liveness
- `/schema` — controller schema dump
- `/events` — SSE event stream
- `/events/webhooks` — webhook SSE stream
- `/rpc` — JSON-RPC POST
- `/ws/dictation` — dictation WebSocket
- `/auth/telegram` — Telegram OAuth callback
- `/v1/*` — OpenAI-compatible REST surface (chat completions etc., served via `inference/provider/compatible*.rs`)

**Frontend:**
- Location: `app/src/main.tsx` → `App.tsx` → `AppRoutes.tsx` (HashRouter)
- Triggers: Tauri WebView load
- Responsibilities: Mount provider chain, drive routes (`/`, `/onboarding/*`, `/home`, `/human`, `/intelligence`, `/skills`, `/chat`, `/channels`, `/invites`, `/notifications`, `/rewards`, `/webhooks`, `/settings/*`).

## Architectural Constraints

- **Threading:** Single tokio runtime for the core (in-process inside Tauri). Axum on tokio. Frontend single-threaded JS.
- **Transport boundary:** HTTP loopback only; bearer required. Frontend must use `invoke('core_rpc_relay', …)`, never raw `fetch` (CORS preflight will fail).
- **Global state:** Event bus (`EventBus` / `NativeRegistry`) are singletons via module-level fns — never construct directly.
- **No new JS injection in CEF child webviews:** see `CLAUDE.md` — scraping/observability must run via CDP from the per-provider scanner module.
- **No dynamic imports in `app/src` production code** — static `import` / `import type` only.
- **Module placement:** New Rust functionality under `src/openhuman/<new_domain>/`; do not add new top-level `.rs` files under `src/openhuman/` (`dev_paths.rs`, `util.rs` are grandfathered).
- **File size:** prefer ≤ ~500 lines per file.

## Anti-Patterns

### Adding domain logic to `src/core/`

**What happens:** Branching in `src/core/cli.rs` / `src/core/jsonrpc.rs` to handle a new feature.
**Why it's wrong:** Bypasses the controller registry, duplicates dispatch, no auto-schema.
**Do this instead:** Add `src/openhuman/<domain>/schemas.rs` with `all_registered_controllers()` and wire into `src/core/all.rs`.

### Calling core over raw `fetch` from the UI

**What happens:** UI code uses `fetch('http://127.0.0.1:.../rpc')`.
**Why it's wrong:** Triggers CORS preflight; bearer token isn't safely accessible from JS.
**Do this instead:** Use `coreRpcClient` which calls `invoke('core_rpc_relay', …)` (`app/src/services/coreRpcClient.ts`).

### Injecting JS into provider CEF webviews

**What happens:** Adding a `Page.addScriptToEvaluateOnNewDocument` or new `.js` under `app/src-tauri/src/webview_accounts/`.
**Why it's wrong:** Expands scraping/attack surface inside third-party origins; explicitly banned in `CLAUDE.md`.
**Do this instead:** Implement behavior in per-provider CDP scanner under `app/src-tauri/src/<provider>_scanner/`.

### Constructing `EventBus` / `NativeRegistry` directly

**What happens:** `EventBus::new(...)` outside the singleton init.
**Why it's wrong:** Splits the bus; subscribers don't see events.
**Do this instead:** `init_global(capacity)` at boot; use `publish_global` / `subscribe_global` / `register_native_global` / `request_native_global`.

## Error Handling

**Strategy:** `Result<T, E>` end-to-end in Rust; controllers return `RpcOutcome<T>` (per `AGENTS.md`) which serializes to JSON-RPC error envelopes. Frontend wraps `invoke` and surfaces typed errors through services.

**Patterns:**
- Domain code returns `anyhow::Result` / domain-specific error enums.
- Controller `handle_*` maps to `RpcOutcome<T>`.
- Sentry boundary at the React root captures UI exceptions.

## Cross-Cutting Concerns

**Logging:** Rust uses `tracing` / `log` (`src/core/logging.rs`, `src/core/observability.rs`). File logging in Tauri shell at `app/src-tauri/src/file_logging.rs`. UI uses namespaced `debug`. Stable grep-friendly prefixes: `[domain]`, `[rpc]`, `[ui-flow]`.

**Validation:** Schema declared in domain `schemas.rs`; types in `src/core/types.rs` (`ControllerSchema`, `FieldSchema`, `TypeSchema`).

**Authentication:** Per-launch hex bearer in `OPENHUMAN_CORE_TOKEN` mints by `CoreProcessHandle`; verified in Axum middleware in `src/core/auth.rs`. User-facing auth lives in the core (`src/openhuman/credentials/`, `src/openhuman/security/`) — never persisted in redux.

---

*Architecture analysis: 2026-05-22*

# Plan — `claude-code` Provider for OpenHuman

**Owner:** jamie · **Status:** Locked v1 · **Branch:** `feat/claude-code-provider`

## 1. Goal

Add `claude-code` as a selectable LLM provider in OpenHuman that drives Anthropic's `claude` CLI (`--output-format stream-json --verbose --print --resume`) instead of calling the Anthropic HTTP API directly. Existing API providers stay. Native OpenHuman tools remain Rust-side and are exposed to the CLI over MCP so CC can call them.

Reference implementation: `C:\Users\artic\GitHub\opencode` — `packages/opencode/src/provider/claude-code/`.

## 2. Non-goals (v1)

- Subscription/OAuth auth (Claude Pro/Max) — v1 passes through `~/.claude/.credentials.json` if the user has run `claude login` (CLI handles refresh). v1.1 adds **detection + UI** (auth_status RPC + settings card surfacing). In-app OAuth flow still deferred to v2.
- Exposing **write** tools (memory mutation, channel send, etc.) via MCP — defer to v1.1 after threat model.
- Co-enabling CC's built-in tools (`Bash`/`Read`/`Edit`) — disabled in v1 via `--disallowedTools`.
- Cost accounting wired into `cost.rs` — defer to v1.1.
- Process pool / cold-spawn optimization — defer to v2 if needed.

## 3. Architecture (confirmed via Backend Architect review)

```
Frontend  ──invoke──>  Tauri shell  ──HTTP+bearer──>  openhuman-core (Axum :7788)
                                                       │
                                                       ├─ /rpc        (existing JSON-RPC)
                                                       └─ /mcp        (NEW — MCP server, SSE)
                                                                ▲
                                                                │  mcp__openhuman__*
                                                                │
   ChatRequest ──Provider::chat──> ClaudeCodeProvider ──spawn──> `claude --print
                                                                  --output-format stream-json
                                                                  --verbose --resume <uuid>
                                                                  --mcp-config <tmp.json>
                                                                  --disallowedTools <CC builtins>`
                                                                ▲       │
                                                       SSE+bearer       │ stdout JSONL
                                                                        ▼
                                                                 stream_parser ─→ event_mapper
                                                                                       │
                                                                                       ▼
                                                                                 ProviderDelta stream
                                                                                 → harness turn loop
```

**Key files (existing, do not invent):**
- `src/openhuman/inference/provider/traits.rs` — `Provider` trait, `ProviderDelta`, `ToolsPayload`, `ChatRequest`.
- `src/openhuman/inference/provider/factory.rs` — `create_chat_provider_from_string(role, provider, config)`. String-grammar dispatch.
- `src/openhuman/inference/provider/openhuman_backend.rs` — reference impl with auth.
- `src/openhuman/inference/provider/compatible.rs` — reference impl with streaming + Anthropic-style auth.
- `src/openhuman/config/schema/cloud_providers.rs` — `CloudProviderType`, `AuthStyle`.
- `src/core/` — Axum server, bearer auth middleware, existing `/rpc` route.

## 4. Module layout

### 4.1 Provider

```
src/openhuman/inference/provider/claude_code/
  mod.rs              — pub struct ClaudeCodeProvider; impl Provider for ...
  driver.rs           — process spawn, stdin/stdout/stderr piping, kill-on-drop,
                        tokio::sync::Semaphore(4) concurrency cap
  stream_parser.rs    — line-buffered JSONL → ClaudeCodeEvent
  event_mapper.rs     — ClaudeCodeEvent → ProviderDelta + tool-call accumulator
  session_store.rs    — ThreadId ↔ CC session UUID, persisted under config dir
  input_builder.rs    — ChatRequest → CLI argv + stdin payload
  mcp_config.rs       — generate per-launch mcp-config JSON (bearer + url),
                        write to temp, delete on drop
  version_check.rs    — `claude --version` parse + MIN_VERSION gate
  auth.rs             — API key resolution: env > config > ~/.claude/.credentials.json
  schemas.rs          — serde types for CC's stream-json envelope
  types.rs            — internal types
  tests/
    fixtures/         — canned JSONL transcripts pulled from opencode fork's test fixtures
    parser.rs         — golden tests on each fixture
    mapper.rs         — event→delta correctness
    driver.rs         — spawn happy-path + version-fail + missing-binary
```

### 4.2 MCP server (sibling, not under provider)

```
src/openhuman/mcp_server/
  mod.rs              — Axum sub-router mounted at /mcp on core HTTP
  transport.rs        — SSE transport (MCP HTTP server protocol)
  tool_registry.rs    — bridge to existing tool dispatch
  schemas.rs          — MCP wire types
  bus.rs              — EventBus subscriber for tool-result fan-out
  tests/
```

Wire mount in `src/core/all.rs` next to JSON-RPC route. Reuses existing bearer-auth middleware — **no new auth surface**.

### 4.3 Config

Add to `src/openhuman/config/schema/cloud_providers.rs`:
- `CloudProviderType::ClaudeCode`
- Fields: `binary_path: Option<PathBuf>`, `min_version: String`, `disallowed_builtins: Vec<String>` (defaults to all of CC's built-in tool names).

### 4.4 RPC additions

New controller methods (per AGENTS.md `RpcOutcome<T>` contract, exposed via registry):
- `openhuman.claude_code_status` → `{ installed, version, path, min_satisfied, auth_state, last_error }`
- `openhuman.claude_code_check_version` — re-probe `claude --version`
- `openhuman.claude_code_set_auth` — store API key in credentials domain
- Extend `openhuman.providers_list` to surface CC entry with `requires_external_binary: true`

Per layout rule, these live in `src/openhuman/inference/rpc.rs` extension (or new `inference/claude_code_rpc.rs`).

### 4.5 Frontend

Files under `app/src/`:
- `app/src/components/settings/ProviderSettings/ClaudeCodeSection.tsx` — install status, install instructions, API key input, version display.
- `app/src/components/settings/ProviderSettings/index.tsx` — add picker entry.
- `app/src/services/api/claudeCode.ts` — thin RPC wrappers.
- `app/src/store/slices/claudeCodeSlice.ts` — status state.

## 5. Provider dispatch grammar

`factory.rs::create_chat_provider_from_string`:
- New arm matches `"claude-code:<model>[@<temp>]"` (e.g. `claude-code:sonnet-4-5`, `claude-code:opus-4-7@0.7`).
- Model string passed verbatim to `--model`.
- Temperature → input payload (CC stream-json supports it in the input message).

Existing `provider_for_role` reading `chat_provider`, `agentic_provider`, etc., now resolves CC for any role.

## 6. Tool exposure via MCP

**v1 surface (read-only safe subset)** — to be confirmed once we read the existing tool registry:
- `memory_search`, `memory_get`
- `threads_list`, `threads_get`, `threads_messages`
- `channels_list`, `channels_messages_read`
- `people_search`, `people_get`
- `webhooks_list`

CC auto-prefixes MCP tools → CC sees them as `mcp__openhuman__memory_search` etc. **No collision risk** with CC built-ins.

CC built-ins (`Bash`, `Read`, `Write`, `Edit`, `Grep`, `Glob`, `WebFetch`, `WebSearch`, `Task`, `TodoWrite`, etc.) disabled via `--disallowedTools` for v1.

## 7. Auth (v1)

`auth.rs` resolution order:
1. `ChatRequest`/Config explicit key (per-thread/per-agent override)
2. `ANTHROPIC_API_KEY` env
3. `~/.claude/.credentials.json` (read-only — never write it; if present, set `ANTHROPIC_API_KEY` in spawned process env)
4. None → `claude_code_status.auth_state = "missing"`, provider returns clear error on `chat()`

API key set per-process via env var on spawn (`Command::env`), not as CLI arg (would leak in process listings).

## 8. Concurrency & lifecycle

- One CC process per turn (`--print` exits after assistant response). Reuse session UUID across turns via `--resume`.
- Global `Semaphore(4)` in `driver.rs` to cap concurrent processes.
- `Child` wrapped in a guard that calls `kill_on_drop(true)` + waits for exit; abort on harness interrupt.
- Hard timeout: 5 min per turn (configurable). Surface as `ProviderError::Timeout`.

## 9. Risks / open questions

| # | Risk | Mitigation |
|---|------|------------|
| R1 | CC stream-json schema drift between versions | Pin `MIN_VERSION` (initially `2.0.0`); `version_check` blocks startup with clear error. Re-test on every CC release. |
| R2 | Windows `claude.cmd` shim | `driver.rs` uses `where claude` resolution + spawns via `cmd /c` on Windows when target is `.cmd`. |
| R3 | `OPENHUMAN_CORE_TOKEN` rotates per launch | mcp-config JSON regenerated each session, written to tempfile, deleted on drop. Never cached. |
| R4 | CC built-ins re-enabled accidentally | v1 hard-codes `--disallowedTools` list; flag in config but undocumented until threat model. |
| R5 | Cost data lost (no `cost.rs` wiring) | v1.1. v1 logs `result.total_cost_usd` to debug log. |
| R6 | MCP server perf under tool spam | SSE on same Axum runtime — same backpressure story as `/rpc`. Add semaphore on tool-dispatch handler if it becomes a hotspot. |
| R7 | Subscription users without API key can't use v1 | Clear UX in settings: "v1 requires API key; subscription support coming." |

## 10. Phases & checkpoints

### Phase 1 — Skeleton + version check (1–2 days)
- Create branch `feat/claude-code-provider` off `upstream/main`.
- Add `CloudProviderType::ClaudeCode` config variant.
- Scaffold `claude_code/` module with `version_check.rs`, `auth.rs`, `types.rs`, `schemas.rs`, `mod.rs` (Provider impl returning `not_implemented` for `chat`).
- Add `claude_code_status` + `claude_code_check_version` RPC.
- Frontend: minimal settings panel showing install status only.
- Unit tests: version parsing, auth resolution.
- **Checkpoint**: settings panel shows `installed: true/false`, version, path on real Windows install.

### Phase 2 — Driver + stream parsing (2–3 days)
- `input_builder.rs`, `driver.rs` (spawn, kill-on-drop, semaphore), `stream_parser.rs`, `event_mapper.rs`, `session_store.rs`.
- Pull JSONL fixtures from opencode `packages/opencode/test/fixtures/claude-code-stream/`. Re-license headers if needed.
- Unit tests against fixtures: every event type maps to correct `ProviderDelta`.
- **Skip MCP for now**: spawn CC with `--disallowedTools <all>` and no MCP — just verify text streaming round-trip.
- Wire into `factory.rs` grammar.
- **Checkpoint**: pick provider in dev settings → run a turn → text streams back correctly. Multi-turn `--resume` works.

### Phase 3 — MCP server (2–3 days)
- `src/openhuman/mcp_server/` scaffold. Mount `/mcp` SSE route under existing auth.
- Expose v1 read-only tool subset via `tool_registry.rs`.
- `mcp_config.rs` generates per-launch JSON, driver passes `--mcp-config` + `--strict-mcp-config`.
- Integration test: spawn CC, ask "list my threads", verify tool call lands and result returns.
- **Checkpoint**: end-to-end roundtrip — CC calls `mcp__openhuman__threads_list`, gets result, continues turn.

### Phase 4 — Frontend polish + docs (1 day)
- Settings UI: install instructions per-OS, API key entry, "test connection" button.
- Per-role override UI if existing provider-selection UI supports it.
- Add docs entry in `gitbooks/developing/` covering the provider.
- Update `CLAUDE.md` if anything contract-changing landed (e.g. new `/mcp` route).

### Phase 5 — E2E + ship (1–2 days)
- E2E spec: configure CC provider, send a message, verify response.
- Rust integration test exercising `Provider::chat` against a mocked `claude` binary (`scripts/test-rust-with-mock.sh` harness extension).
- Coverage ≥ 80% on changed lines (merge gate).
- PR to `tinyhumansai/openhuman:main` from `senamakel:feat/claude-code-provider`.

**Total estimate:** 7–11 days of focused work.

## 11. Testing strategy

- **Unit (Vitest)** — frontend slice + components.
- **Unit (cargo)** — parser, mapper, auth, version check (all against fixtures, no real CC binary).
- **Rust integration** — driver against mocked binary that emits canned JSONL on stdin → stdout.
- **E2E (WDIO)** — happy path with CC mocked at the binary level via `OPENHUMAN_CLAUDE_BINARY` env override.

## 12. Rollout

- Behind a settings toggle (defaults to off) for first release. No auto-selection.
- Document beta status in settings panel until v1.1 (cost wiring + write tools) lands.

## 13. Locked decisions

1. **MIN_VERSION**: `2.0.0`. `version_check.rs` blocks startup below this.
2. **Read-only MCP tool subset (v1)**: `memory_search`, `memory_get`, `threads_list`, `threads_get`, `threads_messages`, `channels_list`, `channels_messages_read`, `people_search`, `people_get`, `webhooks_list`. Exposed as `mcp__openhuman__<name>`. Write tools deferred to v1.1.
3. **Per-role provider selection**: CC selectable independently for `chat`, `agentic`, `reasoning` roles via factory string grammar. No single global toggle.
4. **UI branding**: "Claude Code CLI" in all settings copy, provider picker labels, and status panel headings.
5. **Subscription detection (v1.1)**: Separate `openhuman.claude_code_auth_status` RPC (pure FS, no CLI spawn). Reads `~/.claude/.credentials.json` tolerantly — returns `subscription | api_key_env | none` with optional `account_email` + `expires_at`. Token never round-trips through RPC. Sign-out delegated to `claude logout` (no in-app file deletion to avoid half-state).

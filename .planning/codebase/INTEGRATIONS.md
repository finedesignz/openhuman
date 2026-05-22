# External Integrations

**Analysis Date:** 2026-05-22

## AI / LLM Providers

**Inference providers** (`src/openhuman/inference/provider/`):
- **Anthropic Claude Code CLI** — `src/openhuman/inference/provider/claude_code/` (newly landed, PR scaffolded Phase 1)
  - Modules: `mod.rs`, `driver.rs`, `stream_parser.rs`, `event_mapper.rs`, `input_builder.rs`, `session_store.rs`, `auth.rs`, `types.rs`, `version_check.rs`
  - Drives the Claude Code CLI as a subprocess; streams events back through the provider trait
- **OpenAI-compatible** — `compatible.rs`, `compatible_parse.rs`, `compatible_stream.rs`, `compatible_types.rs`, `compatible_dump.rs` — generic OpenAI-protocol client (works with OpenAI, Groq, local LM Studio, OpenRouter, etc.)
- **OpenHuman backend** — `openhuman_backend.rs` — hosted inference via OpenHuman's own backend
- **Local inference** — `src/openhuman/inference/local/` including `lm_studio.rs`
- **Router / factory** — `router.rs`, `factory.rs`, `reliable.rs` (retry wrapper), `temperature.rs`, `thread_context.rs`, `traits.rs`

**OpenAI OAuth** — `src/openhuman/inference/openai_oauth/` (`mod.rs`, `flow.rs`, `store.rs`, `config.rs`)
- Codex/ChatGPT OAuth via `motosan-ai-oauth` 0.2 (codex feature)

**Voice/Transcription:**
- `whisper-rs` 0.16 (local, on-device; Metal on macOS)
- Cloud transcribe fallback: `src/openhuman/inference/voice/cloud_transcribe.rs`

## MCP (Model Context Protocol)

**MCP server** (we expose) — `src/openhuman/mcp_server/`:
- `mod.rs`, `protocol.rs`, `session.rs`, `stdio.rs`, `tools.rs`
- Transport: stdio JSON-RPC
- Tauri-side bridge: `app/src-tauri/src/mcp_commands.rs`

**MCP clients** (we consume) — `src/openhuman/mcp_client/` and `src/openhuman/mcp_clients/`

**Frontend MCP transport** — `app/src/lib/mcp/`: JSON-RPC over Socket.IO

## Composio Aggregator

`src/openhuman/composio/` — unified integration layer for SaaS tools (Slack, Gmail, GoHighLevel, Google Calendar, etc.) via Composio's action API.
- `client.rs` — HTTP client
- `action_tool.rs` — agent tool exposure
- `auth_retry.rs` — OAuth token refresh
- `execute_dispatch.rs`, `execute_prepare.rs` — action execution
- `googlecalendar_args.rs` — Google Calendar argument shaping
- `trigger_history.rs` — webhook trigger log
- `periodic.rs` — periodic sync
- `error_mapping.rs` — surfaces Gmail scope errors as permissions (per recent fix #2414)
- `providers/` — per-Composio-provider adapters

## Channel Providers (messaging)

`src/openhuman/channels/providers/` — Rust-side channel adapters:
- **Slack** — `slack.rs` (helper binary `src/bin/slack_backfill.rs`)
- **Telegram** — `telegram/` (directory)
- **Discord** — `discord/` (directory)
- **WhatsApp** — `whatsapp.rs`, `whatsapp_web.rs` (via `whatsapp-rust` 0.5, feature-gated)
- **iMessage** — `imessage.rs` (reads `~/Library/Messages/chat.db` on macOS)
- **Matrix** — `matrix.rs` (via `matrix-sdk` 0.16, feature-gated)
- **Mattermost** — `mattermost.rs`
- **Signal** — `signal.rs`
- **IRC** — `irc.rs`
- **DingTalk** — `dingtalk.rs`
- **Lark** — `lark.rs`
- **LINQ** — `linq.rs`
- **QQ** — `qq.rs`
- **Email** — `email_channel.rs` (SMTP via `lettre`, IMAP via `async-imap`)
- **Web** — `web.rs` (web channel widget)
- **Presentation** — `presentation.rs`

## Embedded Provider Webviews (CEF, Tauri shell)

`app/src-tauri/src/*_scanner/` — per-provider CEF webview scrapers driven via Chrome DevTools Protocol (no JS injection in migrated providers):
- `discord_scanner/` — Discord web client
- `gmessages_scanner/` — Google Messages web
- `imessage_scanner/` — iMessage (macOS native chat.db scanner)
- `meet_scanner/` — Google Meet
- `slack_scanner/` — Slack web
- `telegram_scanner/` — Telegram web (`web.telegram.org`)
- `whatsapp_scanner/` — WhatsApp Web

**Meet stack:**
- `meet_audio/` — audio capture for Meet bot
- `meet_call/` — call orchestration; uses `resvg` + `tiny-skia` for fake-camera mascot rendering
- `meet_video/` — video pipeline
- `fake_camera/` — `--use-file-for-fake-video-capture` Y4M frame generation

**Webview accounts framework:**
- `app/src-tauri/src/webview_accounts/` — multi-account CEF profile management
- `app/src-tauri/src/webview_apis/` — JSON-RPC bridge from core → live webview connectors via CDP
- Frontend service: `app/src/services/webviewAccountService.ts`

**Legacy JS injection (grandfathered, must shrink):**
- Gmail, LinkedIn, Google Meet recipe files + `runtime.js` bridge
- New webview JS injection is **forbidden** by repo policy (CLAUDE.md)

## Domain Integrations (`src/openhuman/integrations/`)

Per-domain external API clients:
- **Apify** — `apify.rs` (web scraping platform)
- **Google Places** — `google_places.rs` (Places API)
- **SearXNG** — `searxng.rs` (federated search)
- **Seltz** — `seltz.rs`
- **Stock Prices** — `stock_prices.rs`
- **TinyFish** — `tinyfish.rs`
- **Twilio** — `twilio.rs` (SMS / voice)
- Generic client + parallel-fan-out: `client.rs`, `parallel.rs`, `types.rs`

## Data Storage

**Local databases:**
- SQLite via `rusqlite` 0.37 (bundled) — primary local store
- Postgres via `postgres` 0.19 — test infra / dev tooling only
- iMessage `chat.db` — read-only on macOS

**File storage:**
- Workspace dir: `~/.openhuman/` (override via `OPENHUMAN_WORKSPACE`)
- Staging: `~/.openhuman-staging/` (with `OPENHUMAN_APP_ENV=staging`)
- Path resolution: `src/openhuman/dev_paths.rs`

**Vault / Credentials:**
- `src/openhuman/vault/` — credential store
- `src/openhuman/credentials/` — credential domain logic
- Encryption: `src/openhuman/encryption/` (aes-gcm, chacha20poly1305, argon2)

**Memory / Embeddings:**
- `src/openhuman/memory/` — memory tree + ingest pipeline
- `src/openhuman/embeddings/` — embedding generation

## Authentication & Identity

- **OAuth flows** — per-provider via Composio (`src/openhuman/composio/auth_retry.rs`) and direct (OpenAI Codex via `motosan-ai-oauth`)
- **Deep-link OAuth callbacks** — `app/src-tauri/src/lib.rs` via `tauri-plugin-deep-link` + `tauri-plugin-single-instance` (deep-link feature forwards second-launch payloads to primary instance)
- **Frontend slice** — `app/src/store/deepLinkAuth/`
- **Wallet identity** — `ethers-core` + `ethers-signers` 2.0.14 (`src/openhuman/wallet/`)
- **Recovery phrase / BIP39** — `@scure/bip32`, `@scure/bip39`, `@noble/curves`, `@noble/hashes`, `@noble/secp256k1` (frontend)
- **Per-launch RPC bearer** — `OPENHUMAN_CORE_TOKEN` (hex token gating HTTP RPC at `127.0.0.1:<port>/rpc`)

## Realtime / Transport

**Socket.IO:**
- Server: `socketioxide` 0.15 (Rust core)
- Client: `socket.io-client` 4.8.3 (frontend)
- Frontend service: `app/src/services/socketService.ts`
- Slice: `app/src/store/socket/`
- Architecture: dual-socket (see `gitbooks/developing/architecture.md`)

**JSON-RPC over HTTP:**
- `axum` 0.8 server in core
- Frontend client: `app/src/services/coreRpcClient.ts` + `coreCommandClient.ts`
- Tauri IPC bridge: `core_rpc_relay` command (avoids CORS preflight)

**Chrome DevTools Protocol (CDP):**
- `tokio-tungstenite` 0.24 — WebSocket client to CEF `--remote-debugging-port=9222`
- Used for: WhatsApp/Telegram/Slack/Discord scrapers, Gmail connector, IndexedDB reads, Network/DOMSnapshot
- Module: `app/src-tauri/src/cdp/`

## Monitoring & Observability

**Sentry** (three separate projects):
- Frontend: `@sentry/react` ^10.38.0 (Vite plugin uploads sourcemaps)
- Rust core: `sentry` 0.47.0 — DSN via env
- Tauri shell: `sentry` 0.47.0 — DSN baked at compile via `option_env!("OPENHUMAN_TAURI_SENTRY_DSN")` in `app/src-tauri/src/lib.rs::run()`, env-overridable at runtime

**OpenTelemetry:**
- `opentelemetry` 0.32 + `opentelemetry_sdk` 0.32 + `opentelemetry-otlp` 0.32
- Traces + metrics via OTLP HTTP-proto

**Prometheus:**
- `prometheus` 0.14 metrics in core

**Logging:**
- Rust core: `tracing` + `tracing-subscriber` + `tracing-appender` (file rotation)
- Tauri shell: `log` + `env_logger`; file logging in `app/src-tauri/src/file_logging.rs`
- Frontend: namespaced `debug` 4.4.3

**Health / Diagnostics:**
- `src/openhuman/health/` — health checks
- `src/openhuman/heartbeat/` — heartbeat
- `src/openhuman/doctor/` — diagnostic CLI
- `src/openhuman/connectivity/` — connectivity probes
- Daemon health service: `app/src/services/daemonHealthService.ts`

## CI/CD & Deployment

**CI:**
- GitHub Actions
- Coverage gate: `.github/workflows/coverage.yml` (diff-cover ≥80% on changed lines)
- E2E gates per-flow (WDIO + tauri-driver on Linux, Appium Mac2 on macOS)

**Auto-update:**
- `tauri-plugin-updater` — Tauri app bundle updater
- Core has its own updater (`src/openhuman/update/`)
- Both must update in lockstep for new RPC methods

## Webhooks & Triggers

**Incoming:**
- `src/openhuman/webhooks/` — webhook receiver domain
- Frontend route: `/settings/webhooks-triggers`
- Composio triggers logged via `src/openhuman/composio/trigger_history.rs`

**Cron:**
- `src/openhuman/cron/` — cron domain
- Crate: `cron` 0.12
- Event bus integration: `src/openhuman/cron/bus.rs` (`CronDeliverySubscriber`)

## Notifications

- Rust core: `src/openhuman/notifications/` + `src/openhuman/webview_notifications/`
- Native:
  - macOS: `mac-notification-sys` 0.6 + `objc2-user-notifications` 0.3.2
  - Linux: `notify-rust` 4 (dbus)
  - Windows: via `tauri-plugin-notification` (vendored at `app/src-tauri/vendor/tauri-plugin-notification`)
- Web Notification intercept in CEF webviews: custom fork at `vendor/tauri-cef` patches `window.Notification` and `ServiceWorkerRegistration.prototype.showNotification`
- Tauri commands: `app/src-tauri/src/native_notifications/`, `app/src-tauri/src/notification_settings/`

## Update Channels / Distribution

- macOS: `.app` + `.dmg` bundles
- Windows: `.exe` / `.msi`
- Linux: `.AppImage` / `.deb`
- All built via vendored CEF-aware `tauri-cli` (`app/src-tauri/vendor/tauri-cef/crates/tauri-cli`)

## Environment Variables (key)

**Rust core:**
- `OPENHUMAN_CORE_TOKEN` — per-launch RPC bearer (hex)
- `OPENHUMAN_WORKSPACE` — override workspace dir (used by E2E)
- `OPENHUMAN_APP_ENV` — `staging` switches default workspace path
- `OPENHUMAN_CORE_REUSE_EXISTING=1` — attach to external `openhuman-core` instead of spawning
- `OPENHUMAN_SERVICE_MOCK=1` — E2E mock mode

**Tauri shell:**
- `OPENHUMAN_TAURI_SENTRY_DSN` — shell Sentry DSN (compile-time or runtime)
- `CEF_PATH` — CEF runtime cache dir
- `APPLE_SIGNING_IDENTITY` — macOS codesign identity

**Frontend (`VITE_*`):**
- Core RPC URL, backend URL, Sentry DSN, dev helpers (see `app/.env.example`)

**Secrets policy:** Per CLAUDE.md, the only env vars that should appear on MCP-hosted apps are the four gateway-pair vars — but this is **not** how OpenHuman itself authenticates (OpenHuman uses Composio + direct OAuth via its core, not the MCP gateway pair). The gateway-pair rule applies to other repos under the user's account, not this one.

---

*Integration audit: 2026-05-22*

# Technology Stack

**Analysis Date:** 2026-05-22

## Languages

**Primary:**
- Rust (edition 2021) - Core domain logic + RPC server (`src/`), Tauri shell (`app/src-tauri/`)
- TypeScript ~5.8.3 - React frontend (`app/src/`)

**Secondary:**
- JavaScript / Node ESM - Build scripts, mock API server (`scripts/*.mjs`)
- Bash - Dev/test orchestration scripts (`scripts/`, `app/scripts/`)
- PowerShell - Windows installer tests (`scripts/tests/*.ps1`)

## Runtime

**Desktop runtime:**
- Tauri v2.10 with **CEF (Chromium Embedded Framework) v146.4.1** — only supported runtime (not Wry). Vendored fork at `app/src-tauri/vendor/tauri-cef/`.
- Rust core runs **in-process** as a tokio task inside the Tauri host (no sidecar since PR #1061). JSON-RPC at `http://127.0.0.1:<port>/rpc`, bearer auth via `OPENHUMAN_CORE_TOKEN`.

**Node:**
- Required: Node `>=24.0.0` (see `app/package.json` engines)
- Used for: Vite dev server, build pipeline, Vitest, WDIO, scripts

**Package Manager:**
- pnpm 10.10.0 (pinned via `packageManager` field in root `package.json`)
- Workspace: root is `openhuman-repo` (private); `app/` is `openhuman-app`
- Cargo: workspace-style with two manifests — root `Cargo.toml` (core) and `app/src-tauri/Cargo.toml` (shell)
- Lockfiles: `pnpm-lock.yaml` (committed), `Cargo.lock` (committed)

**Platform support:**
- Windows, macOS, Linux desktop **only**. No Android/iOS branches.

## Frameworks

**Frontend Core:**
- React 19.1.0
- React DOM 19.1.0
- React Router DOM 7.13.0 (HashRouter)
- Redux Toolkit 2.11.2 + React-Redux 9.2.0 + redux-persist 6.0.0 + redux-logger 3.0.6
- Socket.IO Client 4.8.3
- Zod 4.3.6 (schema validation)

**UI / Styling:**
- Tailwind CSS 3.4.19 (+ `@tailwindcss/forms`, `@tailwindcss/typography`)
- PostCSS 8.5.6, autoprefixer 10.4.23
- Radix UI Dialog 1.1.15
- cmdk 1.1.1 (command palette)
- react-icons 5.6.0
- react-joyride 3.1.0 (walkthroughs)
- react-markdown 10.1.0
- lottie-react 2.4.1
- three.js 0.183.2 + `@types/three`
- @remotion/player 4.0.454 + remotion 4.0.454 (mascot rendering)

**Tauri Plugins (frontend bindings):**
- `@tauri-apps/api` ^2.10.0 (resolution-pinned to 2.10.1 root-level)
- `@tauri-apps/plugin-deep-link` ^2
- `@tauri-apps/plugin-opener` ^2 (init-iife.js disabled by audit policy)
- `@tauri-apps/plugin-os` ^2.3.2

**Tauri Plugins (Rust side, `app/src-tauri/Cargo.toml`):**
- `tauri-plugin-deep-link` 2.0.0
- `tauri-plugin-global-shortcut` 2
- `tauri-plugin-notification` (vendored at `vendor/tauri-plugin-notification`)
- `tauri-plugin-opener` 2
- `tauri-plugin-single-instance` 2 (features: `deep-link`) — prevents CEF double-init panic
- `tauri-plugin-updater` 2 (app bundle updater)

**Rust Core Frameworks:**
- `tokio` 1 (features: `full`, `sync`) — async runtime
- `axum` 0.8 (default-features off, features: `http1`, `json`, `tokio`, `query`, `ws`, `macros`) — HTTP/JSON-RPC transport
- `tower` 0.5 (middleware)
- `socketioxide` 0.15 (features: `extensions`) — Socket.IO server
- `clap` 4.5 (derive) + `clap_complete` 4.5 — CLI
- `serde` 1 + `serde_json` 1 + `serde_yaml` 0.9 + `toml` 1.0 — serialization
- `schemars` 1.2 — controller schema generation
- `async-trait` 0.1, `thiserror` 2.0, `anyhow` 1.0, `futures` 0.3, `futures-util` 0.3
- `tracing` 0.1 + `tracing-subscriber` 0.3 + `tracing-appender` 0.2 + `tracing-log` 0.2
- `log` 0.4 + `env_logger` 0.11
- `dialoguer` 0.12 (interactive CLI), `console` 0.16, `nu-ansi-term` 0.46

**Crypto / Security (Rust):**
- `rustls` 0.23 (ring), `tokio-rustls` 0.26.4, `webpki-roots` 1.0.6, `rustls-pki-types` 1.14.0
- `aes-gcm` 0.10, `chacha20poly1305` 0.10, `argon2` 0.5, `sha2` 0.10, `hmac` 0.12
- `ring` 0.17, `base64` 0.22, `hex` 0.4
- `ethers-core` 2.0.14, `ethers-signers` 2.0.14 (wallet domain)

**Storage / Data (Rust):**
- `rusqlite` 0.37 (bundled SQLite)
- `postgres` 0.19 (`with-chrono-0_4`) — used in test infra
- `chrono` 0.4 (serde), `chrono-tz` 0.10, `iana-time-zone` 0.1
- `cron` 0.12 (cron scheduling)
- `tempfile` 3, `dirs` 5, `directories` 6, `shellexpand` 3.1, `walkdir` 2, `glob` 0.3
- `fs2` 0.4 (file locking)

**HTTP / Networking (Rust):**
- `reqwest` 0.12 (default-features off, features: `json`, `blocking`, `rustls-tls`, `native-tls`, `stream`, `http2`, `multipart`, `socks`)
- `tokio-tungstenite` 0.24 (`rustls-tls-webpki-roots`) — WebSocket / CDP
- `url` 2, `urlencoding` 2.1
- `motosan-ai-oauth` 0.2 (`codex` feature) — Codex/OpenAI OAuth helper

**Email (Rust):**
- `lettre` 0.11.22 (`builder`, `smtp-transport`, `rustls-tls`) — SMTP send
- `mail-parser` 0.11.2
- `async-imap` 0.11 (`runtime-tokio`) — IMAP

**Media (Rust):**
- `whisper-rs` 0.16 (+ `metal` feature on macOS) — speech-to-text. Uses patched `whisper-rs-sys` fork from `tinyhumansai/whisper-rs-sys` for Windows MSVC /MT CRT
- `cpal` 0.15 — audio I/O
- `hound` 3.5 — WAV
- `image` 0.25 (png, jpeg)
- `resvg` 0.45 + `tiny-skia` 0.11 — SVG/PNG for mascot fake camera (Tauri shell)

**Telemetry / Errors:**
- Frontend: `@sentry/react` ^10.38.0, `@sentry/vite-plugin` ^2.22.6
- Rust (core + shell): `sentry` 0.47.0 (rustls, reqwest, panic, backtrace, contexts, debug-images, tracing)
- OpenTelemetry: `opentelemetry` 0.32, `opentelemetry_sdk` 0.32, `opentelemetry-otlp` 0.32 (trace + metrics, http-proto)
- `prometheus` 0.14

**Build/Dev:**
- Vite 8.0.0 + `@vitejs/plugin-react` 6.0.1 + `vite-plugin-node-polyfills` 0.26.0
- TypeScript ~5.8.3 (`tsc --noEmit` as `pnpm compile`)
- ESLint 9.39.2 + `@typescript-eslint/eslint-plugin` 8.54.0 + `eslint-config-prettier` 10.1.8 + `eslint-plugin-import` 2.32.0 + `eslint-plugin-react` 7.37.5 + `eslint-plugin-react-hooks` 7.0.1
- Prettier 3.8.1 + `@trivago/prettier-plugin-sort-imports` 6.0.2
- Husky 9.1.7 (pre-push runs `pnpm rust:check`)
- Knip 6.3.1 (dead-code detection, `app/knip.json`)
- cross-env 10.1.0
- tsx 4.20.3 (root)

**Build toolchain (native):**
- `cmake` required for `whisper-rs-sys`
- `xz2` 0.1 (static liblzma), `flate2` 1, `tar` 0.4, `zip` 2 — Node runtime bootstrap
- **Vendored `tauri-cli`** at `app/src-tauri/vendor/tauri-cef/crates/tauri-cli` — stock `@tauri-apps/cli` produces broken bundles (CEF library_loader panic). Installed via `pnpm tauri:ensure` → `scripts/ensure-tauri-cli.sh`.

## Testing Frameworks

**JS/TS:**
- Vitest 4.0.18 + `@vitest/coverage-v8` 4.0.18
- `@testing-library/react` 16.3.2, `@testing-library/dom` 10.4.1, `@testing-library/jest-dom` 6.9.1, `@testing-library/user-event` 14.6.1
- jsdom 28.0.0
- WDIO 9.24.0 stack: `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`, `@wdio/appium-service`
  - Linux: `tauri-driver` (WebDriver :4444)
  - macOS: Appium Mac2 (XCUITest :4723)

**Rust:**
- `cargo test` via `scripts/test-rust-with-mock.sh`
- `wiremock` 0.6 (dev-dep) — HTTP mocking for inference provider E2E
- `sentry` 0.47 with `test` feature for observability smoke tests
- `tokio` `test-util` feature for `start_paused` timer tests (Tauri shell)
- `tempfile` 3 dev-dep

**Coverage gate:** `≥80%` on changed lines, enforced by `.github/workflows/coverage.yml` via `diff-cover` over merged Vitest LCOV + `cargo-llvm-cov` LCOV (core + shell).

## Key Domain Dependencies

**Critical:**
- `openhuman_core` (path = `../..`, package = `openhuman`) — Tauri shell embeds the core crate directly (in-process tokio task)
- `whatsapp-rust` 0.5 (+ `whatsapp-rust-tokio-transport`, `whatsapp-rust-ureq-http-client`, `wacore`) — optional, gated by `whatsapp-web` feature
- `matrix-sdk` 0.16 (optional, `channel-matrix` feature) — Matrix protocol
- `fantoccini` 0.22.0 (optional, `browser-native` feature) — WebDriver
- `pdf-extract` 0.10 (optional, `rag-pdf` feature)
- `starship-battery` 0.10 — scheduler gate (laptop throttling)
- `sysinfo` 0.33 (`system` feature)
- `enigo` 0.3, `arboard` 3, `rdev` 0.5 — input simulation / clipboard
- `wait-timeout` 0.2 — bounded subprocess probes

**Platform-specific (Rust):**
- macOS: `objc2` 0.6 + `objc2-foundation` 0.3 + `objc2-contacts` 0.3.2 + `objc2-app-kit` 0.3.2 + `objc2-web-kit` 0.3.2 + `objc2-user-notifications` 0.3.2 + `block2` 0.6 + `mac-notification-sys` 0.6
- Linux: `landlock` 0.4 (optional, `sandbox-landlock` feature), `rppal` 0.22 (optional, `peripheral-rpi`), `notify-rust` 4 (`dbus`)
- Windows: `windows-sys` 0.59 (Console, WindowsAndMessaging, Threading, Security, Foundation)
- Unix: `nix` 0.29 (`signal`, `user`)

## Cargo Features

**Core (`Cargo.toml`):**
- `sandbox-landlock`, `sandbox-bubblewrap`, `channel-matrix`, `peripheral-rpi`, `browser-native` (alias `fantoccini`), `landlock`, `rag-pdf`, `whatsapp-web`, `e2e-test-support` (exposes `openhuman.test_reset`)

**Tauri shell (`app/src-tauri/Cargo.toml`):**
- `default` = none
- `custom-protocol` — Tauri serves bundled frontend via `tauri://localhost` (auto-enabled by `cargo tauri build`)
- `sandbox-bubblewrap`
- `e2e-test-support` — forwarded to core

## Configuration

**Env files:**
- `.env.example` (root) — Rust core: backend URL, logging, proxy, storage paths, AI binary overrides
- `app/.env.example` — `VITE_*` for frontend: core RPC URL, backend URL, Sentry DSN
- Loaded via `scripts/load-dotenv.sh`

**TOML config:**
- Rust `Config` struct: `src/openhuman/config/schema/types.rs`
- Env overrides: `src/openhuman/config/schema/load.rs`

**Frontend config:**
- Centralized in `app/src/utils/config.ts` — never read `import.meta.env` elsewhere

**Tauri config:**
- `app/src-tauri/tauri.conf.json` (bundles AI prompt resources from `src/openhuman/agent/prompts/`)

## Build Profiles

- `release`: `debug = "line-tables-only"`, `split-debuginfo = "packed"` — slim shipped binary, Sentry-symbolicatable
- `ci`: inherits release, `opt-level=1`, `codegen-units=16`, `lto=false`, `incremental=false`, `strip=true` — fast CI builds

## Platform Requirements

**Development:**
- Node >=24.0.0, pnpm 10.10.0
- Rust toolchain (stable, edition 2021)
- cmake (whisper-rs build)
- CEF runtime — auto-downloaded by `cef-dll-sys` build script on first `cargo tauri` build
- macOS: Xcode CLT (Appium Mac2 for E2E)
- Windows: MSVC toolchain; vendored `whisper-rs-sys` fork forces static CRT (/MT)
- Linux: `tauri-driver` for E2E

**Production deployment:**
- Desktop bundles: `.app`/`.dmg` (macOS), `.exe`/`.msi` (Windows), `.AppImage`/`.deb` (Linux)
- Built only via vendored `tauri-cli` from `app/src-tauri/vendor/tauri-cef/crates/tauri-cli`

---

*Stack analysis: 2026-05-22*

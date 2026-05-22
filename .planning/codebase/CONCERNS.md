# Codebase Concerns

**Analysis Date:** 2026-05-22

## Tech Debt

**Pre-push hook reformats unrelated files (line endings):**
- Issue: Running `git push` triggers Prettier / `cargo fmt` across the workspace, which rewrites ~940 files (CRLF→LF on Windows checkouts) including `app/public/lottie/*.json` and `app/src-tauri/Cargo.lock`. Empirically observed on `feat/claude-code-provider`.
- Files: Husky config in `app/.husky/`, formatters configured at repo root (`pnpm format` covers Prettier + `cargo fmt`).
- Impact: Forces contributors into a `git push --no-verify` workflow (sanctioned in `CLAUDE.md` "Git workflow" section), which defeats the hook and lets actual format errors slip through.
- Fix approach: Either (a) constrain Prettier/`cargo fmt` in pre-push to only changed files (use `lint-staged` style filtering), (b) commit a `.gitattributes` policy that normalizes EOL on checkout, or (c) move format enforcement to a CI-only gate.

**Submodule drift on `tauri-cef`:**
- Issue: `app/src-tauri/vendor/tauri-cef` shows ` m` (untracked modifications inside the submodule) on a clean clone across most workstations. Currently dirty on this branch (`git status --short` confirms).
- Files: `app/src-tauri/vendor/tauri-cef`, `.gitmodules`, `scripts/ensure-tauri-cli.sh`.
- Impact: `git status` is permanently noisy; contributors can't trust the "clean tree" signal; `--no-verify` becomes habitual.
- Fix approach: Document the cause (likely line-ending normalization or `Cargo.lock` regeneration inside the vendored submodule on `pnpm tauri:ensure`) in `CLAUDE.md`. Either pin the submodule with `update = none` for non-maintainers, or pre-build the CEF-aware CLI into a release artifact and skip the in-tree install.

**Legacy top-level Rust modules grandfathered:**
- Issue: `src/openhuman/dev_paths.rs` and `src/openhuman/util.rs` violate the "new code lives in a subdirectory" rule from `CLAUDE.md` but are kept indefinitely.
- Files: `src/openhuman/dev_paths.rs`, `src/openhuman/util.rs`, `src/openhuman/mod.rs`.
- Impact: Mixed precedent; reviewers must enforce the rule manually since the codebase itself shows counter-examples. `ceil_char_boundary` in `util.rs` is widely used so it can't be quietly relocated.
- Fix approach: Move `ceil_char_boundary` into a `src/openhuman/text/` or `src/openhuman/strings/` module; move dev-only path helpers into `src/openhuman/config/` (where `load.rs` already lives). Track via a single grooming PR.

**Skills runtime removed — domain is metadata-only:**
- Issue: `src/openhuman/skills/` retains `ops_create`, `ops_discover`, `ops_install`, `ops_parse`, `inject`, `schemas`, `types` after QuickJS/`rquickjs` removal. Anything that still expects skill execution end-to-end is dead.
- Files: `src/openhuman/skills/inject.rs` (carries `#[allow(dead_code)]` x3 — confirmed via grep), `src/openhuman/skills/mod.rs` (header comment "Legacy skill metadata helpers retained after QuickJS runtime removal").
- Impact: Any caller relying on skill execution (downstream agents, prompts referencing skill outputs) silently no-ops. Webhook router previously hardcoded HTTP 410 "skill runtime removed" for this reason (see `.claude/memory.md` "Webhook & Cron Triggers" entry).
- Fix approach: Audit consumers of `skills::inject` / `ops_install`. Either restore an execution path (new sandbox) or delete the metadata APIs once consumers are confirmed dead.

## Known Bugs / Build Blockers

**Whisper-rs CMake dependency surfaces opaquely:**
- Symptom: `pnpm dev:app` fails inside `whisper-rs-sys-*/build.rs` when CMake isn't on `PATH`. On Windows, CMake commonly only exists under `C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin`.
- Files: `Cargo.toml:130,162`, `app/src-tauri/Cargo.toml:189-192` (forked `whisper-rs-sys` patches `/MT` MSVC CRT mismatch but does not address the CMake-on-PATH requirement).
- Trigger: Fresh dev shell without VS dev-tools env activation, or contributors without VS BuildTools at all.
- Workaround: Pre-install CMake system-wide, or run from `Developer PowerShell for VS 2022`. On macOS Tahoe (Apple Silicon) there's a parallel issue — `GGML_NATIVE=ON` breaks Apple clang 21+; see `.claude/memory.md` "Build Blockers" section for the registry-patch workaround.

**In-process core PID-reuse race (mitigated, not eliminated):**
- Symptom: When the listener port (`7788`) is occupied by a stale process, the core handle probes `GET /`, then term/force-kills the PID. PR #1130 added re-validation of the PID before force-kill to avoid killing an unrelated process that recycled the PID. The race window is narrower but not zero.
- Files: `app/src-tauri/src/core_process.rs` (`CoreProcessHandle`); see CLAUDE.md "Tauri shell" section and `.claude/memory.md` "Core process" entry.
- Workaround: `OPENHUMAN_CORE_REUSE_EXISTING=1` to attach instead of killing; on suspect environments, `lsof -i :7788` then `kill <PID>` manually.

## Security Considerations

**CEF child webviews: no new JS injection (third-party origins):**
- Risk: Tauri plugins can ship default JS init scripts (`init-iife.js`) that run inside provider webviews loading `web.telegram.org`, `linkedin.com`, etc. This is a scraping/attack-surface liability — host-controlled JS executes inside third-party origins.
- Files: `app/src-tauri/src/lib.rs:2367-2380` (explicit `.open_js_links_on_click(false)` on `tauri-plugin-opener`), `app/src-tauri/src/webview_accounts/` (provider webviews), `app/src-tauri/Cargo.toml:48,215` (pinned `tauri-plugin-opener` git rev).
- Current mitigation: `tauri-plugin-opener` opt-out at registration. CLAUDE.md "CEF child webviews — no new JS injection" rule documents the ban. Migrated providers (whatsapp/telegram/slack/discord/browserscan) ship zero injected JS.
- Recommendation: Any new Tauri plugin added to `app/src-tauri/src/lib.rs` must be audited for a `js_init_script` call before merge. Add an automated check (grep CI step) that flags new `addScriptToEvaluateOnNewDocument` / `Runtime.evaluate` calls under `webview_accounts/`.

**Path validation must precede `create_dir_all`:**
- Risk: Symlink TOCTOU lets a malicious file path create directories outside the workspace.
- Files: `src/openhuman/security/policy.rs` (`validate_path`, `validate_parent_path`), all tool impls under `src/openhuman/tools/impl/filesystem/`.
- Current mitigation: Issue #1927 fix — `validate_parent_path` is called *before* `create_dir_all`. Legacy `is_path_allowed` / `is_resolved_path_allowed` deprecated.
- Recommendation: Add a clippy/lint rule or grep CI check that flags `create_dir_all` calls not preceded by `validate_parent_path` in the same fn.

## Outstanding Deferred Items — Claude Code Provider (PR #2472)

Embedded directly in module headers; tracked here so they don't drift:

- **Subscription / OAuth auth (Claude Pro/Max) — deferred to v2.** `src/openhuman/inference/provider/claude_code/auth.rs:12`.
- **AuthService-backed key lookup — v1.1.** Will wire `auth-profiles.json`. `src/openhuman/inference/provider/claude_code/auth.rs:10`.
- **Write-tool MCP exposure — v1.1.** Not yet exposed.
- **Cost wiring into `src/openhuman/cost/`** — Provider does not yet contribute usage rows to the cost domain.
- **`ChatRequest` carrying `thread_id` — Phase 4 deferred.** Current impl in `src/openhuman/inference/provider/claude_code/mod.rs:120,144` hashes the first user message as a synthetic session key. Two different conversations with identical first messages will collide; renames/edits of the first message reset the session.
- **v2 native protocol.** `src/openhuman/inference/provider/claude_code/mod.rs:5` notes v1 calls Anthropic HTTP API directly; v2 will use OpenHuman's native streaming surface.

## Stale Documentation Risk

**`.claude/memory.md` is dense and partially stale:**
- File: `C:\Users\artic\GitHub\openhuman\.claude\memory.md` (260 lines).
- Stale entries observed:
  - "Settings is a full route, not a modal" contradicts `.claude/rules/15-settings-modal-system.md` — the rule file is explicitly called out as outdated and should be deleted, not just countered in memory.
  - `voice-mode.spec.ts` "still references legacy labels that don't match current steps (pre-existing tech debt)" — open-ended.
  - "Pre-existing flaky tests" (composio::action_tool, agent::harness::session::turn) — accepted as flaky rather than triaged.
- Recommendation: Quarterly memory-keeper pass to age out entries that have been superseded by code changes; resolve or delete the `.claude/rules/15-settings-modal-system.md` reference.

## Test Coverage Gaps

**`#[allow(dead_code)]` clusters indicate untested or speculative APIs:**
- 21 files contain `#[allow(dead_code)]` (full list via `grep`). Notable clusters:
  - `src/openhuman/socket/manager.rs`, `src/openhuman/socket/types.rs` — socket transport.
  - `src/openhuman/agent/harness/test_support.rs`, `src/openhuman/agent/harness/session/tests.rs` — agent harness test plumbing has dead helpers, suggests test scaffolding rot.
  - `src/openhuman/inference/provider/compatible_types.rs`, `src/openhuman/inference/local/ollama.rs` — provider abstractions with unreached branches.
  - `src/openhuman/memory/tree/store.rs`, `src/openhuman/memory/tree/read_rpc.rs` — high-traffic memory tree module.
- Recommendation: Each `#[allow(dead_code)]` should either get a test that exercises it or be deleted. Memory tree (602 tests under `memory::tree` per `.claude/memory.md`) is well-covered; socket/inference providers are not.

**Coverage gate is mandatory:**
- Requirement: ≥ 80% on changed lines via `diff-cover` (`.github/workflows/coverage.yml`), merging Vitest (`app/coverage/lcov.info`) + `cargo-llvm-cov` lcov outputs.
- Risk: PRs that add new branches without unit tests cannot merge. New code on `feat/claude-code-provider` (`src/openhuman/inference/provider/claude_code/*`) must hit this bar — verify before requesting review.
- File: `.github/workflows/coverage.yml`.

## Fragile Areas

**`CoreStateProvider` — high blast radius:**
- Files: `app/src/providers/CoreStateProvider.tsx` (consumed by ~25 components per `.claude/memory.md`).
- Why fragile: Auth bootstrap path; race conditions with sidecar startup historically caused blank Settings screens (issue #413, #2158). Premature `isBootstrapping: false` cascades into redirects.
- Safe modification: Always preserve the 5-attempt bootstrap retry with `bootstrapFailCountRef` reset on success. Keep `RouteLoadingScreen` mounted during bootstrap.

**Provider webview migration is partial:**
- Files: `app/src-tauri/src/webview_accounts/` (migrated providers ship zero JS); legacy injection still present for `gmail`, `linkedin`, `google-meet` (`runtime.js` bridge + recipe files).
- Why fragile: Two parallel patterns in the same directory tree — easy for a new contributor to extend the legacy one. The CLAUDE.md rule says legacy injection is "grandfathered but should shrink, not grow"; no automated enforcement.
- Safe modification: New providers must use CDP from the scanner side (`*_scanner/` modules) only.

## Pre-existing Test Failures (accepted)

- `composio::action_tool::tests::factory_routes_through_direct_when_mode_is_direct` — unrelated to current branch work; do not fix unless tasked.
- `composio::action_tool::tests::mode_toggle_between_calls_is_observed` — flaky in full suite, passes in isolation. Shared global composio session state.
- `agent::harness::session::turn` — intermittent in full suite, passes individually.

---

*Concerns audit: 2026-05-22*

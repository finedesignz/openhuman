# Testing Patterns

**Analysis Date:** 2026-05-22

## Test Framework

**Frontend Runner:**
- Vitest
- Config: `app/test/vitest.config.ts`
- Setup: `app/src/test/setup.ts`

**E2E Runner:**
- WebdriverIO (WDIO)
- Config: `app/test/wdio.conf.ts`
- Linux (CI): `tauri-driver` (WebDriver on :4444)
- macOS (local): Appium Mac2 (XCUITest on :4723) against built `.app` bundle

**Rust:**
- `cargo test` via `scripts/test-rust-with-mock.sh` (boots shared mock backend before tests).

**Run Commands (from repo root):**
```bash
pnpm test                                # Vitest, app workspace
pnpm test:coverage                       # Vitest + coverage (lcov)
pnpm test:rust                           # cargo test with mock backend
pnpm test:e2e:build                      # build .app bundle for E2E
pnpm test:e2e:all:flows                  # run all E2E flow specs
bash app/scripts/e2e-run-spec.sh test/e2e/specs/smoke.spec.ts smoke
docker compose -f e2e/docker-compose.yml run --rm e2e   # Linux E2E on macOS
pnpm mock:api                            # run shared mock backend manually
```

## Test File Organization

**Vitest unit tests:**
- Co-located: `app/src/**/*.test.ts` or `*.test.tsx` next to source.
- Setup: `app/src/test/setup.ts`.
- Helpers: `app/src/test/`.

**WDIO E2E specs:**
- `app/test/e2e/specs/*.spec.ts` (one spec per flow).
- Helpers: `app/test/e2e/helpers/`.
- Mock server wrapper: `app/test/e2e/mock-server.ts`.

**Rust tests:**
- Integration tests under `tests/*.rs` (e.g. `tests/json_rpc_e2e.rs`).
- Unit tests inline `#[cfg(test)] mod tests`.

## Test Structure

**Vitest:**
- Use Testing Library; prefer behavior assertions over implementation.
- No real network. No time flakes — fake timers / deterministic clocks when needed.
- Use helpers in `app/src/test/` for common setup.

**WDIO:**
- Always use `app/test/e2e/helpers/element-helpers.ts`:
  - `clickNativeButton(...)`
  - `waitForWebView(...)`
  - `clickToggle(...)`
- NEVER use raw `XCUIElementType*` selectors.
- Assert UI outcomes AND mock-backend effects (via admin endpoints below).

## Shared Mock Backend

Used by Vitest and Rust tests.

**Files:**
- Core: `scripts/mock-api-core.mjs`
- Server: `scripts/mock-api-server.mjs`
- E2E wrapper: `app/test/e2e/mock-server.ts`

**Admin endpoints:**
- `GET /__admin/health`
- `POST /__admin/reset`
- `POST /__admin/behavior`
- `GET /__admin/requests`

## Deterministic E2E Core Reset

- `app/scripts/e2e-run-spec.sh` creates and cleans a temp `OPENHUMAN_WORKSPACE`.
- `OPENHUMAN_WORKSPACE` redirects core config + storage away from `~/.openhuman`.
- Each spec gets a fresh in-process core inside the freshly-built Tauri bundle.

## Mocking

**Frontend:**
- `vi.mock(...)` for module mocks.
- Mock `coreRpcClient` / `apiClient` at the service boundary, not Tauri internals.

**Rust:**
- Point HTTP clients at the mock backend (`scripts/test-rust-with-mock.sh` exports the URL).
- Use admin `POST /__admin/behavior` to script responses.

**Do NOT mock:** Redux store internals, React Router, Tauri's `invoke` IPC (use `isTauri()` guards instead).

## Coverage Gate

**Merge requirement:** ≥ 80% coverage on changed lines.

**Enforcement:** `.github/workflows/coverage.yml`
- Tool: `diff-cover`.
- Inputs: merged Vitest (`app/coverage/lcov.info`) + `cargo-llvm-cov` lcov (core crate + Tauri shell).
- PR will not merge below threshold. Add tests for new/changed lines, not just happy paths.

## Test Types

**Unit (Vitest):**
- Component behavior, hook logic, slice reducers, service modules.
- Co-located with source.

**Integration / RPC E2E (Rust):**
- `tests/json_rpc_e2e.rs` exercises core JSON-RPC over real HTTP against mock backend.
- Extend when adding new RPC methods.

**E2E (WDIO):**
- User-visible desktop flows on the built `.app` (macOS) or Linux tauri-driver.
- Specs in `app/test/e2e/specs/`.

## Debug Runners (`scripts/debug/`)

Bounded-output wrappers — stdout stays summary-sized, full output teed to `target/debug-logs/<kind>-<suffix>-<timestamp>.log`. Prefer over raw Vitest / WDIO / cargo when iterating.

```bash
pnpm debug unit                                    # all Vitest
pnpm debug unit src/components/Foo.test.tsx        # one file
pnpm debug unit -t "renders empty state"           # filter by name
pnpm debug unit Foo -t "renders empty" --verbose   # +stream raw

pnpm debug e2e test/e2e/specs/smoke.spec.ts        # one spec
pnpm debug e2e test/e2e/specs/cron-jobs-flow.spec.ts cron-jobs --verbose

pnpm debug rust                                    # all cargo tests (with mock)
pnpm debug rust json_rpc_e2e                       # single test

pnpm debug logs                                    # list 50 most recent
pnpm debug logs last                               # print most recent (last 400 lines)
pnpm debug logs unit                               # most recent matching "unit"
pnpm debug logs last --tail 100
```

Entry: `pnpm debug` (`scripts/debug/cli.sh`). Implementation files: `scripts/debug/{cli,unit,e2e,rust,logs,lib}.sh` + `README.md`.

## Feature Workflow Test Gates

Per `CLAUDE.md` "Feature design workflow":
1. Rust unit tests until domain correct in isolation.
2. Extend `tests/json_rpc_e2e.rs` / `scripts/test-rust-with-mock.sh` so RPC matches what the UI calls.
3. Vitest unit tests for new app code.
4. WDIO E2E spec for user-visible flow.

**Planning rule:** define E2E scenarios (core RPC + app) covering happy paths, failure modes, auth gates, regressions before implementing. Not testable end-to-end ⇒ incomplete spec or too-large cut.

## Common Patterns

**Async testing:** prefer `await` over callbacks; use Vitest's `vi.useFakeTimers()` for time-sensitive logic.

**Error paths:** assert structured `RpcOutcome<T>` error variants in Rust RPC tests, not stringly-matched messages.

**Mock reset:** call `POST /__admin/reset` between specs / scenarios that share the mock backend.

---

*Testing analysis: 2026-05-22*

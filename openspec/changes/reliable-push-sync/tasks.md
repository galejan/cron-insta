# Tasks: Reliable Push Sync

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~180 (Rust) + ~10 (Svelte) + ~9 (CI) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR — all commits in `main` |
| Delivery strategy | already applied |

**All tasks completed in commits:**

## Phase 1: SSH Agent Reliability

- [x] 1.1 `system_command()`: add `GIT_SSH_COMMAND` with `BatchMode=yes` + `ConnectTimeout=5` (prevents 30s SSH hangs)
- [x] 1.2 `system_command()`: add `find_ssh_auth_sock_fallback()` for desktop-launched Tauri (no terminal env)
- [x] 1.3 `ssh_available()` helper: checks env var + fallback paths

## Phase 2: Decouple Commit from Push

- [x] 2.1 `perform_commit()`: all error paths return `Ok(msg)` + `eprintln!` instead of `Err` (never blocks push)
- [x] 2.2 Remove `push_enabled` gate from `sincronizar_checkpoint()` (close and button always push)
- [x] 2.3 Re-enable `push_enabled` on successful push in `sincronizar_checkpoint()` (auto-recover from 3-strike)
- [x] 2.4 `commit_metadata_file()` helper: `git add .config/metadata.json && git commit -m "cron-insta: ..."` after push

## Phase 3: Full Sync Cycle

- [x] 3.1 `sync_with_remote()`: fetch → `rev-list --left-right @{upstream}...HEAD` → ahead/behind → pull --ff-only if behind → push if ahead
- [x] 3.2 `do_checkpoint()`: use `sync_with_remote()` instead of `check_ahead` + `sincronizar_checkpoint`
- [x] 3.3 `push_ahora()`: use `sync_with_remote()` instead of `check_ahead` + `sincronizar_checkpoint`
- [x] 3.4 Remove `check_ahead()` (replaced by `sync_with_remote()`)

## Phase 4: Save Button Fix

- [x] 4.1 Extract `async function doSave()` from debounced closure in `+page.svelte`
- [x] 4.2 Button handler calls `await doSave()` directly (immediate, no 20s debounce)

## Phase 5: CI Warning

- [x] 5.1 Add `pnpm svelte-kit sync` step before `pnpm tauri build` in all 3 CI jobs

## Phase 6: Verification

- [x] 6.1 `cargo test` — 99 passed, 0 failed
- [x] 6.2 `pnpm check` — 0 errors, 0 warnings
- [x] 6.3 Manual: close with ahead commits → push succeeds
- [x] 6.4 Manual: save button → instant save (no 20s delay)
- [x] 6.5 Manual: `git status` clean after push (no dirty metadata.json)

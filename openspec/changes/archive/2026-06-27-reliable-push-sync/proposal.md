# Proposal: Reliable Push Sync

## Intent

Fix three interconnected bugs in the push-on-close flow that made auto-push unreliable for non-technical users:

1. **Commit failure skipped push**: `perform_commit` used `?` — if commit failed (git lock, disk issue), push was silently skipped even when there were pending commits.
2. **No fetch + ahead check**: Push ran blind — `sincronizar_checkpoint` called `git push` without checking if local was actually ahead of remote.
3. **No pull before push**: If remote had advanced (user switched PCs), git rejected the push. No pull was attempted.

## Scope

### In Scope
- `perform_commit` becomes best-effort (never returns `Err`, always `Ok`)
- New `check_ahead()` → `git fetch` + `git rev-list` to detect ahead commits
- New `sync_with_remote()` → fetch → pull (if behind) → push (if ahead)
- `do_checkpoint` (close) uses `sync_with_remote` instead of raw push
- `push_ahora` (save & push button) uses `sync_with_remote`
- `crear_checkpoint` (auto-save timer) unchanged — commit only, no push
- Remove `push_enabled` gate from `sincronizar_checkpoint` — close and button always push
- `SSH_AUTH_SOCK` fallback for desktop-launched Tauri (no terminal env)
- `GIT_SSH_COMMAND` with `ConnectTimeout=5` + `BatchMode=yes` to avoid 30s hangs
- Metadata changes after push are committed immediately to keep git status clean
- Save button calls `doSave()` directly (bypasses 20s debounce)

### Out of Scope
- Windows SSH agent support
- Git merge conflict resolution UI
- HTTPS remote support
- Visual diff or sync status indicator

## Capabilities

### New Capabilities
- `sync-with-remote`: full fetch → pull → push cycle on close and button

### Modified Capabilities
- `git-abstraction`: `perform_commit` best-effort; `sync_with_remote` replaces `check_ahead`
- `git-remote-sync`: `sincronizar_checkpoint` no longer gates on `push_enabled`; commits metadata after push
- `git-auto-detection`: `system_command` gains `SSH_AUTH_SOCK` fallback + `GIT_SSH_COMMAND` timeout

## Approach

1. **`system_command()`** → add SSH_AUTH_SOCK fallback paths + GIT_SSH_COMMAND env
2. **`perform_commit()`** → all error paths return `Ok(msg)` instead of `Err`; log via `eprintln!`
3. **`sincronizar_checkpoint()`** → remove `push_enabled` gate; re-enable on success; commit metadata after write
4. **`sync_with_remote()`** → `find_git` → remote check → fetch → ahead/behind → pull --ff-only if behind → push if ahead
5. **`do_checkpoint()`** → call `sync_with_remote` instead of `check_ahead` + `sincronizar_checkpoint`
6. **`push_ahora()`** → call `sync_with_remote` instead of `check_ahead` + `sincronizar_checkpoint`
7. **`+page.svelte`** → extract `doSave()`; button calls `await doSave()` directly

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modified | `system_command`, `perform_commit`, `sincronizar_checkpoint` refactored; `ssh_available()`, `find_ssh_auth_sock_fallback()`, `commit_metadata_file()`, `sync_with_remote()` added; `check_ahead` removed; `do_checkpoint`, `push_ahora` updated |
| `src/routes/+page.svelte` | Modified | `doSave()` extracted; button calls direct save |
| `.github/workflows/build.yml` | Modified | Added `svelte-kit sync` step before build |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `git pull --ff-only` fails on diverged branches | Low | Pull aborts cleanly, no push attempted. User's commits stay local. |
| SSH agent not available on desktop launch | Medium | `find_ssh_auth_sock_fallback()` checks `/run/user/$UID/keyring/ssh` and `/run/user/$UID/ssh-agent.socket`. `GIT_SSH_COMMAND` ensures 5s timeout instead of 30s. |
| Metadata commit creates noise in git history | Low | Single commit per close/button with message "cron-insta: actualizar estado de sincronización". Only created when metadata actually changes. |

## Rollback Plan

Revert to prior commit. All changes are in `src-tauri/src/lib.rs`, `src/routes/+page.svelte`, and `.github/workflows/build.yml`.

## Dependencies

- `per-project-remote-config` (landed) — push state in `metadata.json`
- `auto-detect-git-config` (landed) — `SSH_AUTH_SOCK` in `system_command`

## Success Criteria

- [ ] Closing the app with commits ahead pushes them automatically
- [ ] Closing when remote has advanced pulls first, then pushes
- [ ] "Guardar y subir" button saves instantly and syncs
- [ ] No `metadata.json` dirty after push
- [ ] SSH operations timeout in ~5s instead of hanging 30s
- [ ] `git push` works when launched from desktop (not terminal)
- [ ] All 99 Rust tests pass, svelte-check 0 errors
- [ ] CI builds produce no tsconfig warnings

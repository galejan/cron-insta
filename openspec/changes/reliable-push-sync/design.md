# Design: Reliable Push Sync

## Technical Approach

Replace the simple "commit then push" model with a full remote sync: fetch → detect ahead/behind → pull if behind (fast-forward only) → push if ahead. Decouple commit from push so a commit failure never blocks the push. Add SSH agent fallback and timeouts for desktop-launched scenarios.

## Architecture Decisions

| Decision | Choice | Rejected | Rationale |
|----------|--------|----------|-----------|
| Sync strategy | `fetch → pull --ff-only → push` | Simple push-only | `--ff-only` is safe for non-technical users: either fast-forwards cleanly or aborts. No merge commits, no conflict states. |
| `perform_commit` error handling | Best-effort: always `Ok` | Returning `Err` on failure | A failed commit shouldn't skip push. There may be pending commits from auto-save. Log the error and continue. |
| `push_enabled` gate | Removed from `sincronizar_checkpoint` | Keeping it | Close and button are explicit user actions. `push_enabled` controls auto-push during editing (which no longer exists). Still stored in metadata for UI indicator. |
| Metadata after push | Commit automatically | Leaving dirty | Dirty `metadata.json` in git status is confusing. A small follow-up commit keeps the tree clean. |
| SSH agent discovery | Env var + fallback paths | Env var only | Tauri desktop launch doesn't inherit terminal `SSH_AUTH_SOCK`. Fallback to `/run/user/$UID/keyring/ssh` covers GNOME Keyring and common agents. |
| SSH timeout | `GIT_SSH_COMMAND` with `ConnectTimeout=5` + `BatchMode=yes` | No timeout | Default SSH timeout is 30+ seconds. For a desktop app this feels like a hang. 5s with `BatchMode=yes` (no password prompt) is acceptable. |
| Branch detection | `@{upstream}` via `rev-parse` | Hardcoded `origin/main` | Works with any branch name (main, master, etc.). Handles missing upstream gracefully. |
| Save button | Call `doSave()` directly | `save.trigger()` (20s debounce) | Button shows "guardando" immediately but save didn't run for 20s due to debounce timer. |

## Data Flow

```
Close / Save & Push
    │
    ├── perform_commit(path)         ← best-effort, always Ok
    │     git add . + git commit -m "..."
    │     returns hash or "Sin cambios para guardar."
    │
    └── sync_with_remote(app, path)
          │
          ├── find_git() + remote get-url origin
          ├── git fetch origin         ← only if SSH agent available
          ├── rev-list --left-right @{upstream}...HEAD
          │     → (ahead, behind)
          │
          ├── if behind > 0:
          │     git pull --ff-only     ← safe fast-forward
          │
          └── if ahead > 0:
                sincronizar_checkpoint(app, path)
                  ├── read metadata
                  ├── git push
                  ├── update metadata (consecutive_failures, etc.)
                  ├── git add .config/metadata.json
                  └── git commit -m "cron-insta: ..."
```

### Auto-save timer (unchanged)

```
Timer fires → guardarCapitulo → crearCheckpoint → perform_commit
                                                     └── commit only, NO push
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modify | `system_command`: SSH fallback + GIT_SSH_COMMAND. New `ssh_available()`, `find_ssh_auth_sock_fallback()`, `sync_with_remote()`, `commit_metadata_file()`. Refactored `perform_commit` (best-effort). Refactored `sincronizar_checkpoint` (no push_enabled gate, metadata commit). Refactored `do_checkpoint` and `push_ahora` (use sync_with_remote). Removed `check_ahead()`. |
| `src/routes/+page.svelte` | Modify | Extracted `doSave()` from debounced closure. Button calls `await doSave()` directly. |
| `.github/workflows/build.yml` | Modify | Added `pnpm svelte-kit sync` before build to avoid tsconfig warning. |

## Interfaces / Contracts

### `sync_with_remote(app, path, project_path) -> Result<String, String>`

```
Entrada: app handle, project path string, project path Path
Salida:  Ok("") on clean sync
         Ok("warning message") on push warning (e.g. 3-strike)
         Err("error") on unexpected failure
Efectos: git fetch → git pull --ff-only (if behind) → git push (if ahead)
         Updates metadata.json + commits it
```

### `perform_commit(project_path) -> Result<String, String>`

```
Entrada: project path
Salida:  Ok("commit_hash") on new commit
         Ok("Sin cambios para guardar.") on no changes
         Ok("error message") on any failure (never Err)
Efectos: git add . + git commit
```

### `sincronizar_checkpoint(app, path) -> Result<String, String>`

```
Entrada: app handle, project path
Salida:  Ok("") on success
         Ok("warning") on 3-strike or partial failure
         Err("error") on unexpected failure
Efectos: git push → update metadata → commit metadata
         Does NOT check push_enabled flag
```

### `commit_metadata_file(project_path, git_exe)`

```
Entrada: project path, git executable path
Efectos: git add .config/metadata.json + git commit -m "cron-insta: ..."
         Best-effort: ignores errors (e.g. nothing to commit)
```

### `system_command(program) -> Command`

```
Entrada: program name (e.g. "git")
Salida:  std::process::Command with:
         - stdin: null
         - SSH_AUTH_SOCK: from env or fallback path
         - GIT_SSH_COMMAND: "ssh -o BatchMode=yes -o ConnectTimeout=5"
         - Windows: CREATE_NO_WINDOW flag
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | All 99 existing tests | `cargo test` — must pass unchanged |
| Manual | Close with ahead commits | Open project, write, close → verify push via git log |
| Manual | Close with behind remote | Simulate by pushing from another terminal, close app → verify pull |
| Manual | Save button | Click "guardar" → verify instant save (no 20s delay) |
| CI | Build on all 3 platforms | Verify no tsconfig warnings |

## Migration / Rollout

No migration needed. All changes are backward-compatible:
- `metadata.json` with `push_enabled: false` — close and button now push anyway
- Old `perform_commit` callers (none besides `crear_checkpoint`, `do_checkpoint`, `push_ahora`) — all updated
- SSH_AUTH_SOCK fallback is additive — no change for terminal-launched instances

## Open Questions

None.

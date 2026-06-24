# Proposal: Auto-Detect Git Config

## Intent

Cloned repos carry identity and `origin` remote in `.git/config`. Cron-Insta ignores this — users must manually reconfigure both. Detect and merge this data on project open.

## Scope

### In Scope
- Rust command `detectar_config_git(project_path)` → `{name, email, remote_url}` or nulls
- Non-blocking background call on `abrirProyecto` and auto-reopen
- Update global identity when detected values differ from stored
- Set `push_enabled: true` when remote found (unless 3-strike disabled)
- Toast: "Git detectado: origin → user/repo, identidad → Name <email>"

### Out of Scope
- Auto-configuring remote (must already be named `origin`)
- Overriding 3-strike auto-disable (`consecutive_failures > 0`)
- Detection during project creation
- Windows SSH agent detection

## Capabilities

### New Capabilities
- `git-auto-detection`: background detection of `.git/config` identity and remote on project open

### Modified Capabilities
- `git-identity-config`: identity updatable from auto-detected `.git/config`
- `git-remote-sync`: `push_enabled` auto-set when remote detected
- `project-file-management`: metadata.json updated on project open (push_enabled)

## Approach

1. **Rust**: `detectar_config_git` runs `git config user.name/email` + `git remote get-url origin` in project dir. Best-effort — returns `null` for unavailable values, never errors.
2. **TS binding**: `detectarConfigGit(path)` in `tauri.ts`.
3. **Frontend**: on open, compare detected identity with stored (via `cargarIdentidadGit`). If different, update. If remote found but `push_enabled` false, update metadata. Show toast.
4. **Guard**: skip auto-enable when `consecutive_failures > 0`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | New + Modified | New command + handler registration |
| `src/lib/tauri.ts` | New | `detectarConfigGit()` binding |
| `src/routes/+page.svelte` | Modified | Detection call in open + auto-reopen flows |
| `src/lib/i18n.svelte.ts` | Modified | Toast labels (es + en) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Overrides user-set identity | Low | Only updates when values differ |
| Auto-enables push on broken remote | Low | Skip when `consecutive_failures > 0` |
| Git unavailable | Low | Returns null — no-op |

## Rollback Plan

Revert commit. Detection is additive — identity/remote configs unaffected when removed.

## Dependencies

- `per-project-remote-config` (landed) — push state already in `metadata.json`

## Success Criteria

- [ ] Opening cloned repo shows toast with detected origin + identity
- [ ] Global identity updates when `.git/config` user differs
- [ ] `push_enabled: true` set when remote detected (unless 3-strike disabled)
- [ ] No `.git` / no remote → no toast, no side effects
- [ ] All existing Rust tests pass

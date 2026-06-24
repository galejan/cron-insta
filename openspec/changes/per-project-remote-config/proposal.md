# Proposal: Per-Project Remote Config

## Intent

Fix broken Git auto-push caused by two bugs: (1) push state stored in a **global** config shared across all projects, so configuring remote on project B silently overwrites project A's settings; (2) SSH agent unavailable in Tauri GUI, causing silent auth failures (confirmed by `consecutive_failures: 2` in global config). Users who configured a remote NEVER got push working because of SSH, and the failure counter was shared globally.

## Scope

### In Scope
- Move `push_enabled` and `consecutive_failures` from global `git-config.json` to per-project `metadata.json`
- Remove `remote` section from global `GitConfig` struct; keep identity-only
- Read remote URL from `git remote get-url origin` instead of storing it
- Pass `SSH_AUTH_SOCK` environment variable to git subprocess on Linux
- Surface push errors instead of silently dropping them (`let _` → log + update metadata counter)

### Out of Scope
- SSH key management UI or key generation
- Windows SSH agent support (no `SSH_AUTH_SOCK` equivalent)
- Changing the 3-strike rule behavior
- Adding HTTPS remote support

## Capabilities

### New Capabilities
None — this is a scope correction across existing capabilities.

### Modified Capabilities
- `git-remote-sync`: push state source moves from global config to per-project `metadata.json`; URL read dynamically from git
- `project-file-management`: `Metadata` struct gains `push_enabled: bool` and `consecutive_failures: u32` fields
- `project-settings-dialog`: Remote panel reads/writes project's git config instead of global config
- `git-abstraction`: `sincronizar_checkpoint` reads per-project state; SSH env passed to subprocess; push errors surfaced
- `git-identity-config`: global config loses `remote` section, becomes identity-only

## Approach

1. **Metadata struct**: add `push_enabled` (default `false`) and `consecutive_failures` (default `0`) fields with serde defaults
2. **sincronizar_checkpoint**: read push state from project `metadata.json` instead of global config; read remote URL via `git remote get-url origin`; pass `SSH_AUTH_SOCK` env to subprocess; return `Result` with surfaceable errors
3. **do_checkpoint**: replace `let _ = sincronizar_checkpoint(...)` with error-handled call that updates metadata counter
4. **reintentar_push / configurar_remoto / guardar_config_remoto**: operate on per-project metadata, not global config
5. **Remote panel UI**: `cargar_config_remoto` → reads project metadata; `guardar_config_remoto` → writes project metadata; `configurar_remoto` stays (operates on git remotes)
6. **remove_remote_section_from_global_config**: migration step — strip `remote` from existing `git-config.json` on first load
7. **Warning icon**: `actualizarGitStatus` reads from per-project metadata instead of global `cargarConfigRemoto`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modified | `Metadata` struct +2 fields; `sincronizar_checkpoint`, `do_checkpoint`, `reintentar_push`, `configurar_remoto`, `guardar_config_remoto`, `cargar_config_remoto` rewritten |
| `src/lib/tauri.ts` | Modified | TS signatures for `cargarConfigRemoto`, `guardarConfigRemoto` gain `projectPath` param |
| `src/lib/components/ProjectSettingsDialog.svelte` | Modified | Remote panel passes `projectPath` to config commands |
| `src/routes/+page.svelte` | Modified | `actualizarGitStatus` passes `projectPath`; `reintentarPush` call updated |
| `src/lib/i18n.svelte.ts` | Unchanged | Labels reuse existing keys, new error labels added |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Existing global config has stale remote data | High | Migration strips `remote` section from global config on first load; per-project state starts fresh (push_enabled=false) |
| `SSH_AUTH_SOCK` not set on all Linux desktops | Med | Attempt env var; if push fails, error is surfaced — user can fix SSH config |
| Metadata.json backward compat for projects without new fields | Low | Serde `#[serde(default)]` — old projects default to `push_enabled: false`, `consecutive_failures: 0` |

## Rollback Plan

1. Revert to prior commit. `metadata.json` fields are additive with defaults — old code just ignores them.
2. Global config migration is idempotent (only strips `remote` key). If reverted, re-saving identity restores valid config.

## Dependencies

- None external. Pure code change within existing Rust/Svelte architecture.

## Success Criteria

- [ ] Setting remote on project A does NOT affect project B's push state
- [ ] SSH push succeeds from Tauri GUI when `SSH_AUTH_SOCK` is set in the environment
- [ ] Push failures increment `consecutive_failures` in the project's `metadata.json`, not globally
- [ ] Warning icon in toolbar reflects per-project state
- [ ] All 84 existing Rust tests pass (`cargo test --manifest-path src-tauri/Cargo.toml`)
- [ ] Old projects (no new metadata fields) open without error

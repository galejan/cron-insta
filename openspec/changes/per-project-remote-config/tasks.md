# Tasks: Per-Project Remote Config

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 280–340 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR — all phases in one cohesive change |
| Delivery strategy | single-pr |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Metadata + SSH + Checkpoint + Config Cleanup + Frontend + Tests | PR 1 (single) | All changes in one PR; tests included per phase |

## Phase 1: Metadata Foundation

- [ ] 1.1 Add `push_enabled: bool` and `consecutive_failures: u32` to `Metadata` struct in `src-tauri/src/lib.rs` with `#[serde(default)]`
- [ ] 1.2 Update `crear_proyecto` Metadata literal in `src-tauri/src/lib.rs` to include `push_enabled: false`, `consecutive_failures: 0`
- [ ] 1.3 Update SCHEMA.md template in `crear_proyecto` (`src-tauri/src/lib.rs`) adding `push_enabled` and `consecutive_failures` rows to metadata table

## Phase 2: SSH Agent

- [ ] 2.1 Modify `system_command()` in `src-tauri/src/lib.rs` to inherit `SSH_AUTH_SOCK` env var on non-Windows: `if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") { cmd.env("SSH_AUTH_SOCK", sock); }`

## Phase 3: Checkpoint Rewrite

- [ ] 3.1 Rewrite `sincronizar_checkpoint` in `src-tauri/src/lib.rs`: read `push_enabled`/`consecutive_failures` from project `metadata.json` via `read_metadata`; read remote URL via `git remote get-url origin`; write back counter to metadata; remove global `GitConfig` dependency
- [ ] 3.2 Rewrite `do_checkpoint` in `src-tauri/src/lib.rs`: replace `let _ = sincronizar_checkpoint(...)` with error-handled call that logs via `eprintln!` and updates metadata `consecutive_failures`
- [ ] 3.3 Rewrite `reintentar_push` in `src-tauri/src/lib.rs`: read/write push state from project `metadata.json`; reset counter on success; increment on failure

## Phase 4: Config Cleanup

- [ ] 4.1 Remove `remote: Option<GitRemoteConfig>` field from `GitConfig` struct in `src-tauri/src/lib.rs`; keep `GitRemoteConfig` struct for test use
- [ ] 4.2 Rewrite `cargar_config_remoto` in `src-tauri/src/lib.rs`: add `proyecto_path: String` param; read from project `metadata.json`; return `{push_enabled, consecutive_failures}` (no url — frontend reads URL from git)
- [ ] 4.3 Rewrite `guardar_config_remoto` in `src-tauri/src/lib.rs`: add `proyecto_path: String` param; read-modify-write project `metadata.json`

## Phase 5: Frontend

- [ ] 5.1 Update `cargarConfigRemoto` and `guardarConfigRemoto` in `src/lib/tauri.ts`: add `projectPath: string` param; adjust return type (`cargarConfigRemoto` no longer returns `url`)
- [ ] 5.2 Update `loadRemote()`/`saveRemote()` in `src/lib/components/ProjectSettingsDialog.svelte`: pass `projectPath` to `cargarConfigRemoto`/`guardarConfigRemoto`
- [ ] 5.3 Update `actualizarGitStatus()` in `src/routes/+page.svelte`: call `cargarConfigRemoto(path)`; adjust `remoteWarningVisible` check (no `remote.url` field)
- [ ] 5.4 Update `guardarConfigRemoto` calls in `src/lib/components/GitIdentityDialog.svelte`: pass `projectPath` param (use empty string or omit; dialog operates pre-project-creation)

## Phase 6: Tests

- [ ] 6.1 Unit test — Metadata serde backward compat: deserialize old JSON (no push fields), verify defaults; round-trip preserves new fields (`src-tauri/src/lib.rs` `#[cfg(test)] mod tests`)
- [ ] 6.2 Unit test — GitConfig migration: deserialize config with `remote` key; verify identity preserved, remote silently ignored (`src-tauri/src/lib.rs` tests)
- [ ] 6.3 Unit test — Per-project isolation: create two projects, enable push on A, verify B stays `false` (`src-tauri/src/lib.rs` tests)
- [ ] 6.4 Unit test — SSH env: spawn a command via `system_command`, verify `SSH_AUTH_SOCK` inherited in child process (`src-tauri/src/lib.rs` tests)
- [ ] 6.5 Regression: run `cargo test --manifest-path src-tauri/Cargo.toml` — all 84 existing tests must pass

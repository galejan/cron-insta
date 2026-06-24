# Design: Per-Project Remote Config

## Technical Approach

Move push state (`push_enabled`, `consecutive_failures`) from **global** `git-config.json` into **per-project** `metadata.json`. Remove the `remote` section from the global `GitConfig` struct entirely (serde ignores unknown fields → migration is automatic on next write-back). Read the remote URL dynamically via `git remote get-url origin` instead of storing it. Pass `SSH_AUTH_SOCK` to all git subprocesses. Surface push errors through result types instead of silently dropping them.

## Architecture Decisions

| Decision | Choice | Rejected | Rationale |
|----------|--------|----------|-----------|
| Push state storage | `Metadata` struct in `.config/metadata.json` | Separate per-project config file | Metadata.json already carries project-specific state; adding 2 fields is minimal. One file to read/modify for push logic. |
| Remote URL source | `git remote get-url origin` at runtime | Cached in metadata or global config | Git is the source of truth; no risk of config-git mismatch. `git remote` is a local operation (<1ms). |
| Migration strategy | Drop `remote` field from `GitConfig`; serde ignores unknown keys on deserialize | Explicit migration function | Simpler: old files deserialize OK (unknown keys ignored), next `guardar_identidad_git` write-back cleans them. Idempotent by nature. |
| `GitConfig` struct | Remove `remote: Option<GitRemoteConfig>`; become identity-only (`schema_version`, `identity`) | Keep optional field | Dead code removed at the struct level. `GitRemoteConfig` struct stays as a test-only type for 3-strike logic tests. |
| SSH env propagation | Inherit `SSH_AUTH_SOCK` from current env in `system_command()` | Per-call SSH env passing | One change in `system_command` covers all git subprocesses. Env::var is cheap. |
| Error surfacing | `do_checkpoint` logs push errors via `eprintln!` and updates metadata counter; `sincronizar_checkpoint` returns `bool` | Full Result propagation | Close-handler can't surface UI; logging is pragmatic. Counter in metadata propagates via frontend on next open. |
| Frontend API change | `cargarConfigRemoto` and `guardarConfigRemoto` gain `projectPath: string` param | Separate new commands | Minimal delta to TS signatures; existing call sites update cleanly. |

## Data Flow

```
  ┌────────────────────┐
  │  ProjectSettings   │  guardarConfigRemoto(projectPath, url, true)
  │  Remote Panel      │────▶ writes push_enabled to .config/metadata.json
  └────────────────────┘       (URL stored in git config by configurar_remoto)

  ┌────────────────────┐
  │  +page.svelte      │  cargarConfigRemoto(projectPath)
  │  actualizarGitStatus│◀──── reads push state from metadata.json
  └────────────────────┘       checks git remote get-url origin for URL existence

  ┌────────────────────┐         ┌──────────────────┐
  │  do_checkpoint     │─commit─▶│ sincronizar_     │
  │  (close handler)   │         │ checkpoint       │
  └────────────────────┘         └──────┬───────────┘
                                        │ reads metadata.json → push_enabled?
                                        │ git remote get-url origin → URL
                                        │ git push (with SSH_AUTH_SOCK) → result
                                        │ updates metadata.json counter
                                        │
                                ┌───────▼──────────┐
                                │  metadata.json   │
                                │  + push_enabled   │
                                │  + consecutive_   │
                                │    failures       │
                                └──────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modify | Metadata +2 fields; `sincronizar_checkpoint` rewritten; `system_command` gains SSH env; `cargar_config_remoto`/`guardar_config_remoto` rewritten for per-project; `reintentar_push` rewritten; `GitConfig.remote` removed; `crear_proyecto` seeds push fields; `do_checkpoint` logs errors; new tests |
| `src/lib/tauri.ts` | Modify | `cargarConfigRemoto` and `guardarConfigRemoto` add `projectPath` param; return types adjust |
| `src/lib/components/ProjectSettingsDialog.svelte` | Modify | `loadRemote` calls `cargarConfigRemoto(projectPath)`; `saveRemote` calls `guardarConfigRemoto(projectPath, ...)`; URL pre-fill from backend result instead of global config |
| `src/lib/components/GitIdentityDialog.svelte` | Modify | `guardarConfigRemoto` calls gain `projectPath` param |
| `src/routes/+page.svelte` | Modify | `actualizarGitStatus` calls `cargarConfigRemoto(path)` instead of no-args; `reintentarPush` already passes path |

## Interfaces / Contracts

### Metadata struct (Rust)

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    project_name: String,
    last_modified: String,
    chapters_order: Vec<String>,
    characters_index: Vec<CharacterIndex>,
    // ... existing fields ...
    #[serde(default)]
    places_index: Vec<LugarIndexItem>,
    #[serde(default = "default_font_family")]
    font_family: String,
    // ── NEW ──────────────────────────────────
    #[serde(default)]
    push_enabled: bool,           // default: false
    #[serde(default)]
    consecutive_failures: u32,    // default: 0
}
```

### GitConfig struct (trimmed)

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GitConfig {
    schema_version: u32,
    #[serde(default)]
    identity: Option<GitIdentity>,
    // remote field REMOVED — serde silently ignores it on old files
}
```

### `system_command` SSH inheritance

```rust
fn system_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(std::process::Stdio::null());
    // Inherit SSH agent socket for git operations
    if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") {
        cmd.env("SSH_AUTH_SOCK", sock);
    }
    #[cfg(target_os = "windows")]
    { cmd.creation_flags(0x08000000); }
    cmd
}
```

### Tauri command signature changes

| Command | Old signature | New signature |
|---------|--------------|---------------|
| `cargar_config_remoto` | `(app)` | `(app, proyecto_path: String)` |
| `guardar_config_remoto` | `(app, url, push_enabled)` | `(app, proyecto_path, url, push_enabled)` |
| `sincronizar_checkpoint` | `(app, path)` — reads global config | `(app, path)` — reads metadata.json + git remote |

### TypeScript bindings

```typescript
// Old
cargarConfigRemoto(): Promise<{url, push_enabled, consecutive_failures} | null>
guardarConfigRemoto(url: string, pushEnabled: boolean): Promise<string>

// New
cargarConfigRemoto(projectPath: string): Promise<{push_enabled: boolean, consecutive_failures: number} | null>
guardarConfigRemoto(projectPath: string, url: string, pushEnabled: boolean): Promise<string>
```

Note: `cargarConfigRemoto` no longer returns `url` — the frontend gets the URL from the separate `git remote get-url origin` call done by `configurar_remoto` / `actualizarGitStatus`.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| **Unit — Metadata serde** | Old metadata.json (no push fields) deserializes with defaults; round-trip preserves new fields | New `#[test]` in existing `#[cfg(test)] mod tests` |
| **Unit — GitConfig migration** | Old config with `remote` key deserializes cleanly (unknown key ignored); write-back outputs identity-only JSON | New test: write old-format JSON, read with new struct, verify identity preserved and remote absent |
| **Unit — 3-strike logic** | Counter transitions on success/failure/disable | Existing tests adapted to operate on `Metadata` fields instead of `GitRemoteConfig` |
| **Unit — SSH env** | `system_command` sets `SSH_AUTH_SOCK` when env var present | New test: spawn command, verify env var inherited |
| **Integration — per-project isolation** | Push state in project A doesn't affect project B | New test: two projects, set push_enabled on A, verify B stays false |
| **Integration — metadata round-trip** | Read-modify-write of metadata preserves push fields | Extend existing `actualizar_fuente_proyecto` test pattern |
| **Existing tests** | All 84 tests must still pass | Run `cargo test --manifest-path src-tauri/Cargo.toml` |

## Migration / Rollout

**No explicit migration code needed.** The `remote` key is dropped from `GitConfig` struct. Serde ignores unknown fields during deserialization (`deny_unknown_fields` is not set). When `guardar_identidad_git` performs its read-modify-write cycle, the old `remote` key is naturally stripped on write-back. The first time a user saves their identity after this change, the global config is cleaned.

Likewise, old `metadata.json` files without `push_enabled`/`consecutive_failures` deserialize with defaults (`false`/`0`) thanks to `#[serde(default)]`.

## Open Questions

None. All design decisions resolved.

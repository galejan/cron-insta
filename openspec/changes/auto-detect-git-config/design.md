# Design: Auto-Detect Git Config

## Technical Approach

Add a `detectar_config_git(path)` Tauri command that runs `git config user.name`, `git config user.email`, and `git remote get-url origin` inside the project directory. All three calls are best-effort — no `.git` returns all `null` fields, never errors. Call this asynchronously from the frontend after `cargarIndice` succeeds (in both `abrirProyecto` and auto-reopen flows). If identity differs from stored config, auto-save it. If remote is SSH and not blocked by the 3-strike rule, auto-enable push.

## Architecture Decisions

| Decision | Option | Tradeoff | Choice |
|----------|--------|----------|--------|
| **Return type** | `Result<String, String>` (serialized JSON like `cargar_config_remoto`) | vs struct-return: matches existing command pattern, simple | Serialized JSON — follows existing convention (`cargar_identidad_git`, `cargar_config_remoto`) |
| **Rust error handling** | Best-effort — never `Err` | vs early-return on missing `.git`: simpler caller contract, less code | Best-effort — spec says "never errors", matches `detectar_git` philosophy |
| **SSH check location** | Frontend (`startsWith("git@") \|\| startsWith("ssh://")`) | vs Rust: keeps auto-detection policy in UI layer, simpler Rust command | Frontend — Rust returns raw data, policy lives in UI where toast/guard logic already resides |
| **Identity comparison** | Compare `name+email` from detected vs stored `cargarIdentidadGit` | vs always-save: respects existing user-set identity, no unnecessary writes | Compare — only update when values differ |
| **Consecutive failures guard** | Call `cargarConfigRemoto(path)` to read current `consecutive_failures`, only set push_enabled if `=== 0` | vs add field to `detectar_config_git` return: keeps Rust command stateless, reuses existing infrastructure | Call `cargarConfigRemoto` — minimal addition, Rust command stays focused on detection |

## Data Flow

```
abrirProyecto / auto-reopen
    │
    ├── cargarIndice(path) ──→ success
    │       │
    │       ├── actualizarGitStatus(path)
    │       └── detectarConfigGit(path)    ← async, fire-and-forget
    │               │
    │               ▼
    │       ┌─────────────────────────────┐
    │       │  detectar_config_git (Rust) │
    │       │  → git config user.name     │
    │       │  → git config user.email    │
    │       │  → git remote get-url       │
    │       │  → {name?, email?, url?}    │
    │       └──────────┬──────────────────┘
    │                  ▼
    │       ┌─────────────────────────────────┐
    │       │  Frontend detection handler     │
    │       │  1. compare identity → save     │
    │       │  2. check SSH + failures → push │
    │       │  3. show toast                  │
    │       └─────────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modify | Add `detectar_config_git` command + `GitDetectedConfig` struct + handler registration |
| `src/lib/tauri.ts` | Modify | Add `detectarConfigGit()` binding + `GitDetectedConfig` interface |
| `src/routes/+page.svelte` | Modify | Import binding, call after `cargarIndice` in `abrirProyecto` and auto-reopen; implement detection handler |
| `src/lib/i18n.svelte.ts` | Modify | Add `git.detected` and `git.detectedOrigin` toast labels (es + en) |

## Interfaces / Contracts

**Rust struct** — follows existing single-module convention:
```rust
#[derive(Serialize)]
struct GitDetectedConfig {
    name: Option<String>,
    email: Option<String>,
    remote_url: Option<String>,
}
```

**Rust command** — registered via `generate_handler![]`:
```rust
#[tauri::command]
fn detectar_config_git(project_path: String) -> GitDetectedConfig { ... }
```

**TS binding** — follows existing `invoke` pattern:
```typescript
export interface GitDetectedConfig {
  name?: string;
  email?: string;
  remote_url?: string;
}
export async function detectarConfigGit(path: string): Promise<GitDetectedConfig> {
  return invoke("detectar_config_git", { projectPath: path });
}
```

**i18n keys** (es + en):
- `git.detected`: `"Git detectado: origin → {repo}, identidad → {name} <{email}>"`
- `git.detectedOrigin`: `"Git detectado: origin → {repo}"`

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | `detectar_config_git` with real git repo, no-git dir, partial config | Add `#[cfg(test)] mod tests` — git identity is cheap to set/read locally |
| Unit (Rust) | No `.git` → all `None` | Simple path test, no git binary needed |
| Manual (UI) | Full flow: open cloned repo → toast with identity + origin | Existing manual verification pattern |

## Migration / Rollout

No migration required. New command is additive — existing identity/config flows are unchanged. The 3-strike guard ensures push is never auto-enabled on broken remotes.

## Open Questions

None.

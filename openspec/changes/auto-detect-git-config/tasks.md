# Tasks: Auto-Detect Git Config

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~110 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Rust — detectar_config_git command

- [x] 1.1 `src-tauri/src/lib.rs`: Add `GitDetectedConfig` struct (derive Serialize, Option fields: name, email, remote_url) near existing structs (~L47)
- [x] 1.2 `src-tauri/src/lib.rs`: Implement `detectar_config_git(project_path)` command — run `git config user.name`, `git config user.email`, `git remote get-url origin` inside project dir via `system_command()`. Best-effort: missing .git or partial config returns `None` fields, never `Err`
- [x] 1.3 `src-tauri/src/lib.rs`: Register `detectar_config_git` in the `generate_handler![]` macro (~L2735)

## Phase 2: TypeScript binding

- [x] 2.1 `src/lib/tauri.ts`: Add `GitDetectedConfig` interface and `detectarConfigGit(path)` async binding, following existing `invoke` + `JSON.parse` pattern (~L273)

## Phase 3: Frontend — auto-detect on open

- [x] 3.1 `src/routes/+page.svelte`: Import `detectarConfigGit`, `cargarIdentidadGit`, `guardarIdentidadGit`, `cargarConfigRemoto` (as needed for handler)
- [x] 3.2 `src/routes/+page.svelte`: Implement `detectarYFusionarConfigGit(path)` handler — call `detectarConfigGit`, compare identity (name+email) with stored via `cargarIdentidadGit`, auto-save if different. Check remote: if SSH (`git@` or `ssh://`) AND `consecutive_failures === 0`, set `push_enabled: true` via `guardarConfigRemoto`. Build toast from `git.detected` or `git.detectedOrigin` keys
- [x] 3.3 `src/routes/+page.svelte`: Call handler (fire-and-forget, after `cargarIndice` success block) in `abrirProyecto()` (~L887)
- [x] 3.4 `src/routes/+page.svelte`: Call handler (fire-and-forget, after `cargarIndice` success block) in auto-reopen `$effect` (~L1511)

## Phase 4: i18n — toast labels

- [x] 4.1 `src/lib/i18n.svelte.ts`: Add `git.detected` key (es: `"Git detectado: origin → {repo}, identidad → {name} <{email}>"`, en: `"Git detected: origin → {repo}, identity → {name} <{email}>"`)
- [x] 4.2 `src/lib/i18n.svelte.ts`: Add `git.detectedOrigin` key (es: `"Git detectado: origin → {repo}"`, en: `"Git detected: origin → {repo}"`)

## Phase 5: Tests — Rust unit tests

- [x] 5.1 `src-tauri/src/lib.rs`: Test full config — init git repo with user.name, user.email, and origin remote; verify all three fields populated
- [x] 5.2 `src-tauri/src/lib.rs`: Test missing remote — repo with identity but no origin; verify `remote_url: None`
- [x] 5.3 `src-tauri/src/lib.rs`: Test no .git directory — non-repo path; verify all fields `None`, no error
- [x] 5.4 `src-tauri/src/lib.rs`: Test partial identity — repo with only user.name, no email; verify name populated, email `None`

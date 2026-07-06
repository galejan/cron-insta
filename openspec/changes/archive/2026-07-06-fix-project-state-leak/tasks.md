# Tasks: Fix Project State Leak Between Projects

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 35–40 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr-default |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Flush session + reset leaked state | PR 1 | All 3 files; ~35 lines; fits within 400-line budget |

## Phase 1: Rust — Flush Session on Project Deactivation

- [x] 1.1 Modify `set_active_project` in `src-tauri/src/commands/project.rs`:
  - Before `*active = path;`, capture `old_path` from `active_project`.
  - When `path.is_none()` AND old path exists: lock `session_tracker`, if `start_time.is_some()` call `finalizar_sesion_escritura(&mut tracker, Path::new(&old))`, else `*tracker = SessionTracker::default()`.
  - Add `use std::path::Path;` import if missing.
  - **Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` passes.

## Phase 2: Frontend — Add `sessionActive` Flag

- [x] 2.1 Declare `let sessionActive = $state(false);` near line 290 in `src/routes/+page.svelte` (alongside existing editor state variables).
- [x] 2.2 Set `sessionActive = true;` after `iniciarSesionEscritura` succeeds in `cargarCapituloActual` (line 748).

## Phase 3: Frontend — Warning Dialog on Active Session

- [x] 3.1 In `cerrarProyecto()` (line 1400), add at top before `guardarCapitulo`: `if (sessionActive) { const ok = await ask(t("session.closeWarning")); if (!ok) return; }`
  - `ask` is already imported from `@tauri-apps/plugin-dialog` (line 84).

## Phase 4: Frontend — Reset Leaked $state Variables

- [x] 4.1 In `cerrarProyecto()`, after `mediaFiles = [];` (line 1436), add resets for:
  `projectStats`, `noteDocked`, `placeDocked`, `mediaDocked`, `mediaViewer`, `gitLogEntries`, `gitLogVisible`, `dragId`, `dragChapter`, `dragOverTrama`, `cercaDelFinal`, `sessionActive`.
  - Per spec: null for docked/viewer/drag, `[]` for gitLogEntries, `false` for gitLogVisible/cercaDelFinal/sessionActive, `{total_sessions:0,total_hours:0,total_words:0}` for projectStats.

## Phase 5: i18n — Add Warning Dialog Key

- [x] 5.1 In `src/lib/i18n.svelte.ts`, add `"session.closeWarning"` key in the `es` block (alphabetically, near `"session.*"` keys):
  `"session.closeWarning": "Tu sesión de escritura actual se finalizará. ¿Continuar?",`
- [x] 5.2 Add same key in the `en` block:
  `"session.closeWarning": "Your current writing session will be finalized. Continue?",`

## Phase 6: Verification — Manual Test Scenarios

- [x] 6.1 **Flush on switch**: Open project A, start session, close project → open project B → verify B's `stats.json` has only B's data.
- [x] 6.2 **Warning dialog shown**: Start session → click Close → verify dialog appears → cancel → verify project stays open.
- [x] 6.3 **Warning dialog confirmed**: Start session → click Close → confirm dialog → verify project closes and session is flushed.
- [x] 6.4 **No dialog when inactive**: Without starting session → click Close → verify no dialog, project closes immediately.
- [x] 6.5 **Double-flush safety**: Close project (flushes) → close window → verify `do_checkpoint` doesn't duplicate stats.
- [x] 6.6 **Existing tests pass**: Run `cargo test --manifest-path src-tauri/Cargo.toml` — all tests green.

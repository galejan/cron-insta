# Design: Fix Project State Leak Between Projects

## Technical Approach

Two-layer fix: Rust flushes unsaved session stats on `set_active_project(None)`, and the frontend resets 10 leaked `$state` variables in `cerrarProyecto()` with a confirmation dialog before close when a session is active.

## Architecture Decisions

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Flush on `set_active_project(None)` | Adds I/O to project switch, but prevents data loss | **Chosen**: best-effort flush — blocking is acceptable for project transitions |
| Flush via separate Tauri command | Avoids coupling commands, but requires frontend to remember to call it | Rejected: frontend-optional approach is the leak we're fixing |
| Custom Svelte dialog | Full control over styling, but adds component complexity | Rejected: codebase already uses `ask()` from `@tauri-apps/plugin-dialog` (native OS dialog) for all confirmations |
| Use `ask()` for warning | Consistent with existing patterns; native look-and-feel | **Chosen**: no new components, no new CSS |

## Data Flow

```
User clicks "Close" → cerrarProyecto()
  ├─ sessionActive? → ask("session.closeWarning")
  │   ├─ No → return (project stays open)
  │   └─ Yes → continue
  ├─ guardarCapitulo (save unsaved editor content)
  ├─ setActiveProject(null) → Rust side:
  │     ├─ capture old_path from active_project
  │     ├─ lock session_tracker
  │     ├─ start_time.is_some()? → finalizar_sesion_escritura(tracker, old_path)
  │     │     └─ write stats.json → git commit → reset tracker to default
  │     └─ start_time.is_none()? → reset tracker to default
  ├─ clear 10 leaked $state variables
  └─ sessionActive = false
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/commands/project.rs` | Modify | `set_active_project`: flush session before clearing path |
| `src/routes/+page.svelte` | Modify | `cerrarProyecto`: reset 10 vars + `sessionActive` + warning dialog; set `sessionActive=true` in `cargarCapituloActual` |
| `src/lib/i18n.svelte.ts` | Modify | Add `session.closeWarning` key (es + en) |

## Implementation Details

### Rust: `set_active_project` (line 166)

Capture `old_path` from `active_project` BEFORE mutation. If `path.is_none()` and old path exists:

1. Lock `session_tracker`
2. If `start_time.is_some()`: call `finalizar_sesion_escritura(&mut tracker, Path::new(old))` — flushes chapter_times + word diffs to `old/.config/stats.json`, then resets tracker to `default()` via line 246 of stats.rs
3. If no session: `*tracker = SessionTracker::default()`
4. Then set `*active = path` (via second lock acquisition)

**Double-flush safety**: After flush, `tracker.start_time` is `None`. When `do_checkpoint` on window close calls `finalizar_sesion_escritura` again, the guard at stats.rs:153-155 returns immediately. Locks are separate mutexes — no deadlock.

### Frontend: `cerrarProyecto()` (line 1400)

1. **New `$state` variable** (line ~290, near other state): `let sessionActive = $state(false);`
2. **Set on session start** (in `cargarCapituloActual`, after `iniciarSesionEscritura` succeeds at line 747): `sessionActive = true;`
3. **Warning dialog** (at top of `cerrarProyecto`, before `setActiveProject(null)`): `if (sessionActive) { const ok = await ask(t("session.closeWarning")); if (!ok) return; }`
4. **Reset 10 variables** in the cleanup section (after `mediaFiles = [];`):
   - `projectStats = { total_sessions: 0, total_hours: 0, total_words: 0 };`
   - `noteDocked = null;`
   - `placeDocked = null;`
   - `mediaDocked = null;`
   - `mediaViewer = null;`
   - `gitLogEntries = [];`
   - `gitLogVisible = false;`
   - `dragId = null;`
   - `dragChapter = null;`
   - `dragOverTrama = null;`
   - `cercaDelFinal = false;`
   - `sessionActive = false;`

### i18n Keys

| Key | ES | EN |
|-----|----|----|
| `session.closeWarning` | "Tu sesión de escritura actual se finalizará. ¿Continuar?" | "Your current writing session will be finalized. Continue?" |

Keys placed alphabetically in both language blocks of `src/lib/i18n.svelte.ts`.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (Rust) | `set_active_project(None)` flushes session | Existing `cargo test` suite |
| Unit (Rust) | `set_active_project(None)` no-op when no session | Existing `cargo test` suite |
| Manual | Project A session → switch → Project B stats.json clean | Open project A, start session, close, open project B, verify B's stats.json |
| Manual | Warning dialog appears/stops close | Open project, start session, click Close → verify dialog; cancel → verify project still open |
| Manual | Double-flush does not corrupt | Close project (flushes), close window → verify `do_checkpoint` doesn't duplicate stats |

## Migration / Rollout

No migration required. Existing `stats.json` files untouched. Rollback: revert `set_active_project` to 2-line original, remove frontend resets and dialog.

## Open Questions

None.

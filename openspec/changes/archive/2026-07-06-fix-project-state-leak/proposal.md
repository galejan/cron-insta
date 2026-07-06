# Proposal: Fix Project State Leak Between Projects

## Intent

When switching projects without restarting, Rust's `SessionTracker` and 10 frontend `$state` variables retain data from the previous project — writing Project A's chapter names and times into Project B's `stats.json`.

## Scope

### In Scope
- **Rust**: Reset `SessionTracker` in `set_active_project(None)`, flushing unsaved session data before clearing
- **Frontend**: Reset 8 leaked `$state` variables in `cerrarProyecto()`: `projectStats`, `noteDocked`, `placeDocked`, `mediaDocked`, `mediaViewer`, `gitLogEntries`, `gitLogVisible`, drag state (`dragId`, `dragChapter`, `dragOverTrama`)
- **UX**: Warning dialog before project close/switch when a writing session is active

### Out of Scope
- Auto-repair of already-corrupted `stats.json` (existing Ctrl+Shift+R handles this)
- Multi-window project isolation (single-window app today)
- Direct project-to-project switch without going through `set_active_project(None)` (frontend always goes through null)

## Capabilities

### New Capabilities
None

### Modified Capabilities
- `session-stats`: Session finalization (writing to `stats.json`) now triggered on `set_active_project(None)`, not solely on window close via `do_checkpoint`

## Approach

**Rust fix** (primary): In `set_active_project`, before overwriting `active_project`, capture the old path. If `path` is `None` and a session is active (`start_time.is_some()`), call `finalizar_sesion_escritura()` to flush accumulated `chapter_times` and word diffs to the departing project's `stats.json`, then reset the tracker. `finalizar_sesion_escritura` already resets the tracker to `default()` at completion (line 246 in stats.rs).

**Frontend fix**: In `cerrarProyecto()`, add reset for all 8 leaked variables. Before closing, check for active session and show a confirmation dialog: "Tu sesion de escritura actual se finalizara. ¿Continuar?"

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/commands/project.rs` | Modified | `set_active_project`: flush session before clearing path |
| `src/routes/+page.svelte` | Modified | `cerrarProyecto`: reset 8 vars + warning dialog |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Double-flush if user closes window after switching projects | Low | `start_time` is `None` after reset → `finalizar_sesion_escritura` returns immediately (guard at line 155) |
| Lock contention between `active_project` and `session_tracker` | Low | Separate mutexes; `finalizar_sesion_escritura` never locks `active_project` |

## Rollback Plan

Revert `set_active_project` to only clear path (2 lines). Revert frontend state resets. No data migration — old `stats.json` files unchanged.

## Dependencies

None. `finalizar_sesion_escritura` is already `pub` in `commands/stats.rs`.

## Success Criteria

- [ ] Switching projects mid-session does not leak chapter names or times between `stats.json` files
- [ ] All 10 leaked frontend variables are null/empty after `cerrarProyecto()`
- [ ] Warning dialog appears when closing with active session
- [ ] Existing Rust tests pass (`cargo test --manifest-path src-tauri/Cargo.toml`)
- [ ] No regression: `do_checkpoint` on window close still works correctly

## Product Decisions (from Q&A round)

| # | Decision |
|---|----------|
| P1 | Flush unsaved session data to `stats.json` before resetting tracker |
| P2 | Show warning dialog before close when session is active |
| P3 | Prevent future corruption only; already-affected projects use Ctrl+Shift+R |
| P4 | Session stats are lower priority than editor content; flush is best-effort |

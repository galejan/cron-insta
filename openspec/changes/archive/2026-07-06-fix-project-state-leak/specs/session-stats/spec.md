# Delta for session-stats

## ADDED Requirements

### Requirement: SessionTracker Reset on Project Deactivation

The system MUST call `finalizar_sesion_escritura` when `set_active_project(None)` is invoked with an active session, flushing data to the departing project's `stats.json` before resetting the tracker to `SessionTracker::default()`.

#### Scenario: Session flushed on project switch

- GIVEN project A has an active session with a current chapter
- WHEN `set_active_project(None)` is called (project switch)
- THEN `finalizar_sesion_escritura` writes chapter times and word diffs to project A's `stats.json`
- AND the `SessionTracker` is reset to `default()`

#### Scenario: No session to flush on deactivation

- GIVEN a project is open but no session was started (`start_time` is `None`)
- WHEN `set_active_project(None)` is called
- THEN `finalizar_sesion_escritura` returns immediately without writing to `stats.json`

#### Scenario: Double-flush safety after project switch

- GIVEN session was flushed during project switch (tracker = default)
- WHEN window closes and `do_checkpoint` invokes `finalizar_sesion_escritura` again
- THEN the guard on `start_time.is_none()` causes immediate return
- AND no duplicate session record is written

### Requirement: Frontend State Cleanup on Project Close

The system MUST reset 10 `$state` variables in `cerrarProyecto()`: `projectStats`, `noteDocked`, `placeDocked`, `mediaDocked`, `mediaViewer`, `gitLogEntries`, `gitLogVisible`, `dragId`, `dragChapter`, `dragOverTrama`.

#### Scenario: No state leakage after project close

- GIVEN project A open with docked note, media viewer, visible git log, active drag state
- WHEN user closes project A
- THEN all 10 variables are null/empty/default
- AND opening project B shows only project B's data

#### Scenario: Clean close with no active panels

- GIVEN project A with none of the leaked variables currently in use
- WHEN user closes the project
- THEN the reset assignments are no-ops

### Requirement: Active-Session Warning Dialog

The system MUST show a confirmation dialog before project close when a writing session is active.

#### Scenario: Dialog shown when session is active

- GIVEN an active writing session (timer started)
- WHEN user triggers project close via toolbar, shortcut, or new-project action
- THEN a confirmation dialog appears warning the session will be finalized
- AND close proceeds only after user confirmation

#### Scenario: No dialog when session is inactive

- GIVEN no active session (timer never started)
- WHEN user triggers project close
- THEN no dialog is shown; close proceeds immediately

#### Scenario: User cancels the dialog

- GIVEN warning dialog is visible
- WHEN user cancels
- THEN project remains open; session continues uninterrupted

## MODIFIED Requirements

### Requirement: Stats Accumulation

On session close OR project deactivation, the system MUST compute duration as `now - session_start`, compute `words_added` per chapter as `current_words - initial_words`, and accumulate into `SessionStats`. Chapter keys SHALL be filenames.
(Previously: accumulation only on session close, not project deactivation)

#### Scenario: First session populates empty stats

- GIVEN empty `stats.json`, chapter "0001.md" 0→150 words, 600s elapsed
- WHEN session closes
- THEN `total_time_seconds: 600`, `total_words: 150`, one session record

#### Scenario: Subsequent session accumulates

- GIVEN `total_time_seconds: 600, total_words: 150`
- WHEN second session adds 300s, 100 words
- THEN `total_time_seconds: 900, total_words: 250`, two records

#### Scenario: Deleted chapter between open and close

- GIVEN "0002.md" deleted before close
- WHEN `word_count_chapter` fails for it
- THEN chapter skipped with warning; remaining stats recorded normally

#### Scenario: New chapter added mid-session

- GIVEN "0003.md" created after session start (no initial snapshot)
- WHEN session closes
- THEN `words_added` = full word count, accumulated normally

#### Scenario: Session flushed on project switch

- GIVEN project A: active session, 300s, chapter "0001.md" 0→100 words
- WHEN user switches to project B (triggers deactivation)
- THEN project A's `stats.json` updated; project B starts with fresh tracker

### Requirement: Auto-Commit on Close

The system MUST auto-commit `stats.json` after any stats persistence (close or deactivation) using `"cron-insta: actualizar estadísticas de sesión"`. Best-effort — failure MUST NOT block the flow.
(Previously: scoped only to window close; now also covers project deactivation)

#### Scenario: Stats commit succeeds

- GIVEN `stats.json` updated during persistence
- WHEN stats collection completes
- THEN git add + commit executed and succeeds

#### Scenario: Stats commit fails gracefully

- GIVEN git commit fails (no repo)
- WHEN commit attempted
- THEN error logged; flow continues uninterrupted

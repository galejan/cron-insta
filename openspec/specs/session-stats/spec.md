# session-stats Specification

## Purpose

Tracks writer productivity across sessions: elapsed time, net word count per chapter, and cumulative totals. Persisted to `.config/stats.json`.

## Requirements

### Requirement: Session Timer

The system MUST start a timer when the user opens a chapter in a project. Only one active timer SHALL exist per project. Switching chapters within the same project MUST continue the timer without resetting. A second `iniciar_sesion_escritura` call for the same project SHALL reset the timer to the current instant.

#### Scenario: Timer starts on project open

- GIVEN no active project session
- WHEN the user opens a chapter in a project
- THEN the session timer begins recording elapsed time from that instant

#### Scenario: Timer continues across chapter switches

- GIVEN an active session with timer running at 300s elapsed
- WHEN the user opens a different chapter in the same project
- THEN the timer continues accumulating from 300s without reset

#### Scenario: Timer restart on concurrent open

- GIVEN an active session with timer at 300s
- WHEN `iniciar_sesion_escritura` is called again for the same project
- THEN the timer resets to 0 (current instant)

### Requirement: Word Counting

The system MUST provide per-chapter word counts by stripping HTML tags from TipTap markdown content and counting remaining text words delimited by Unicode whitespace boundaries. Markdown formatting characters (headings, emphasis markers, list bullets) that remain after HTML-stripping SHALL be treated as text.

#### Scenario: Strips HTML and counts text words

- GIVEN chapter file contains `<p>Hola <strong>mundo</strong></p>`
- WHEN word count is computed
- THEN result is 2 (Hola, mundo)

#### Scenario: Empty chapter returns zero

- GIVEN an empty chapter file, or file containing only HTML tags with no text content
- WHEN word count is computed
- THEN result is 0

#### Scenario: Chapter with mixed markdown and HTML

- GIVEN `# Título\n<p>Texto del <em>capítulo</em>.</p>`
- WHEN word count is computed
- THEN HTML stripped, remaining text words counted: Título, Texto, del, capítulo = 4

### Requirement: Stats Accumulation

On session close OR project deactivation, the system MUST compute duration as `now - session_start`, compute `words_added` per chapter as `current_words - initial_words` (snapshot taken at session start), and accumulate into the `SessionStats` struct. Chapter keys SHALL be filenames (e.g. `"0001_prologo.md"`).

#### Scenario: First session populates empty stats

- GIVEN empty `stats.json`, chapter "0001.md" with 0→150 words, 600s elapsed
- WHEN session closes
- THEN `total_time_seconds: 600`, `total_words: 150`, chapter `"0001.md": {time_seconds: 600, words: 150}`, one session record with `date`, `duration_seconds: 600`, `words_added: 150`, `chapter_id: "0001.md"`

#### Scenario: Subsequent session accumulates

- GIVEN `total_time_seconds: 600, total_words: 150`
- WHEN a second session adds 300s and 100 words to "0001.md"
- THEN `total_time_seconds: 900, total_words: 250`, two session records

#### Scenario: Deleted chapter between open and close

- GIVEN chapter "0002.md" existed at session start but was deleted before close
- WHEN session closes and `word_count_chapter("0002.md")` fails
- THEN the missing chapter is skipped with a warning log; stats for remaining chapters are recorded normally

#### Scenario: New chapter added mid-session

- GIVEN chapter "0003.md" was created after session start (no initial snapshot)
- WHEN session closes
- THEN its `words_added` is its full word count, accumulated normally

#### Scenario: Session flushed on project switch

- GIVEN project A: active session, 300s, chapter "0001.md" 0→100 words
- WHEN user switches to project B (triggers deactivation)
- THEN project A's `stats.json` updated; project B starts with fresh tracker

### Requirement: Stats Persistence

The system MUST read and write `stats.json` from `.config/` within the project directory. On read failure (missing or corrupt file), the system SHALL initialize an empty `SessionStats` and proceed without error.

#### Scenario: Missing stats.json auto-initializes

- GIVEN `.config/stats.json` does not exist
- WHEN stats are loaded for accumulation
- THEN an empty `SessionStats` struct ({total_time_seconds: 0, total_words: 0, chapters: {}, sessions: []}) is returned

#### Scenario: Corrupt stats.json recovers gracefully

- GIVEN `stats.json` contains invalid JSON
- WHEN stats are loaded
- THEN the error is logged, an empty `SessionStats` is returned, and the session proceeds

### Requirement: Auto-Commit on Close

The system MUST auto-commit `stats.json` after any stats persistence (close or deactivation) using message `"cron-insta: actualizar estadísticas de sesión"`. This commit SHALL be best-effort — failure MUST NOT block the flow.

#### Scenario: Stats commit succeeds

- GIVEN `stats.json` updated during persistence
- WHEN stats collection completes
- THEN git add + commit executed and succeeds

#### Scenario: Stats commit fails gracefully

- GIVEN git commit fails (no repo)
- WHEN commit attempted
- THEN error logged; flow continues uninterrupted

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

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

On session close, the system MUST compute duration as `now - session_start`, compute `words_added` per chapter as `current_words - initial_words` (snapshot taken at session start), and accumulate into the `SessionStats` struct. Chapter keys SHALL be filenames (e.g. `"0001_prologo.md"`).

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

The system MUST auto-commit `stats.json` after updating it during session close, using message `"cron-insta: actualizar estadísticas de sesión"`. This commit SHALL be best-effort — failure MUST NOT block the close flow.

#### Scenario: Stats commit succeeds

- GIVEN stats.json was updated with new session data during `do_checkpoint()`
- WHEN the stats collection phase completes
- THEN `git add .config/stats.json && git commit -m "cron-insta: actualizar estadísticas de sesión"` is executed and succeeds

#### Scenario: Stats commit fails gracefully

- GIVEN git commit of stats.json fails (e.g., no git repo initialized)
- WHEN the commit is attempted
- THEN the error is logged; the close flow continues to sync without interruption

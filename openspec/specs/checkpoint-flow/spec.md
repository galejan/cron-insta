# checkpoint-flow Specification

## Purpose

Defines the project close sequence: checkpoint commit, session stats collection, and remote sync. Runs entirely in Rust to avoid frontend deadlocks during close.

## Requirements

### Requirement: Checkpoint Execution Order

The system MUST execute the close sequence in a fixed order: commit all changes, collect and persist session stats, then sync with remote. Each phase SHALL complete fully before the next begins. Stats collection failure MUST NOT block the sync phase.

#### Scenario: Full close flow succeeds

- GIVEN an active project with pending changes and session stats
- WHEN `do_checkpoint()` is invoked
- THEN `git add . && git commit` runs first
- AND session stats are computed and written to `stats.json`
- AND stats.json is committed with message `"cron-insta: actualizar estadísticas de sesión"`
- AND `git push origin main` runs last

#### Scenario: Stats collection failure does not block sync

- GIVEN stats.json write fails (e.g., disk full)
- WHEN `do_checkpoint()` reaches the stats collection phase
- THEN the error is logged
- AND the sync phase (`git push`) executes normally

### Requirement: Session Stats Collection During Close

During `do_checkpoint()`, the system SHALL collect session statistics by computing elapsed time since session start, taking a word count snapshot of all current chapters, calculating the word diff against initial snapshots, updating `SessionStats` accumulators in `stats.json`, and committing the updated file.

#### Scenario: Stats collected and committed

- GIVEN session started at T0, chapter "0001.md" had 0 words at start and 150 words at close, elapsed 600s
- WHEN the stats collection phase runs
- THEN `stats.json` reflects `total_time_seconds += 600`, `total_words += 150`, chapter `"0001.md": {time_seconds: 600, words: 150}`, and a new session record is appended

#### Scenario: No active session skips stats

- GIVEN no session timer was started (session_start is None)
- WHEN `do_checkpoint()` reaches the stats phase
- THEN stats collection is skipped entirely; no `stats.json` write or commit occurs

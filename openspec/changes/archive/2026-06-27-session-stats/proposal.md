# Proposal: Session Statistics Tracking

## Intent

Writers have no visibility into their productivity — how long they write or how many words they produce per session. Adding automatic session stats tracking (time + word count) gives writers objective feedback on their writing habits, mirroring features in established writing tools.

## Scope

### In Scope
- Session timer: elapsed time between chapter open and project close
- Word count diff: net words added per session (snapshot at open vs. close)
- Per-chapter and global accumulators in `.config/stats.json`
- Auto-commit stats.json on project close via `do_checkpoint()`
- Seed `.config/stats.json` on new project creation

### Out of Scope
- Stats UI/dashboard (future), idle detection, words-per-minute metrics
- Export or external reporting, live word count display during editing

## Capabilities

### New Capabilities
- `session-stats`: Session duration and word count tracking, persisted to `.config/stats.json` on project close. Includes per-chapter and global accumulators plus session history.

### Modified Capabilities
- `project-file-management`: "Project Folder Creation" requirement — SHALL also seed `.config/stats.json` with initial accumulator structure (`{ total_time_seconds: 0, total_words: 0, chapters: {}, sessions: [] }`).

## Approach

Compute everything in Rust during `do_checkpoint()` — the existing close handler that already runs git commit + sync. No frontend IPC during close (avoids deadlock risk).

1. **Session start**: `ProjectState` gains `session_start: Instant`, set when `set_active_project()` receives a path
2. **Word counting**: New `count_words_per_chapter()` — reads markdown, strips HTML tags, counts text-content words, returns `HashMap<filename, usize>`
3. **Stats structs**: `SessionStats`, `StatsChapter`, `StatsSession` — serialized via serde
4. **On close**: Read session start → compute duration → snapshot words per chapter → compute diff → update accumulators → append session record → write `stats.json` → git add + commit with `"cron-insta: actualizar estadísticas de sesión"`
5. **Chapter key**: filename (e.g. `"0001_prologue.md"`) — matches existing codebase convention, no UUIDs

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` — `ProjectState` | Modified | Add `session_start: Mutex<Option<Instant>>` |
| `src-tauri/src/lib.rs` — `set_active_project()` | Modified | Record session start on project open |
| `src-tauri/src/lib.rs` — `do_checkpoint()` | Modified | Inject stats collection before sync |
| `src-tauri/src/lib.rs` — `count_words_in_chapters()` | Extended | New `count_words_per_chapter()` variant |
| `src-tauri/src/lib.rs` — `crear_proyecto()` | Modified | Seed `.config/stats.json` |
| `.config/stats.json` | New | Accumulated stats file (in-repo) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Chapter deleted between open and close | Low | Skip missing files; log warning, don't fail stats |
| Word count includes HTML tags | Medium | New function strips HTML before counting |
| Concurrent session confusion (reopen same chapter) | Low | Accumulate time in same session; one record per close |

## Rollback Plan

Delete `.config/stats.json` and remove the stats collection block from `do_checkpoint()`. The file is additive-only — no other system depends on it. Revert `crear_proyecto()` seed change. No database migration needed.

## Dependencies

None. Uses existing `do_checkpoint()`, `count_words_in_chapters()`, and `commit_metadata_file()` patterns.

## Success Criteria

- [ ] `stats.json` created on new project with correct seed structure
- [ ] Session duration recorded on close (tolerance ±5s)
- [ ] Word diff matches actual words added (excluding HTML tags)
- [ ] Stats auto-committed with message `"cron-insta: actualizar estadísticas de sesión"`
- [ ] All existing + new `cargo test` pass

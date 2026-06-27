# Design: Session Statistics Tracking

## Technical Approach

Track writer productivity automatically during the Tauri lifecycle: start a session timer when the user opens a chapter, accumulate per-chapter word counts and elapsed time, persist to `.config/stats.json` on project close. Stats collection runs inside `do_checkpoint()` — the existing close handler — so no frontend IPC during close. A new `iniciar_sesion_escritura` command is called from the frontend whenever a chapter loads.

## Architecture Decisions

| ID | Decision | Options | Choice | Rationale |
|----|----------|---------|--------|-----------|
| D1 | Concurrency for SessionTracker | `Mutex<SessionTracker>` in `ProjectState` | **Mutex in ProjectState** | SessionTracker has 5 fields (compound state). Atomic only works for primitives. Project already uses `Mutex` for `active_project` and `closing` — follows existing pattern. Mutations are infrequent (chapter open, close). |
| D2 | HTML tag stripping | Regex crate vs char-by-char state machine vs HTML parser | **Char-by-char state machine** | No `regex` in Cargo.toml — adding a dep just for tag stripping is overkill. Full HTML parser (e.g. scraper) is too heavy. Two-state toggle (inside/outside `<`/`>`) handles TipTap output cleanly. |
| D3 | Corrupt `stats.json` | Regenerate from scratch vs return default vs crash | **Return `SessionStats::default()`** | Spec requirement: "On read failure the system SHALL initialize an empty `SessionStats`." Default struct is `{ total_time_seconds: 0, total_words: 0, chapters: {}, sessions: [] }`. Losing historical data is acceptable — it's additive, not critical. |
| D4 | Chapter switch behavior | Accumulate time in same session vs separate session records | **Same session, per-chapter time tracked separately** | Session timer runs from first chapter open to project close. Chapter switches accumulate elapsed time into a `chapter_times: HashMap<String, u64>` inside `SessionTracker`. One `StatsSession` record per close event. |

## Data Flow

```
FRONTEND                           RUST BACKEND
─────────                          ────────────
cargarCapituloActual()
  │
  ├─ invoke("iniciar_sesion_escritura") ──→ SessionTracker::start_session()
  │                                           ├─ snapshot words in chapter file
  │                                           └─ set start_time + chapter_start
  │
  ... user edits ...
  │
close window
  │
  └─ on_window_event(CloseRequested)
       └─ do_checkpoint()
            ├─ perform_commit()               ← existing
            ├─ finalizar_sesion_escritura()   ← NEW (best-effort)
            │    ├─ elapsed = now - start_time
            │    ├─ chapter diff = recount - initial
            │    ├─ read stats.json (or default)
            │    ├─ accumulate totals + chapter stats
            │    ├─ append session record
            │    ├─ write stats.json
            │    └─ git add + commit stats.json
            └─ sync_with_remote()             ← existing
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/lib.rs` | Modify | Add `StatsChapter`, `StatsSession`, `SessionStats` structs; add `SessionTracker` with `chapter_times` field; add `count_words_in_html()` utility; add `#[tauri::command] iniciar_sesion_escritura`; add internal `finalizar_sesion_escritura()` called from `do_checkpoint()`; seed `stats.json` in `crear_proyecto()`; inject stats collection between commit and sync in `do_checkpoint()`; add tests |
| `src/lib/tauri.ts` | Modify | Add `iniciarSesionEscritura(path, chapterFilename)` IPC binding |
| `src/routes/+page.svelte` | Modify | Call `iniciarSesionEscritura` within `cargarCapituloActual()` after successful chapter load |

## Interfaces / Contracts

### SessionTracker (Rust — added to `ProjectState`)

```rust
struct SessionTracker {
    start_time: Option<std::time::Instant>,  // overall session start
    chapter_start: Option<std::time::Instant>, // current chapter start
    chapter_filename: Option<String>,
    initial_word_count: Option<u64>,
    chapter_times: HashMap<String, u64>,  // filename → accumulated seconds
}

impl SessionTracker {
    fn start_session(&mut self, chapter_filename: &str, initial_words: u64);
    fn end_session(&mut self, project_path: &Path) -> Option<SessionResult>;
    fn reset(&mut self);
}
```

### IPC Commands

| Command | Params | Returns | Caller |
|---------|--------|---------|--------|
| `iniciar_sesion_escritura` | `path: String, chapter_filename: String` | `Result<(), String>` | Frontend: `cargarCapituloActual()` |
| `finalizar_sesion_escritura` (internal) | `project_path: &Path, state: &ProjectState` | `Result<(), String>` (best-effort) | `do_checkpoint()` |

### `count_words_in_html(html: &str) -> u64`

Two-state machine: iterate chars, skip content between `<` and `>`, count Unicode whitespace-delimited tokens from remaining text. No regex dependency.

### `stats.json` Schema

```json
{
  "total_time_seconds": 0,
  "total_words": 0,
  "chapters": {},
  "sessions": []
}
```

Chapter keys: chapter filenames (e.g. `"0001_prologo.md"`). Session date format: `"2026-06-27"`.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `count_words_in_html()` — empty, HTML-only, mixed markdown, entities | `#[test]` in lib.rs tests module |
| Unit | `SessionTracker::start_session()` / `end_session()` — first start, chapter switch accumulation, reset | `#[test]` with `TempDir` |
| Unit | `SessionStats` serialization round-trip — seed file matches default struct | `#[test]` with `serde_json` |
| Unit | `crear_proyecto()` now seeds `stats.json` with correct schema | Extend existing test helper |
| Integration | Full close flow: session start → chapter switch → close → verify `stats.json` | New `#[test]` simulating `do_checkpoint` path |

All tests run via `cargo test --manifest-path src-tauri/Cargo.toml`.

## Migration / Rollout

No migration required. `stats.json` is an additive-only file. Existing projects without it will auto-initialize on first close. Rollback: delete `.config/stats.json` and revert the `do_checkpoint` injection block.

## Open Questions

- [ ] Should `iniciar_sesion_escritura` also be called from the auto-save timer path (`guardarCapitulo`)? Current design only triggers on explicit chapter load. (Doesn't block implementation — can add later.)

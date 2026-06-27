# Tasks: Session Statistics Tracking

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~310 (260 Rust + 5 TS + 5 Svelte + 40 tests) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | auto-forecast |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Stats structs + word counter + SessionTracker | PR 1 | ~250 Rust lines, all backend. Frontend wiring is trivial (~10 lines) and can ride along or be a separate 5-min commit. |

## Phase 1: Data Structures & Word Counter (src-tauri/src/lib.rs)

- [x] 1.1 Add `StatsChapter`, `StatsSession`, `SessionStats` structs with `#[derive(Serialize, Deserialize, Default)]`, placed near existing `Metadata`/`Character` structs
- [x] 1.2 Add `use std::collections::HashMap;` at top of lib.rs if not present
- [x] 1.3 Implement `count_words_in_html(html: &str) -> u64` — two-state machine: skip chars inside `<`…`>`, split remaining text on Unicode whitespace, count tokens
- [x] 1.4 Write unit tests for `count_words_in_html`: empty input, HTML-only, mixed markdown+HTML (`# Título\n<p>Texto del <em>capítulo</em>.</p>`), entity-heavy `<p>&amp; &lt; &gt;</p>`

## Phase 2: SessionTracker (src-tauri/src/lib.rs)

- [x] 2.1 Add `SessionTracker` struct with fields: `start_time: Option<Instant>`, `chapter_start: Option<Instant>`, `chapter_filename: Option<String>`, `initial_word_count: Option<u64>`, `chapter_times: HashMap<String, u64>`
- [x] 2.2 Add `session_tracker: Mutex<SessionTracker>` field to `ProjectState`, default initialized
- [x] 2.3 Implement `#[tauri::command] iniciar_sesion_escritura(path: String, chapter_filename: String)` — reads chapter file, counts words via `count_words_in_html`, accumulates prior chapter time into `chapter_times`, sets `chapter_start`/`chapter_filename`/`initial_word_count`, sets `start_time` if `None`
- [x] 2.4 Implement internal `finalizar_sesion_escritura(tracker: &mut SessionTracker, project_path: &Path)` — computes elapsed total + chapter time, reads current words per chapter via `count_words_in_html`, reads `stats.json` (or `default`), accumulates totals, appends session record, writes updated JSON, runs `git add .config/stats.json && git commit -m "cron-insta: actualizar estadísticas de sesión"`. All errors logged to `eprintln!` only — never block caller

## Phase 3: Integration into Checkpoint & Project Creation (src-tauri/src/lib.rs)

- [x] 3.1 In `crear_proyecto()`, write `stats.json` seed: `{"total_time_seconds":0,"total_words":0,"chapters":{},"sessions":[]}` after timeline.json write
- [x] 3.2 In `do_checkpoint()`, inject `finalizar_sesion_escritura()` between `perform_commit()` and `sync_with_remote()`. Lock `session_tracker` mutex, call the internal function, log any error without blocking sync
- [x] 3.3 Write integration test: start session on chapter "0001.md", simulate `do_checkpoint` path via tmp dir, assert `stats.json` has correct totals, session record, and per-chapter stats

## Phase 4: Frontend Wiring

- [x] 4.1 Add `iniciarSesionEscritura(path: string, chapterFilename: string): Promise<void>` to `src/lib/tauri.ts` (invoke `iniciar_sesion_escritura`)
- [x] 4.2 In `src/routes/+page.svelte`, call `iniciarSesionEscritura(projectPath, filename)` inside `cargarCapituloActual()` after successful chapter load (within the try block, after `editorRef?.setContent(content)`)

## Phase 5: Verification

- [x] 5.1 Run `cargo test --manifest-path src-tauri/Cargo.toml` — all existing + new tests pass
- [x] 5.2 Run `pnpm check` — no TypeScript errors in frontend wiring
- [x] 5.3 Manual smoke: create project → open chapter → close window → verify `.config/stats.json` exists with populated session record

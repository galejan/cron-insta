use chrono::Local;
use std::path::Path;
use crate::models::*;
use crate::utils::*;

/// Start a writing session timer for the given chapter.
///
/// Reads the chapter file, counts words via `count_words_in_html`,
/// and records the start time. If another session was already active,
/// accumulates the previous chapter's elapsed time into `chapter_times`
/// and switches to the new chapter (timer continues).
///
/// The session timer remains active until `do_checkpoint()` calls
/// `finalizar_sesion_escritura` on project close.
#[tauri::command]
pub fn iniciar_sesion_escritura(
    state: tauri::State<ProjectState>,
    path: String,
    chapter_filename: String,
) -> Result<(), String> {
    let mut tracker = state.session_tracker.lock()
        .map_err(|e| format!("Error al acceder al tracker de sesión: {}", e))?;
    let project_path = Path::new(&path);
    // Read current chapter and count words
    let word_count = word_count_chapter(project_path, &chapter_filename);
    // If we were already tracking a chapter, accumulate its elapsed time
    if tracker.chapter_start.is_some() && tracker.chapter_filename.is_some() {
        let ch_file = tracker.chapter_filename.clone().unwrap();
        let ch_start = tracker.chapter_start.unwrap();
        let elapsed = ch_start.elapsed().as_secs();
        let accum = tracker.chapter_times.entry(ch_file).or_insert(0);
        *accum += elapsed;
    }
    // Set or reset session state
    let now = std::time::Instant::now();
    if tracker.start_time.is_none() {
        tracker.start_time = Some(now);
    }
    tracker.chapter_start = Some(now);
    tracker.chapter_filename = Some(chapter_filename);
    tracker.initial_word_count = Some(word_count);
    Ok(())
}
/// Read session statistics from the project's stats.json.
///
/// Returns a JSON object with `total_sessions`, `total_hours`, and `total_words`
/// for quick display in the UI footer. If the file doesn't exist or is corrupt,
/// returns zeros.
#[tauri::command]
pub fn cargar_estadisticas(project_path: String) -> Result<String, String> {
    let stats_path = Path::new(&project_path).join(".config").join("stats.json");
    // Check if we need to bootstrap (file missing OR seeded empty by crear_proyecto)
    let needs_bootstrap = if !stats_path.exists() {
        true
    } else if let Ok(raw) = std::fs::read_to_string(&stats_path) {
        let stats: SessionStats = serde_json::from_str(&raw).unwrap_or_default();
        stats.total_words == 0 && stats.sessions.is_empty()
    } else {
        false
    };
    if needs_bootstrap {
        if let Ok(stats) = inicializar_estadisticas_historicas(Path::new(&project_path)) {
            if let Ok(json) = serde_json::to_string_pretty(&stats) {
                let _ = std::fs::write(&stats_path, &json);
            }
            let total_hours = stats.total_time_seconds as f64 / 3600.0;
            let result = serde_json::json!({
                "total_sessions": stats.sessions.len(),
                "total_hours": (total_hours * 10.0).round() / 10.0,
                "total_words": stats.total_words,
            });
            return Ok(result.to_string());
        }
        return Ok(r#"{"total_sessions":0,"total_hours":0,"total_words":0}"#.to_string());
    }
    let raw = std::fs::read_to_string(&stats_path)
        .map_err(|e| format!("Error reading stats.json: {}", e))?;
    let stats: SessionStats = serde_json::from_str(&raw).unwrap_or_default();
    let total_hours = stats.total_time_seconds as f64 / 3600.0;
    let result = serde_json::json!({
        "total_sessions": stats.sessions.len(),
        "total_hours": (total_hours * 10.0).round() / 10.0,
        "total_words": stats.total_words,
    });
    Ok(result.to_string())
}
/// Build a baseline `SessionStats` for projects created before stats tracking existed.
///
/// Uses `git rev-list --count HEAD` for the session count and counts words in every
/// `.md` file under `capitulos/`. Time cannot be reconstructed — set to 0.
pub fn inicializar_estadisticas_historicas(project_path: &Path) -> Result<SessionStats, String> {
    let mut stats = SessionStats::default();
    let capitulos_dir = project_path.join("capitulos");
    // Count words per chapter
    if capitulos_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&capitulos_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let words = count_words_in_html(&content);
                            stats.total_words += words;
                            stats.chapters.insert(
                                filename.to_string(),
                                StatsChapter { time_seconds: 0, words },
                            );
                        }
                    }
                }
            }
        }
    }
    // Count git commits as proxy for sessions
    let commit_count = if let Ok(git_path) = find_git() {
        if let Ok(out) = std::process::Command::new(&git_path)
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(project_path)
            .output()
        {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).trim().parse::<usize>().unwrap_or(0)
            } else { 0 }
        } else { 0 }
    } else { 0 };
    // Synthesize session entries so total_sessions reflects git history
    for _ in 0..commit_count {
        stats.sessions.push(StatsSession {
            date: "---".to_string(),
            duration_seconds: 0,
            words_added: 0,
            chapter_id: String::new(),
        });
    }
    Ok(stats)
}
// ---------------------------------------------------------------------------
/// Compute and persist session statistics.
///
/// Best-effort — all errors are logged to `eprintln!` and the function
/// never panics.  On completion the tracker is reset to defaults so the
/// next project open starts fresh.
///
/// Steps:
///   1. Compute elapsed time since `start_time`
///   2. Accumulate current chapter time into `chapter_times`
///   3. Re-count words in the current chapter, diff against initial
///   4. Read or initialise `stats.json`
///   5. Update totals, per-chapter stats, and append session record
///   6. Write `stats.json` back to disk
///   7. Stage and commit `stats.json` via `system_command`
pub fn finalizar_sesion_escritura(tracker: &mut SessionTracker, project_path: &Path) {
    let start_time = match tracker.start_time {
        Some(t) => t,
        None => return, // No active session — nothing to collect
    };
    let total_elapsed = start_time.elapsed().as_secs();
    // Accumulate current chapter time before computing diffs
    if tracker.chapter_start.is_some() && tracker.chapter_filename.is_some() {
        let ch_file = tracker.chapter_filename.clone().unwrap();
        let ch_start = tracker.chapter_start.unwrap();
        let chapter_elapsed = ch_start.elapsed().as_secs();
        let accum = tracker.chapter_times.entry(ch_file).or_insert(0);
        *accum += chapter_elapsed;
    }
    // Compute words added for the current chapter
    let words_added = if let Some(ref filename) = tracker.chapter_filename {
        let current_words = word_count_chapter(project_path, filename);
        let initial = tracker.initial_word_count.unwrap_or(0);
        if current_words >= initial {
            current_words - initial
        } else {
            // File was edited outside the editor or truncated; count whatever exists
            current_words
        }
    } else {
        0
    };
    // ── Read or initialise stats.json ──────────────────────────
    let stats_path = project_path.join(".config").join("stats.json");
    let mut stats: SessionStats = if stats_path.exists() {
        std::fs::read_to_string(&stats_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                eprintln!("[stats] Corrupt stats.json — regenerating default");
                SessionStats::default()
            })
    } else {
        SessionStats::default()
    };
    // ── Update cumulative totals ────────────────────────────────
    stats.total_time_seconds += total_elapsed;
    stats.total_words += words_added;
    // ── Update per-chapter stats ────────────────────────────────
    if let Some(ref filename) = tracker.chapter_filename {
        let ch_time = tracker.chapter_times.get(filename).copied().unwrap_or(total_elapsed);
        let ch_stats = stats.chapters.entry(filename.clone()).or_default();
        ch_stats.words += words_added;
        ch_stats.time_seconds += ch_time;
    }
    // ── Append session record ──────────────────────────────────
    let session = StatsSession {
        date: Local::now().format("%Y-%m-%d").to_string(),
        duration_seconds: total_elapsed,
        words_added,
        chapter_id: tracker.chapter_filename.clone().unwrap_or_default(),
    };
    stats.sessions.push(session);
    // ── Write stats.json ───────────────────────────────────────
    let json = match serde_json::to_string_pretty(&stats) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[stats] Error serializing stats.json: {}", e);
            *tracker = SessionTracker::default();
            return;
        }
    };
    if let Err(e) = std::fs::write(&stats_path, &json) {
        eprintln!("[stats] Error writing stats.json: {}", e);
        *tracker = SessionTracker::default();
        return;
    }
    // ── Git add + commit (best-effort) ─────────────────────────
    let git_path = match find_git() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[stats] find_git error (non-fatal): {}", e);
            *tracker = SessionTracker::default();
            return;
        }
    };
    let stats_rel = Path::new(".config").join("stats.json");
    let _ = system_command(&git_path)
        .arg("add")
        .arg(&stats_rel)
        .current_dir(project_path)
        .output();
    let _ = system_command(&git_path)
        .arg("commit")
        .arg("-m")
        .arg("cron-insta: actualizar estadísticas de sesión")
        .current_dir(project_path)
        .output();
    // Reset tracker for the next project open
    *tracker = SessionTracker::default();
}

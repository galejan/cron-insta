// Cron-Insta — Data model structures
//
// All structs, enums, Default impls, serde helpers, and validation functions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Application state structures
// ---------------------------------------------------------------------------

/// Tauri managed state: tracks the active project for close-time checkpoint.
pub struct ProjectState {
    pub active_project: Mutex<Option<String>>,
    pub closing: Mutex<bool>,
    pub session_tracker: Mutex<SessionTracker>,
}

// ── Session statistics data structures ─────────────────────────

/// Per-chapter accumulated stats.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatsChapter {
    #[serde(default)]
    pub words: u64,
    #[serde(default)]
    pub time_seconds: u64,
}

/// A single writing session record.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsSession {
    pub date: String,
    pub duration_seconds: u64,
    pub words_added: u64,
    pub chapter_id: String,
}

/// Top-level stats container persisted to `.config/stats.json`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SessionStats {
    #[serde(default)]
    pub total_time_seconds: u64,
    #[serde(default)]
    pub total_words: u64,
    #[serde(default)]
    pub chapters: HashMap<String, StatsChapter>,
    #[serde(default)]
    pub sessions: Vec<StatsSession>,
}

/// In-memory runtime state for the writing session timer.
///
/// Tracks elapsed time per chapter and overall session duration.
/// Mutations happen only on chapter open (frontend IPC) and project
/// close (do_checkpoint), so contention is minimal.
pub struct SessionTracker {
    pub start_time: Option<std::time::Instant>,
    pub chapter_start: Option<std::time::Instant>,
    pub chapter_filename: Option<String>,
    pub initial_word_count: Option<u64>,
    pub chapter_times: HashMap<String, u64>,
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self {
            start_time: None,
            chapter_start: None,
            chapter_filename: None,
            initial_word_count: None,
            chapter_times: HashMap::new(),
        }
    }
}

/// Auto-detected git identity + remote from `.git/config`.
///
/// All fields are `Option` — missing `.git` or partial config yields `None`.
/// The struct is serialized directly by Tauri (no manual JSON).
#[derive(Serialize)]
pub struct GitDetectedConfig {
    pub name: Option<String>,
    pub email: Option<String>,
    pub remote_url: Option<String>,
}

/// Per-tab visibility toggles for the sidebar.
///
/// All fields default to `true` via serde and `Default` for backward
/// compatibility with old `metadata.json` files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisibleTabs {
    #[serde(default = "default_true")]
    pub chapters: bool,
    #[serde(default = "default_true")]
    pub characters: bool,
    #[serde(default = "default_true")]
    pub places: bool,
    #[serde(default = "default_true")]
    pub timeline: bool,
    #[serde(default = "default_true")]
    pub notes: bool,
    #[serde(default = "default_true")]
    pub media: bool,
}

impl Default for VisibleTabs {
    fn default() -> Self {
        Self {
            chapters: true,
            characters: true,
            places: true,
            timeline: true,
            notes: true,
            media: true,
        }
    }
}

pub fn default_true() -> bool {
    true
}

pub fn validate_visible_tabs(tabs: &VisibleTabs) -> Result<(), String> {
    if !tabs.chapters {
        return Err("Los capítulos deben estar siempre visibles (chapters debe ser true).".to_string());
    }
    Ok(())
}

pub fn validate_auto_save_interval(minutes: u32) -> Result<(), String> {
    match minutes {
        1 | 5 | 10 => Ok(()),
        _ => Err(format!(
            "Intervalo de autoguardado inválido: {}. Debe ser 1, 5 o 10 minutos.",
            minutes
        )),
    }
}

pub fn default_auto_save_interval() -> u32 {
    5
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    /// Schema version for backward compatibility. New projects get version 1.
    /// Old projects without this field deserialize to version 0.
    #[serde(default)]
    pub version: u32,
    pub project_name: String,
    pub last_modified: String,
    pub chapters_order: Vec<String>,
    pub characters_index: Vec<CharacterIndex>,
    #[serde(default)]
    pub places_index: Vec<LugarIndexItem>,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Whether auto-push to remote is active for this project.
    #[serde(default)]
    pub push_enabled: bool,
    /// Consecutive push failure count for the 3-strike rule.
    #[serde(default)]
    pub consecutive_failures: u32,
    /// Per-tab visibility toggles. All default to `true`.
    #[serde(default)]
    pub visible_tabs: VisibleTabs,
    /// Auto-save interval in minutes (1, 5, or 10). Default 5.
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_minutes: u32,
    /// Tramas (plotlines) — metadata-only groupings of chapters.
    #[serde(default)]
    pub tramas: Vec<Trama>,
    /// Chapter-to-trama assignments. Maps filename → optional trama_id.
    #[serde(default)]
    pub chapter_tramas: Vec<ChapterTrama>,
}

pub fn default_font_family() -> String {
    "monospace".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CharacterIndex {
    pub id: String,
    pub file: String,
    pub name: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub physicalDescription: Option<String>,
    #[serde(default)]
    pub personality: Option<String>,
    #[serde(default)]
    pub traumas: Option<String>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    #[serde(default)]
    pub targetId: Option<String>,
    pub targetName: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterIndexItem {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NoteIndexItem {
    pub id: String,
    pub title: String,
}

// ── Places — lugares ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LugarIndexItem {
    pub id: String,
    pub name: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lugar {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

// ── Tramas — plotlines ──────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trama {
    pub id: String,
    pub nombre: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChapterTrama {
    pub filename: String,
    #[serde(default)]
    pub trama_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct TimelineEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub date: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub relatedCharacters: Vec<String>,
    #[serde(default)]
    pub relatedChapters: Vec<String>,
    #[serde(default)]
    pub relatedPlaces: Vec<String>,
}

// ---------------------------------------------------------------------------
// Git identity & remote config data structures
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitIdentity {
    pub name: String,
    pub email: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)] // kept for test use
pub struct GitRemoteConfig {
    pub url: String,
    #[serde(default)]
    pub push_enabled: bool,
    #[serde(default)]
    pub consecutive_failures: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub identity: Option<GitIdentity>,
}

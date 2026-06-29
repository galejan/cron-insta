// Cron-Insta — Project repair/recovery commands
//
// reparar_proyecto — rebuilds indices from actual files on disk
// recrear_metadata — recreates metadata.json from scratch, then calls reparar_proyecto

use std::path::Path;
use chrono::Local;
use crate::models::*;
use crate::commands::project::generate_schema;

/// Report returned by reparar_proyecto detailing what was fixed.
#[derive(serde::Serialize, Debug, Clone)]
pub struct RepairReport {
    /// Items that were recovered/rebuilt (e.g., "personajes/index.json: 5 entries recovered")
    pub repaired: Vec<String>,
    /// Items that were created from scratch (e.g., "metadata.json: recreated from scratch")
    pub recreated: Vec<String>,
    /// Orphaned references that were cleaned up (e.g., "timeline.json: 3 orphan references removed")
    pub cleaned: Vec<String>,
    /// Data that could not be recovered (e.g., "chapter_tramas: 4 trama assignments lost")
    pub lost: Vec<String>,
}

/// Configuration for recreating a project's metadata.json from scratch.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct RecreateMetadataConfig {
    pub project_name: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default)]
    pub visible_tabs: Option<VisibleTabs>,
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_minutes: u32,
}

// ---------------------------------------------------------------------------
// Helper: extract first level-1 heading from markdown content
// ---------------------------------------------------------------------------

/// Extract the first level-1 heading from markdown content.
/// Returns None if no "# Title" line is found in the first 500 bytes.
fn extract_markdown_title(content: &str) -> Option<String> {
    for line in content.lines().take(20) {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            let title = stripped.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

/// Natural sort helper: compare two strings with numeric segments.
/// "capitulo_2" < "capitulo_10" (numeric-aware).
fn natural_sort_key(s: &str) -> Vec<(bool, &str)> {
    let chars: Vec<char> = s.chars().collect();
    let mut parts: Vec<(bool, &str)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            // Find byte offset
            let byte_start = s.char_indices().nth(start).map(|(b, _)| b).unwrap_or(0);
            let byte_end = s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len());
            parts.push((true, &s[byte_start..byte_end]));
        } else {
            let start = i;
            while i < chars.len() && !chars[i].is_ascii_digit() {
                i += 1;
            }
            let byte_start = s.char_indices().nth(start).map(|(b, _)| b).unwrap_or(0);
            let byte_end = s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len());
            parts.push((false, &s[byte_start..byte_end]));
        }
    }
    parts
}

fn natural_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let ka = natural_sort_key(a);
    let kb = natural_sort_key(b);
    for (pa, pb) in ka.iter().zip(kb.iter()) {
        match (pa.0, pb.0) {
            (true, true) => {
                // Both are numbers — compare numerically
                let na: u64 = pa.1.parse().unwrap_or(0);
                let nb: u64 = pb.1.parse().unwrap_or(0);
                let cmp = na.cmp(&nb);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (false, false) => {
                let cmp = pa.1.cmp(pb.1);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
        }
    }
    ka.len().cmp(&kb.len())
}

// ---------------------------------------------------------------------------
// reparar_proyecto
// ---------------------------------------------------------------------------

/// Repair a project's indices from actual files on disk.
///
/// Presupposes metadata.json exists and is readable.
/// Scans personajes/*.json, lugares/*.json, notas/*.md, capitulos/*.md
/// and rebuilds all index files and metadata accordingly.
/// Also cleans orphan references from timeline.json.
#[tauri::command]
pub fn reparar_proyecto(proyecto_path: String) -> Result<RepairReport, String> {
    let base = Path::new(&proyecto_path);
    let config_dir = base.join(".config");

    // 1. Load metadata.json
    let metadata_path = config_dir.join("metadata.json");
    if !metadata_path.exists() {
        return Err(
            "No se encontró metadata.json. Usá recrear_metadata para recrearlo desde cero."
                .to_string(),
        );
    }
    let raw =
        std::fs::read_to_string(&metadata_path).map_err(|e| format!("Error al leer metadata.json: {}", e))?;
    let mut metadata: Metadata =
        serde_json::from_str(&raw).map_err(|e| format!("Error al parsear metadata.json: {}", e))?;

    let mut report = RepairReport {
        repaired: Vec::new(),
        recreated: Vec::new(),
        cleaned: Vec::new(),
        lost: Vec::new(),
    };

    // 3. REPAIR characters_index + personajes/index.json
    {
        let personajes_dir = base.join("personajes");
        let mut rebuilt: Vec<CharacterIndex> = Vec::new();
        let mut index_items: Vec<CharacterIndexItem> = Vec::new();

        if personajes_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&personajes_dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    // EXCLUDE index.json
                    if fname == "index.json" || !fname.ends_with(".json") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(ch) = serde_json::from_str::<Character>(&content) {
                            rebuilt.push(CharacterIndex {
                                id: ch.id.clone(),
                                file: fname.clone(),
                                name: ch.name.clone(),
                            });
                            index_items.push(CharacterIndexItem {
                                id: ch.id,
                                name: ch.name,
                            });
                        }
                    }
                }
            }
        }

        // Sort by id for stable output
        rebuilt.sort_by(|a, b| a.id.cmp(&b.id));
        index_items.sort_by(|a, b| a.id.cmp(&b.id));

        let changed = metadata.characters_index != rebuilt;
        if changed {
            metadata.characters_index = rebuilt;
            let count = index_items.len();
            report.repaired.push(format!(
                "personajes/index.json: {} entradas recuperadas",
                count
            ));
        }

        // Always write index.json
        let index_json = serde_json::to_string_pretty(&index_items)
            .map_err(|e| format!("Error al serializar personajes/index.json: {}", e))?;
        std::fs::write(personajes_dir.join("index.json"), index_json)
            .map_err(|e| format!("Error al escribir personajes/index.json: {}", e))?;
    }

    // 4. REPAIR places_index + lugares/index.json
    {
        let lugares_dir = base.join("lugares");
        let mut rebuilt: Vec<LugarIndexItem> = Vec::new();

        if lugares_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&lugares_dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname == "index.json" || !fname.ends_with(".json") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(lugar) = serde_json::from_str::<Lugar>(&content) {
                            rebuilt.push(LugarIndexItem {
                                id: lugar.id,
                                name: lugar.name,
                            });
                        }
                    }
                }
            }
        }

        rebuilt.sort_by(|a, b| a.id.cmp(&b.id));

        let changed = metadata.places_index != rebuilt;
        if changed {
            metadata.places_index = rebuilt.clone();
            let count = rebuilt.len();
            report.repaired.push(format!(
                "lugares/index.json: {} entradas recuperadas",
                count
            ));
        }

        let index_json = serde_json::to_string_pretty(&rebuilt)
            .map_err(|e| format!("Error al serializar lugares/index.json: {}", e))?;
        std::fs::write(lugares_dir.join("index.json"), index_json)
            .map_err(|e| format!("Error al escribir lugares/index.json: {}", e))?;
    }

    // 5. REPAIR notas/index.json
    {
        let notas_dir = base.join("notas");
        let mut rebuilt: Vec<NoteIndexItem> = Vec::new();

        if notas_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&notas_dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    // EXCLUDE index.json, only process .md files
                    if fname == "index.json" || !fname.ends_with(".md") {
                        continue;
                    }
                    // id = filename without ".md"
                    let id = fname.trim_end_matches(".md").to_string();

                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let title = extract_markdown_title(&content)
                            .unwrap_or_else(|| id.clone());
                        rebuilt.push(NoteIndexItem { id, title });
                    }
                }
            }
        }

        rebuilt.sort_by(|a, b| a.id.cmp(&b.id));

        let count = rebuilt.len();

        // Only report if index.json changed or didn't exist
        let index_path = notas_dir.join("index.json");
        let should_report = if index_path.exists() {
            match std::fs::read_to_string(&index_path) {
                Ok(existing_raw) => {
                    let existing: Result<Vec<NoteIndexItem>, _> = serde_json::from_str(&existing_raw);
                    match existing {
                        Ok(existing_items) => existing_items != rebuilt,
                        Err(_) => true, // corrupt — report the repair
                    }
                }
                Err(_) => true,
            }
        } else {
            true
        };

        if should_report {
            report.repaired.push(format!(
                "notas/index.json: {} entradas recuperadas",
                count
            ));
        }

        let index_json = serde_json::to_string_pretty(&rebuilt)
            .map_err(|e| format!("Error al serializar notas/index.json: {}", e))?;
        std::fs::write(notas_dir.join("index.json"), index_json)
            .map_err(|e| format!("Error al escribir notas/index.json: {}", e))?;
    }

    // 6. REPAIR chapters_order (if empty)
    {
        let capitulos_dir = base.join("capitulos");
        if metadata.chapters_order.is_empty() {
            let mut found: Vec<String> = Vec::new();
            if capitulos_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&capitulos_dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname.ends_with(".md") {
                            found.push(fname);
                        }
                    }
                }
            }

            if !found.is_empty() {
                found.sort_by(|a, b| natural_compare(a, b));
                let count = found.len();
                metadata.chapters_order = found;
                report.repaired.push(format!(
                    "chapters_order: {} capítulos recuperados",
                    count
                ));
            }
        }
        // If NOT empty: leave as-is (user may have custom ordering)
    }

    // 7. REPAIR chapter_tramas (if empty or missing entries)
    {
        let missing: Vec<String> = metadata
            .chapters_order
            .iter()
            .filter(|ch| !metadata.chapter_tramas.iter().any(|ct| &ct.filename == *ch))
            .cloned()
            .collect();

        if metadata.chapter_tramas.is_empty() && !metadata.chapters_order.is_empty() {
            // All chapters need assignment (None)
            metadata.chapter_tramas = metadata
                .chapters_order
                .iter()
                .map(|ch| ChapterTrama {
                    filename: ch.clone(),
                    trama_id: None,
                })
                .collect();
            report.lost.push(
                "chapter_tramas: inicializadas (todas sin asignar)".to_string(),
            );
        } else if !missing.is_empty() {
            // Add only genuinely missing entries
            let existing = std::mem::take(&mut metadata.chapter_tramas);
            metadata.chapter_tramas = existing;
            for ch in missing {
                metadata.chapter_tramas.push(ChapterTrama {
                    filename: ch,
                    trama_id: None,
                });
            }
            // Don't report as lost — unassigned is the default
        }
    }

    // 8. CLEAN timeline.json orphan references
    {
        let timeline_path = config_dir.join("timeline.json");
        if timeline_path.exists() {
            let raw = std::fs::read_to_string(&timeline_path)
                .map_err(|e| format!("Error al leer timeline.json: {}", e))?;
            let mut events: Vec<TimelineEvent> =
                serde_json::from_str(&raw).map_err(|e| format!("Error al parsear timeline.json: {}", e))?;

            // Build sets of valid IDs for quick lookup
            let valid_char_ids: std::collections::HashSet<String> = metadata
                .characters_index
                .iter()
                .map(|ci| ci.id.clone())
                .collect();
            let valid_chapter_files: std::collections::HashSet<String> = metadata
                .chapters_order
                .iter()
                .cloned()
                .collect();
            let valid_place_ids: std::collections::HashSet<String> = metadata
                .places_index
                .iter()
                .map(|pi| pi.id.clone())
                .collect();

            let mut orphan_count = 0u64;
            for evt in &mut events {
                let before_char = evt.relatedCharacters.len();
                let before_chap = evt.relatedChapters.len();
                let before_place = evt.relatedPlaces.len();

                evt.relatedCharacters.retain(|id| valid_char_ids.contains(id));
                evt.relatedChapters.retain(|ch| valid_chapter_files.contains(ch));
                evt.relatedPlaces.retain(|id| valid_place_ids.contains(id));

                if evt.relatedCharacters.len() < before_char
                    || evt.relatedChapters.len() < before_chap
                    || evt.relatedPlaces.len() < before_place
                {
                    orphan_count += (before_char - evt.relatedCharacters.len()) as u64;
                    orphan_count += (before_chap - evt.relatedChapters.len()) as u64;
                    orphan_count += (before_place - evt.relatedPlaces.len()) as u64;
                }
            }

            if orphan_count > 0 {
                let new_json = serde_json::to_string_pretty(&events)
                    .map_err(|e| format!("Error al serializar timeline.json: {}", e))?;
                std::fs::write(&timeline_path, new_json)
                    .map_err(|e| format!("Error al escribir timeline.json: {}", e))?;
                report.cleaned.push(format!(
                    "timeline.json: {} referencias huérfanas eliminadas",
                    orphan_count
                ));
            }
        }
    }

    // 9. Update last_modified
    metadata.last_modified = Local::now().to_rfc3339();

    // 10. Write metadata.json back to disk
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, metadata_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;

    Ok(report)
}

// ---------------------------------------------------------------------------
// recrear_metadata
// ---------------------------------------------------------------------------

/// Recreate metadata.json from scratch when it's missing or corrupt.
///
/// Creates a fresh Metadata with user-provided config, seeds empty
/// timeline.json and stats.json, writes SCHEMA.md, then calls
/// reparar_proyecto internally to rebuild all indices from actual files.
#[tauri::command]
pub fn recrear_metadata(path: String, config: RecreateMetadataConfig) -> Result<RepairReport, String> {
    let base = Path::new(&path);

    // 1. Verify the path has at least capitulos/ directory
    let capitulos_dir = base.join("capitulos");
    if !capitulos_dir.exists() || !capitulos_dir.is_dir() {
        return Err(
            "No parece un proyecto de Cron-Insta (falta capitulos/)".to_string(),
        );
    }

    // Validate font_family
    match config.font_family.as_str() {
        "monospace" | "serif" | "sans-serif" => {}
        other => {
            return Err(format!(
                "Fuente inválida: '{}'. Debe ser monospace, serif o sans-serif.",
                other
            ));
        }
    }

    // Validate auto_save_interval_minutes
    validate_auto_save_interval(config.auto_save_interval_minutes)?;

    // 2. Create .config/ directory if it doesn't exist
    let config_dir = base.join(".config");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Error al crear .config/: {}", e))?;

    // 3. Create fresh Metadata
    let metadata = Metadata {
        version: 1,
        project_name: config.project_name,
        last_modified: Local::now().to_rfc3339(),
        chapters_order: Vec::new(),
        characters_index: Vec::new(),
        places_index: Vec::new(),
        font_family: config.font_family,
        push_enabled: false,
        consecutive_failures: 0,
        visible_tabs: config.visible_tabs.unwrap_or_default(),
        auto_save_interval_minutes: config.auto_save_interval_minutes,
        tramas: Vec::new(),
        chapter_tramas: Vec::new(),
    };

    // 4. Write metadata.json
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(config_dir.join("metadata.json"), &metadata_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;

    // 5. Seed empty timeline.json if missing
    let timeline_path = config_dir.join("timeline.json");
    if !timeline_path.exists() {
        std::fs::write(&timeline_path, "[]")
            .map_err(|e| format!("Error al escribir timeline.json: {}", e))?;
    }

    // 6. Seed empty stats.json if missing
    let stats_path = config_dir.join("stats.json");
    if !stats_path.exists() {
        let stats = SessionStats::default();
        let stats_json = serde_json::to_string_pretty(&stats)
            .map_err(|e| format!("Error al serializar stats.json: {}", e))?;
        std::fs::write(&stats_path, stats_json)
            .map_err(|e| format!("Error al escribir stats.json: {}", e))?;
    }

    // 7. Write SCHEMA.md
    let schema = generate_schema(&metadata.project_name);
    std::fs::write(base.join("SCHEMA.md"), schema)
        .map_err(|e| format!("Error al escribir SCHEMA.md: {}", e))?;

    // 8. Initialize report with recreated entry
    let mut report = RepairReport {
        repaired: Vec::new(),
        recreated: vec!["metadata.json: recreado desde cero".to_string()],
        cleaned: Vec::new(),
        lost: Vec::new(),
    };

    // 9. Call reparar_proyecto internally
    match reparar_proyecto(path) {
        Ok(inner_report) => {
            // 10. Merge reports
            report.repaired.extend(inner_report.repaired);
            report.recreated.extend(inner_report.recreated);
            report.cleaned.extend(inner_report.cleaned);
            report.lost.extend(inner_report.lost);
        }
        Err(e) => {
            // reparar_proyecto failed but metadata is already written
            report.lost.push(format!("reparar_proyecto falló: {}", e));
        }
    }

    Ok(report)
}

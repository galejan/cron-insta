use chrono::Local;
use tauri::Manager;
use std::path::Path;
use crate::models::*;
use crate::utils::*;
use crate::commands::git::inicializar_git;

/// Create a new Cron-Insta literary project.
///
/// Creates the base directory plus four subdirectories (`.config/`,
/// `capitulos/`, `personajes/`, `notas/`), seeds `.config/metadata.json`
/// and `.config/timeline.json`, then automatically initialises a Git
/// repository (silently — disk structure is created regardless of Git
/// availability).
///
/// The Git identity is read from the global config file; falls back to
/// the default "Cron-Insta" identity when no config exists.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn crear_proyecto(app: tauri::AppHandle, path: String, nombre: String, font_family: Option<String>, visible_tabs: Option<VisibleTabs>, auto_save_interval_minutes: Option<u32>) -> Result<String, String> {
    // Normalise trailing separators
    let path = path.trim_end_matches('/').trim_end_matches('\\').to_string();
    let base = Path::new(&path);
    // Reject if a project already exists at this path
    let metadata_path = base.join(".config").join("metadata.json");
    if metadata_path.exists() {
        return Err(format!(
            "PROJECT_ALREADY_EXISTS:Ya existe un proyecto en '{}'. ¿Querés abrirlo en lugar de crear uno nuevo?",
            base.display()
        ));
    }
    // Create base directory
    std::fs::create_dir_all(base)
        .map_err(|e| format!("No se pudo crear el directorio del proyecto: {}", e))?;
    // Create subdirectories
    let subdirs = [".config", "capitulos", "personajes", "notas", "lugares", "media"];
    for sub in &subdirs {
        std::fs::create_dir_all(base.join(sub))
            .map_err(|e| format!("No se pudo crear el directorio {}: {}", sub, e))?;
    }
    // Seed lugares/index.json (empty array)
    std::fs::write(base.join("lugares/index.json"), "[]")
        .map_err(|e| format!("Error al escribir lugares/index.json: {}", e))?;
    // Seed personajes/index.json (empty array)
    std::fs::write(base.join("personajes/index.json"), "[]")
        .map_err(|e| format!("Error al escribir personajes/index.json: {}", e))?;
    // Seed notas/index.json (empty array)
    std::fs::write(base.join("notas/index.json"), "[]")
        .map_err(|e| format!("Error al escribir notas/index.json: {}", e))?;
    // Validate interval if provided
    if let Some(interval) = auto_save_interval_minutes {
        validate_auto_save_interval(interval)?;
    }
    // Validate tabs if provided
    if let Some(ref tabs) = visible_tabs {
        validate_visible_tabs(tabs)?;
    }
    // Write metadata.json
    let metadata = Metadata {
        version: 1,
        project_name: nombre.clone(),
        last_modified: Local::now().to_rfc3339(),
        chapters_order: vec![],
        characters_index: vec![],
        places_index: vec![],
        font_family: font_family.unwrap_or_else(default_font_family),
        push_enabled: false,
        consecutive_failures: 0,
        visible_tabs: visible_tabs.unwrap_or_default(),
        auto_save_interval_minutes: auto_save_interval_minutes.unwrap_or_else(default_auto_save_interval),
        tramas: vec![],
        chapter_tramas: vec![],
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata: {}", e))?;
    std::fs::write(base.join(".config/metadata.json"), metadata_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    // Write timeline.json (empty array)
    std::fs::write(base.join(".config/timeline.json"), "[]")
        .map_err(|e| format!("Error al escribir timeline.json: {}", e))?;
    // Write stats.json (empty seed)
    let stats = SessionStats::default();
    let stats_json = serde_json::to_string_pretty(&stats)
        .map_err(|e| format!("Error al serializar stats: {}", e))?;
    std::fs::write(base.join(".config/stats.json"), stats_json)
        .map_err(|e| format!("Error al escribir stats.json: {}", e))?;
    // Write SCHEMA.md — data model description for AI agent consumption
    let schema = generate_schema(&nombre);
    std::fs::write(base.join("SCHEMA.md"), schema)
        .map_err(|e| format!("Error al escribir SCHEMA.md: {}", e))?;
    // Auto-initialise git — silently ignore if git is unavailable
    let _ = inicializar_git(app, path.clone());
    Ok(format!("Proyecto '{}' creado en {}", nombre, path))
}
/// Copy the app icon into the project and set it as folder icon.
///
/// Best-effort — never fails project creation.
/// - **Linux**: copies 32x32.png as .cron-insta-icon.png, sets GVFS metadata.
/// - **Windows**: copies icon.ico as .cron-insta-icon.ico, creates desktop.ini
///   and marks the folder with +s attribute so Explorer picks up the icon.
#[tauri::command]
pub fn marcar_proyecto_cron_insta(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let base = Path::new(&path);
    #[cfg(target_os = "linux")]
    {
        let icon_dest = base.join(".cron-insta-icon.png");
        if let Ok(resource_dir) = app.path().resource_dir() {
            let icon_src = resource_dir.join("icons/32x32.png");
            if icon_src.exists() {
                std::fs::copy(&icon_src, &icon_dest)
                    .map_err(|e| format!("Error al copiar icono: {}", e))?;
            }
        }
        // Set folder icon via GVFS (GNOME, Nemo, Cinnamon...)
        if let Ok(icon_abs) = icon_dest.canonicalize() {
            let icon_uri = format!("file://{}", icon_abs.display());
            let _ = system_command("gio")
                .arg("set").arg("-t").arg("string")
                .arg(base)
                .arg("metadata::custom-icon")
                .arg(&icon_uri)
                .output();
        }
    }
    #[cfg(target_os = "windows")]
    {
        let icon_dest = base.join(".cron-insta-icon.ico");
        if let Ok(resource_dir) = app.path().resource_dir() {
            let icon_src = resource_dir.join("icons/icon.ico");
            if icon_src.exists() {
                std::fs::copy(&icon_src, &icon_dest)
                    .map_err(|e| format!("Error copying icon: {}", e))?;
            }
        }
        // Create desktop.ini to tell Explorer about the custom icon
        let desktop_ini = base.join("desktop.ini");
        let ini_content = format!(
            "[.ShellClassInfo]\r\nIconFile={}\r\nIconIndex=0\r\n",
            ".cron-insta-icon.ico"
        );
        std::fs::write(&desktop_ini, ini_content)
            .map_err(|e| format!("Error writing desktop.ini: {}", e))?;
        // Mark folder as system so Explorer reads desktop.ini
        let _ = system_command("attrib")
            .arg("+s")
            .arg(base)
            .output();
        // Hide the desktop.ini and icon files
        let _ = system_command("attrib")
            .arg("+h")
            .arg(&desktop_ini)
            .output();
        let _ = system_command("attrib")
            .arg("+h")
            .arg(&icon_dest)
            .output();
    }
    Ok(())
}
/// Tell the Rust backend which project is currently open in the frontend.
///
/// Called when a project is opened (path = Some) or closed (path = None).
/// The backend uses this to run a git checkpoint when the window is closed,
/// avoiding the JS→Rust IPC deadlock during `onCloseRequested`.
#[tauri::command]
pub fn set_active_project(
    state: tauri::State<ProjectState>,
    path: Option<String>,
) -> Result<(), String> {
    let mut active = state.active_project.lock().map_err(|e| e.to_string())?;
    *active = path;
    Ok(())
}
/// Generate the SCHEMA.md content for a new project.
///
/// Centralised here so the schema stays in sync with the data model
/// without hunting through a raw string inside `crear_proyecto`.
pub fn generate_schema(nombre: &str) -> String {
    let schema = r#"# SCHEMA — {NOMBRE}
Generated by Cron-Insta. This file describes the project data model for AI agent consumption.
## Overview
This is a literary writing project managed by **Cron-Insta**, a desktop writing application. Data is stored as files on disk — no database is used.
## Entities
### Chapter
- **Storage**: `capitulos/{{filename}}.md` (one file per chapter)
- **Format**: HTML content rendered by TipTap (ProseMirror-based rich text editor)
- **Indexing**: Ordered list of filenames in `metadata.json → chapters_order`
- **Usage**: The core content of the project; each chapter is a section of the written work.
### Character
- **Storage**: `personajes/{{id}}.json` (one JSON file per character)
- **Fields**:
  - `id` (string): Unique identifier
  - `name` (string): Character display name
  - `physicalDescription` (string, optional): Physical appearance
  - `personality` (string, optional): Personality traits
  - `traumas` (string, optional): Backstory or trauma
  - `relationships` (array): List of relationships with other characters
- **Index**: `personajes/index.json` → array of `{{ id, name }}`
### Character Relationship
- **Location**: Nested inside each Character's `relationships` array
- **Fields**:
  - `targetId` (string, optional): ID of the related character
  - `targetName` (string): Display name of the related character
  - `type` (string): Relationship type (e.g. "friend", "rival", "family")
  - `notes` (string, optional): Free-text notes about the relationship
- **Note**: `targetId` is a soft reference — not validated against the character index.
### Note
- **Storage**: `notas/{{id}}.md` (one Markdown file per note)
- **Format**: HTML content rendered by TipTap
- **Index**: `notas/index.json` → array of `{{ id, title }}`
- **Usage**: Free-form notes, brainstorming, outlines, or research related to the project.
### Place
- **Storage**: `lugares/{{id}}.json` (one JSON file per place)
- **Fields**:
  - `id` (string): Unique identifier
  - `name` (string): Place display name
  - `description` (string): Place description
- **Index**: `lugares/index.json` → array of `{{ id, name }}`
### Media
- **Storage**: `media/` directory (flat — no subdirectories)
- **Format**: Binary files (images, PDFs, audio, etc.) managed by the multimedia gallery
- **Usage**: Reference images, mood boards, covers, and other project-related media files.
### Timeline Event
- **Storage**: `.config/timeline.json` (single JSON array file)
- **Fields**:
  - `id` (string): Unique identifier (format: `evt-{{timestamp_ms}}`)
  - `date` (string): Free-text date or ISO date string
  - `title` (string): Event title
  - `description` (string): Event description
  - `relatedCharacters` (array of strings): IDs of related characters (soft reference)
  - `relatedChapters` (array of strings): Filenames of related chapters (soft reference)
  - `relatedPlaces` (array of strings): IDs of related places (soft reference)
### Trama
- **Storage**: Inline in `.config/metadata.json` under `tramas` array
- **Fields**:
  - `id` (string): Unique identifier derived from the trama name (slug + hex suffix)
  - `nombre` (string): Display name of the trama
- **Usage**: Metadata-only grouping of chapters into narrative plotlines. Chapters remain flat in `capitulos/` regardless of trama assignment.
- **Chapter-Trama Assignment**: Stored in `.config/metadata.json` under `chapter_tramas` as `{{filename, trama_id}}`. `trama_id` is `null` for unassigned chapters.
## Relationships
```
TimelineEvent.relatedCharacters ──soft──▶ Character.id
TimelineEvent.relatedChapters   ──soft──▶ Chapter filename
TimelineEvent.relatedPlaces     ──soft──▶ Place.id
Character.relationships[].targetId ──soft──▶ Character.id
ChapterTrama.trama_id           ──soft──▶ Trama.id
```
All references are **soft** (no foreign key enforcement). Deleting a Character, Chapter, or Place:
- Removes its references from `TimelineEvent.relatedCharacters` / `relatedChapters` / `relatedPlaces`
- Character relationships (`targetId`) are NOT automatically cleaned up
Timeline events linked to a deleted entity are NOT deleted — only the reference is removed.
Deleting a Trama sets all its assigned chapters' `trama_id` to `null` — chapters are never deleted.
## Project Configuration
### `.config/metadata.json`
| Field | Type | Description |
|-------|------|-------------|
| `project_name` | string | Display name of the project |
| `version` | number | Schema version (1 = current). Used for backward compatibility. |
| `last_modified` | string | ISO 8601 timestamp of last modification |
| `chapters_order` | string[] | Ordered list of chapter filenames |
| `characters_index` | object[] | Array of `{{ id, file, name }}` |
| `places_index` | object[] | Array of `{{ id, name }}` |
| `tramas` | object[] | Array of `{{ id, nombre }}` plotline groupings |
| `chapter_tramas` | object[] | Array of `{{ filename, trama_id }}` assignments |
| `font_family` | string | Editor font: `"monospace"`, `"serif"`, or `"sans-serif"` |
| `push_enabled` | boolean | Whether auto-push to remote is active for this project (default: false) |
| `consecutive_failures` | number | Consecutive push failure count for the 3-strike auto-disable rule (default: 0) |
### `.config/timeline.json`
JSON array of TimelineEvent objects (see Entity section above).
## Directory Structure
```
{NOMBRE}/
├── .config/
│   ├── metadata.json
│   ├── timeline.json
│   └── stats.json
├── capitulos/
│   ├── 0001_prologo.md
│   └── ...
├── personajes/
│   ├── index.json
│   ├── {{id}}.json
│   └── ...
├── notas/
│   ├── index.json
│   ├── {{id}}.md
│   └── ...
├── lugares/
│   ├── index.json
│   ├── {{id}}.json
│   └── ...
├── media/
│   ├── image1.jpg
│   └── ...
└── SCHEMA.md          ◀── this file
```
"#;
    schema.replace("{NOMBRE}", nombre)
}

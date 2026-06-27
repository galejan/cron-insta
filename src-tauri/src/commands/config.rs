use chrono::Local;
use std::path::Path;
use crate::models::*;
use crate::commands::git::get_config_path;

/// Update the project font family in metadata.json.
///
/// Reads `{project_path}/.config/metadata.json`, updates `font_family` and
/// `last_modified` (ISO 8601), then writes the modified JSON back to disk.
/// Preserves all other fields (`project_name`, `chapters_order`, `characters_index`).
#[tauri::command]
pub fn actualizar_fuente_proyecto(project_path: String, font_family: String) -> Result<String, String> {
    if project_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if font_family.trim().is_empty() {
        return Err("La familia tipográfica no puede estar vacía.".to_string());
    }
    let valid_fonts = ["monospace", "serif", "sans-serif"];
    if !valid_fonts.contains(&font_family.as_str()) {
        return Err(format!(
            "Fuente inválida: '{}'. Debe ser monospace, serif o sans-serif.",
            font_family
        ));
    }
    let metadata_path = Path::new(&project_path)
        .join(".config")
        .join("metadata.json");
    if !metadata_path.exists() {
        return Err(format!(
            "Archivo de metadatos no encontrado: {}",
            metadata_path.display()
        ));
    }
    let metadata_str = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Error al leer metadata.json: {}", e))?;
    let mut metadata: Metadata = serde_json::from_str(&metadata_str)
        .map_err(|e| format!("Error al parsear metadata.json: {}", e))?;
    metadata.font_family = font_family;
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    Ok("".to_string())
}
/// Merge partial project configuration into metadata.json.
///
/// Reads the current metadata, merges the given partial JSON config
/// (a `serde_json::Value`), validates the result (chapters must be true,
/// interval must be 1|5|10), and writes the merged output back to disk.
///
/// Returns the full merged metadata as a JSON string so the frontend can
/// update its state without re-fetching.
#[tauri::command]
pub fn actualizar_config_proyecto(project_path: String, config: serde_json::Value) -> Result<String, String> {
    if project_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let metadata_path = Path::new(&project_path)
        .join(".config")
        .join("metadata.json");
    if !metadata_path.exists() {
        return Err(format!(
            "Archivo de metadatos no encontrado: {}",
            metadata_path.display()
        ));
    }
    // 1) Read current metadata
    let metadata_str = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Error al leer metadata.json: {}", e))?;
    let mut metadata: Metadata = serde_json::from_str(&metadata_str)
        .map_err(|e| format!("Error al parsear metadata.json: {}", e))?;
    // 2) Merge partial config (only overwrite fields present in the payload)
    if let Some(obj) = config.as_object() {
        if let Some(visible_tabs_val) = obj.get("visible_tabs") {
            if let Ok(tabs) = serde_json::from_value::<VisibleTabs>(visible_tabs_val.clone()) {
                metadata.visible_tabs = tabs;
            }
        }
        if let Some(interval_val) = obj.get("auto_save_interval_minutes") {
            if let Some(interval) = interval_val.as_u64() {
                metadata.auto_save_interval_minutes = interval as u32;
            }
        }
        if let Some(font_val) = obj.get("font_family") {
            if let Some(font) = font_val.as_str() {
                let valid_fonts = ["monospace", "serif", "sans-serif"];
                if valid_fonts.contains(&font) {
                    metadata.font_family = font.to_string();
                }
            }
        }
    }
    // 3) Validate merged result
    validate_visible_tabs(&metadata.visible_tabs)?;
    validate_auto_save_interval(metadata.auto_save_interval_minutes)?;
    // 4) Update timestamp and write
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    // Return full merged metadata so frontend can update state
    let full_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("Error al serializar metadata: {}", e))?;
    Ok(full_json)
}
/// Load the stored Git identity from the global config file.
///
/// Returns the serialised `GitIdentity` JSON `{name, email}` when found,
/// or the literal string `"null"` when no config exists or the file is
/// corrupted (graceful degradation — the frontend decides which preset to
/// show).
#[tauri::command]
pub fn cargar_identidad_git(app: tauri::AppHandle) -> Result<String, String> {
    let config_path = match get_config_path(&app) {
        Some(p) => p,
        None => return Ok("null".to_string()),
    };
    if !config_path.exists() {
        return Ok("null".to_string());
    }
    let raw = match std::fs::read_to_string(&config_path) {
        Ok(r) => r,
        Err(_) => return Ok("null".to_string()),
    };
    let config: GitConfig = match serde_json::from_str(&raw) {
        Ok(c) => c,
        Err(_) => return Ok("null".to_string()), // corrupted JSON → graceful degradation
    };
    match config.identity {
        Some(id) => serde_json::to_string(&id)
            .map_err(|e| format!("Error serializing identity: {}", e)),
        None => Ok("null".to_string()),
    }
}
/// Persist the user's Git identity to the global config file.
///
/// Uses a read-modify-write pattern: the full config is read first (if it
/// exists) so any existing remote configuration is preserved. The config
/// directory is created if it does not yet exist.
#[tauri::command]
pub fn guardar_identidad_git(
    app: tauri::AppHandle,
    name: String,
    email: String,
    github_user: Option<String>,
) -> Result<String, String> {
    let config_path = match get_config_path(&app) {
        Some(p) => p,
        None => return Err("Could not determine config directory".to_string()),
    };
    // Ensure the parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Error creating config directory: {}", e))?;
    }
    let identity = GitIdentity { name, email, github_user };
    // Read-modify-write: preserve identity-only config
    let mut config = if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str::<GitConfig>(&raw).unwrap_or(GitConfig {
            schema_version: 1,
            identity: None,
        })
    } else {
        GitConfig {
            schema_version: 1,
            identity: None,
        }
    };
    config.identity = Some(identity);
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Error serializing config: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("Error writing config: {}", e))?;
    Ok("Identity saved successfully.".to_string())
}

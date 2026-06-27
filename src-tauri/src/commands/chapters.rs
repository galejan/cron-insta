use chrono::Local;
use std::path::Path;
use crate::models::*;

/// Save chapter content to disk (Nivel 1 — no git commit).
///
/// Writes UTF-8 content to `{proyecto_path}/capitulos/{filename}`,
/// creating the parent directory if needed.  Overwrites any existing
/// file at the same path.
#[tauri::command]
pub fn guardar_capitulo(
    proyecto_path: String,
    filename: String,
    contenido: String,
) -> Result<String, String> {
    let cap_dir = Path::new(&proyecto_path).join("capitulos");
    // Ensure the capítulos directory exists
    std::fs::create_dir_all(&cap_dir)
        .map_err(|e| format!("No se pudo crear el directorio capítulos: {}", e))?;
    let file_path = cap_dir.join(&filename);
    std::fs::write(&file_path, contenido)
        .map_err(|e| format!("Error al guardar el capítulo: {}", e))?;
    Ok(format!("Capítulo guardado: {}", file_path.display()))
}
/// Read and return the project metadata index.
///
/// Returns the raw contents of `.config/metadata.json` as a JSON string.
/// The frontend (caller) is responsible for parsing and validation.
#[tauri::command]
pub fn cargar_indice(proyecto_path: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let metadata_path = Path::new(&proyecto_path).join(".config").join("metadata.json");
    if !metadata_path.exists() {
        return Err(format!(
            "Archivo de índice no encontrado: {}",
            metadata_path.display()
        ));
    }
    std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("No se pudo leer el índice del proyecto: {}", e))
}
/// Read a single chapter file from disk.
///
/// Returns the UTF-8 content of `{proyecto_path}/capitulos/{filename}`.
/// The frontend is responsible for parsing and rendering the markdown/HTML.
#[tauri::command]
pub fn cargar_capitulo(proyecto_path: String, filename: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if filename.trim().is_empty() {
        return Err("El nombre del archivo no puede estar vacío.".to_string());
    }
    let file_path = Path::new(&proyecto_path).join("capitulos").join(&filename);
    if !file_path.exists() {
        return Err(format!("Archivo no encontrado: {}", file_path.display()));
    }
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Error al leer el capítulo: {}", e))
}
/// Create a new chapter .md file and register it in metadata.json.
///
/// 1. Rejects duplicates (file already exists in `capitulos/`).
/// 2. Writes the `.md` file first.
/// 3. Updates `metadata.json`: appends `filename` to `chapters_order`
///    and refreshes `last_modified`.
///
/// Write order (file first, then metadata) prevents an index entry
/// pointing to a missing file in case of a crash mid-operation.
#[tauri::command]
pub fn crear_capitulo(
    proyecto_path: String,
    filename: String,
    contenido: String,
    trama_id: Option<String>,
) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if filename.trim().is_empty() {
        return Err("El nombre del archivo no puede estar vacío.".to_string());
    }
    let cap_dir = Path::new(&proyecto_path).join("capitulos");
    let file_path = cap_dir.join(&filename);
    // Reject duplicates
    if file_path.exists() {
        return Err(format!("El capítulo '{}' ya existe.", filename));
    }
    // Ensure the capítulos directory exists
    std::fs::create_dir_all(&cap_dir)
        .map_err(|e| format!("No se pudo crear el directorio capítulos: {}", e))?;
    // 1) Write the .md file first
    std::fs::write(&file_path, &contenido)
        .map_err(|e| format!("Error al crear el capítulo: {}", e))?;
    // 2) Update metadata.json
    let metadata_path = Path::new(&proyecto_path).join(".config").join("metadata.json");
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
    metadata.chapters_order.push(filename.clone());
    metadata.last_modified = Local::now().to_rfc3339();
    // Register trama assignment if provided
    if let Some(ref tid) = trama_id {
        // Validate trama exists
        if !metadata.tramas.iter().any(|t| &t.id == tid) {
            return Err(format!("La trama con ID '{}' no existe.", tid));
        }
        // Remove existing assignment for this filename (if any)
        metadata.chapter_tramas.retain(|ct| ct.filename != filename);
        metadata.chapter_tramas.push(ChapterTrama {
            filename: filename.clone(),
            trama_id: Some(tid.clone()),
        });
    } else {
        // Explicitly register as unassigned
        metadata.chapter_tramas.retain(|ct| ct.filename != filename);
        metadata.chapter_tramas.push(ChapterTrama {
            filename: filename.clone(),
            trama_id: None,
        });
    }
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    Ok(format!("Capítulo creado: {}", file_path.display()))
}
/// Delete a chapter.
///
/// 1. Validates non-empty path and filename.
/// 2. Deletes `capitulos/{filename}` — returns error if not found.
/// 3. Removes `filename` from `chapters_order` in metadata.json.
/// 4. Cleans references from timeline events' `relatedChapters` arrays.
#[tauri::command]
pub fn eliminar_capitulo(proyecto_path: String, filename: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if filename.trim().is_empty() {
        return Err("El nombre del archivo no puede estar vacío.".to_string());
    }
    let file_path = Path::new(&proyecto_path).join("capitulos").join(&filename);
    if !file_path.exists() {
        return Err(format!("El capítulo '{}' no existe.", filename));
    }
    // Delete the chapter file
    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Error al eliminar el capítulo: {}", e))?;
    // Remove from metadata chapters_order
    let metadata_path = Path::new(&proyecto_path).join(".config").join("metadata.json");
    if !metadata_path.exists() {
        return Err("Archivo de metadatos no encontrado.".to_string());
    }
    let metadata_str = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Error al leer metadata.json: {}", e))?;
    let mut metadata: Metadata = serde_json::from_str(&metadata_str)
        .map_err(|e| format!("Error al parsear metadata.json: {}", e))?;
    metadata.chapters_order.retain(|ch| ch != &filename);
    metadata.chapter_tramas.retain(|ct| ct.filename != filename);
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    // Clean references from timeline
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if timeline_path.exists() {
        let raw = std::fs::read_to_string(&timeline_path)
            .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
        let mut timeline: Vec<TimelineEvent> =
            serde_json::from_str(&raw).unwrap_or_default();
        for event in &mut timeline {
            event.relatedChapters.retain(|ch| ch != &filename);
        }
        let timeline_json = serde_json::to_string_pretty(&timeline)
            .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
        std::fs::write(&timeline_path, timeline_json)
            .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    }
    Ok(format!("Capítulo '{}' eliminado.", filename))
}

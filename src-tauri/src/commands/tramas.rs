use chrono::Local;
use std::path::Path;
use crate::models::*;
use crate::utils::*;

/// Create a new trama and persist it to metadata.json.
///
/// Rejects duplicate names. Slugifies the name into a unique ID with a
/// random 8-char hex suffix and appends to `metadata.tramas`.
#[tauri::command]
pub fn crear_trama(path: String, nombre: String) -> Result<Trama, String> {
    if path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if nombre.trim().is_empty() {
        return Err("El nombre de la trama no puede estar vacío.".to_string());
    }
    let proyecto_path = Path::new(&path);
    let metadata_path = proyecto_path.join(".config").join("metadata.json");
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
    // Reject duplicate names
    let nombre_trim = nombre.trim();
    if metadata.tramas.iter().any(|t| t.nombre == nombre_trim) {
        return Err(format!("Ya existe una trama con el nombre '{}'.", nombre_trim));
    }
    let id = slugify_trama_id(nombre_trim);
    let trama = Trama {
        id,
        nombre: nombre_trim.to_string(),
    };
    metadata.tramas.push(trama.clone());
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    Ok(trama)
}
/// Delete a trama by ID. Chapters assigned to it become unassigned.
///
/// Removes the trama from `metadata.tramas` and sets all matching
/// `chapter_tramas` entries to `trama_id: null`. No chapter files
/// are ever deleted.
#[tauri::command]
pub fn eliminar_trama(path: String, id: String) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID de la trama no puede estar vacío.".to_string());
    }
    let proyecto_path = Path::new(&path);
    let metadata_path = proyecto_path.join(".config").join("metadata.json");
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
    // Reject nonexistent trama
    if !metadata.tramas.iter().any(|t| t.id == id) {
        return Err(format!("No existe una trama con el ID '{}'.", id));
    }
    metadata.tramas.retain(|t| t.id != id);
    // Unassign all chapters that belonged to this trama
    for ct in &mut metadata.chapter_tramas {
        if ct.trama_id.as_deref() == Some(&id) {
            ct.trama_id = None;
        }
    }
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    Ok(())
}
/// Assign a chapter to a trama (or unassign it when `trama_id` is None).
///
/// Upserts the `chapter_tramas` entry. Validates that the trama exists when
/// `trama_id` is `Some`. `chapters_order` is never modified.
#[tauri::command]
pub fn asignar_capitulo_trama(path: String, filename: String, trama_id: Option<String>) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if filename.trim().is_empty() {
        return Err("El nombre del archivo no puede estar vacío.".to_string());
    }
    let proyecto_path = Path::new(&path);
    let metadata_path = proyecto_path.join(".config").join("metadata.json");
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
    // Validate trama exists when assigning
    if let Some(ref tid) = trama_id {
        if !metadata.tramas.iter().any(|t| &t.id == tid) {
            return Err(format!("No existe una trama con el ID '{}'.", tid));
        }
    }
    // Upsert: remove existing entry, then push the new one
    metadata.chapter_tramas.retain(|ct| ct.filename != filename);
    metadata.chapter_tramas.push(ChapterTrama {
        filename: filename.clone(),
        trama_id,
    });
    metadata.last_modified = Local::now().to_rfc3339();
    let updated_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata.json: {}", e))?;
    std::fs::write(&metadata_path, updated_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;
    Ok(())
}

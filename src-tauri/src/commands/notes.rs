use std::path::Path;
use crate::models::*;

/// List all notes in a project.
///
/// Reads `notas/index.json`. Returns JSON array string.
/// If file is missing, returns "[]".
#[tauri::command]
pub fn listar_notas(proyecto_path: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let index_path = Path::new(&proyecto_path).join("notas").join("index.json");
    if !index_path.exists() {
        return Ok("[]".to_string());
    }
    std::fs::read_to_string(&index_path)
        .map_err(|e| format!("No se pudo leer el índice de notas: {}", e))
}
/// Create or update a note (upsert — follows guardar_capitulo pattern).
///
/// Creates or overwrites `notas/{id}.md` with the given content.
/// Updates `notas/index.json` (adds or updates title).
#[tauri::command]
pub fn crear_nota(
    proyecto_path: String,
    id: String,
    titulo: String,
    contenido: String,
) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID de la nota no puede estar vacío.".to_string());
    }
    let notas_dir = Path::new(&proyecto_path).join("notas");
    let note_file = notas_dir.join(format!("{}.md", id));
    let existed = note_file.exists();
    // Ensure directory exists
    std::fs::create_dir_all(&notas_dir)
        .map_err(|e| format!("No se pudo crear el directorio notas: {}", e))?;
    // Write / overwrite note file
    std::fs::write(&note_file, &contenido)
        .map_err(|e| format!("Error al guardar la nota: {}", e))?;
    // Update index
    let index_path = notas_dir.join("index.json");
    let mut index: Vec<NoteIndexItem> = if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de notas: {}", e))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        vec![]
    };
    if existed {
        // Update existing entry
        for item in &mut index {
            if item.id == id {
                item.title = titulo.clone();
                break;
            }
        }
    } else {
        index.push(NoteIndexItem {
            id: id.clone(),
            title: titulo.clone(),
        });
    }
    let index_json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Error al serializar el índice de notas: {}", e))?;
    std::fs::write(&index_path, index_json)
        .map_err(|e| format!("Error al escribir el índice de notas: {}", e))?;
    let action = if existed { "actualizada" } else { "creada" };
    Ok(format!("Nota '{}' {}.", titulo, action))
}
/// Load a note by ID.
///
/// Reads `notas/{id}.md` and returns its markdown content.
#[tauri::command]
pub fn cargar_nota(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID de la nota no puede estar vacío.".to_string());
    }
    let note_path = Path::new(&proyecto_path)
        .join("notas")
        .join(format!("{}.md", id));
    if !note_path.exists() {
        return Err(format!("Nota '{}' no encontrada.", id));
    }
    std::fs::read_to_string(&note_path)
        .map_err(|e| format!("Error al leer la nota: {}", e))
}
/// Delete a note.
///
/// Deletes `notas/{id}.md` and removes the entry from `notas/index.json`.
#[tauri::command]
pub fn eliminar_nota(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID de la nota no puede estar vacío.".to_string());
    }
    let notas_dir = Path::new(&proyecto_path).join("notas");
    let note_path = notas_dir.join(format!("{}.md", id));
    if !note_path.exists() {
        return Err(format!("Nota '{}' no encontrada.", id));
    }
    // Delete the file
    std::fs::remove_file(&note_path)
        .map_err(|e| format!("Error al eliminar la nota: {}", e))?;
    // Remove from index
    let index_path = notas_dir.join("index.json");
    if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de notas: {}", e))?;
        let mut index: Vec<NoteIndexItem> = serde_json::from_str(&raw).unwrap_or_default();
        index.retain(|item| item.id != id);
        let index_json = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Error al serializar el índice de notas: {}", e))?;
        std::fs::write(&index_path, index_json)
            .map_err(|e| format!("Error al escribir el índice de notas: {}", e))?;
    }
    Ok(format!("Nota '{}' eliminada.", id))
}

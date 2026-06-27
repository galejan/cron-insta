use std::path::Path;
use crate::models::*;

/// List all characters in a project.
///
/// Reads `personajes/index.json`. Returns JSON array string.
/// If file is missing, returns "[]".
#[tauri::command]
pub fn listar_personajes(proyecto_path: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let index_path = Path::new(&proyecto_path).join("personajes").join("index.json");
    if !index_path.exists() {
        return Ok("[]".to_string());
    }
    std::fs::read_to_string(&index_path)
        .map_err(|e| format!("No se pudo leer el índice de personajes: {}", e))
}
/// Create a new character.
///
/// Parses the input JSON to extract `id` and `name`. Rejects duplicates.
/// Creates `personajes/{id}.json` and updates `personajes/index.json`.
#[tauri::command]
pub fn crear_personaje(proyecto_path: String, personaje_json: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let character: Character = serde_json::from_str(&personaje_json)
        .map_err(|e| format!("Error al parsear el personaje: {}", e))?;
    if character.id.trim().is_empty() {
        return Err("El ID del personaje no puede estar vacío.".to_string());
    }
    if character.name.trim().is_empty() {
        return Err("El nombre del personaje no puede estar vacío.".to_string());
    }
    let personajes_dir = Path::new(&proyecto_path).join("personajes");
    let char_file = personajes_dir.join(format!("{}.json", character.id));
    // Reject duplicates
    if char_file.exists() {
        return Err(format!("El personaje '{}' ya existe.", character.id));
    }
    // Ensure directory exists
    std::fs::create_dir_all(&personajes_dir)
        .map_err(|e| format!("No se pudo crear el directorio personajes: {}", e))?;
    // Write character file
    let char_json = serde_json::to_string_pretty(&character)
        .map_err(|e| format!("Error al serializar el personaje: {}", e))?;
    std::fs::write(&char_file, char_json)
        .map_err(|e| format!("Error al crear el personaje: {}", e))?;
    // Update index
    let index_path = personajes_dir.join("index.json");
    let mut index: Vec<CharacterIndexItem> = if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de personajes: {}", e))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        vec![]
    };
    index.push(CharacterIndexItem {
        id: character.id.clone(),
        name: character.name.clone(),
    });
    let index_json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Error al serializar el índice de personajes: {}", e))?;
    std::fs::write(&index_path, index_json)
        .map_err(|e| format!("Error al escribir el índice de personajes: {}", e))?;
    Ok(format!("Personaje '{}' creado.", character.name))
}
/// Load a character by ID.
///
/// Reads `personajes/{id}.json` and returns the full JSON string.
#[tauri::command]
pub fn cargar_personaje(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del personaje no puede estar vacío.".to_string());
    }
    let char_path = Path::new(&proyecto_path)
        .join("personajes")
        .join(format!("{}.json", id));
    if !char_path.exists() {
        return Err(format!("Personaje '{}' no encontrado.", id));
    }
    std::fs::read_to_string(&char_path)
        .map_err(|e| format!("Error al leer el personaje: {}", e))
}
/// Update a character.
///
/// Overwrites `personajes/{id}.json`. If the name changed, updates the index entry.
#[tauri::command]
pub fn actualizar_personaje(
    proyecto_path: String,
    id: String,
    personaje_json: String,
) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del personaje no puede estar vacío.".to_string());
    }
    let personajes_dir = Path::new(&proyecto_path).join("personajes");
    let char_path = personajes_dir.join(format!("{}.json", id));
    if !char_path.exists() {
        return Err(format!("Personaje '{}' no encontrado.", id));
    }
    // Read old character to detect name change
    let old_raw = std::fs::read_to_string(&char_path)
        .map_err(|e| format!("Error al leer el personaje existente: {}", e))?;
    let old_char: Character = serde_json::from_str(&old_raw)
        .map_err(|e| format!("Error al parsear el personaje existente: {}", e))?;
    let character: Character = serde_json::from_str(&personaje_json)
        .map_err(|e| format!("Error al parsear el personaje actualizado: {}", e))?;
    // Overwrite file
    let char_json = serde_json::to_string_pretty(&character)
        .map_err(|e| format!("Error al serializar el personaje: {}", e))?;
    std::fs::write(&char_path, char_json)
        .map_err(|e| format!("Error al guardar el personaje: {}", e))?;
    // Update index if name changed
    if old_char.name != character.name {
        let index_path = personajes_dir.join("index.json");
        if index_path.exists() {
            let raw = std::fs::read_to_string(&index_path)
                .map_err(|e| format!("Error al leer el índice de personajes: {}", e))?;
            let mut index: Vec<CharacterIndexItem> =
                serde_json::from_str(&raw).unwrap_or_default();
            for item in &mut index {
                if item.id == id {
                    item.name = character.name.clone();
                    break;
                }
            }
            let index_json = serde_json::to_string_pretty(&index)
                .map_err(|e| format!("Error al serializar el índice de personajes: {}", e))?;
            std::fs::write(&index_path, index_json)
                .map_err(|e| format!("Error al escribir el índice de personajes: {}", e))?;
        }
    }
    Ok(format!("Personaje '{}' actualizado.", character.name))
}
/// Delete a character.
///
/// Deletes `personajes/{id}.json`, removes from `personajes/index.json`,
/// and removes references from timeline events' `relatedCharacters` arrays.
#[tauri::command]
pub fn eliminar_personaje(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del personaje no puede estar vacío.".to_string());
    }
    let personajes_dir = Path::new(&proyecto_path).join("personajes");
    let char_path = personajes_dir.join(format!("{}.json", id));
    if !char_path.exists() {
        return Err(format!("Personaje '{}' no encontrado.", id));
    }
    // Delete the file
    std::fs::remove_file(&char_path)
        .map_err(|e| format!("Error al eliminar el personaje: {}", e))?;
    // Remove from index
    let index_path = personajes_dir.join("index.json");
    if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de personajes: {}", e))?;
        let mut index: Vec<CharacterIndexItem> =
            serde_json::from_str(&raw).unwrap_or_default();
        index.retain(|item| item.id != id);
        let index_json = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Error al serializar el índice de personajes: {}", e))?;
        std::fs::write(&index_path, index_json)
            .map_err(|e| format!("Error al escribir el índice de personajes: {}", e))?;
    }
    // Remove references from timeline
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if timeline_path.exists() {
        let raw = std::fs::read_to_string(&timeline_path)
            .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
        let mut timeline: Vec<TimelineEvent> =
            serde_json::from_str(&raw).unwrap_or_default();
        for event in &mut timeline {
            event.relatedCharacters.retain(|cid| cid != &id);
        }
        let timeline_json = serde_json::to_string_pretty(&timeline)
            .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
        std::fs::write(&timeline_path, timeline_json)
            .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    }
    Ok(format!("Personaje '{}' eliminado.", id))
}

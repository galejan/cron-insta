use std::path::Path;
use crate::models::*;

/// List all places in a project.
///
/// Reads `lugares/index.json`. Returns JSON array string.
/// If file is missing, returns "[]".
#[tauri::command]
pub fn listar_lugares(proyecto_path: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let index_path = Path::new(&proyecto_path).join("lugares").join("index.json");
    if !index_path.exists() {
        return Ok("[]".to_string());
    }
    std::fs::read_to_string(&index_path)
        .map_err(|e| format!("No se pudo leer el índice de lugares: {}", e))
}
/// Create a new place.
///
/// Parses the input JSON to extract `id` and `name`. Rejects duplicates.
/// Creates `lugares/{id}.json` and updates `lugares/index.json`.
#[tauri::command]
pub fn crear_lugar(proyecto_path: String, lugar_json: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let lugar: Lugar = serde_json::from_str(&lugar_json)
        .map_err(|e| format!("Error al parsear el lugar: {}", e))?;
    if lugar.id.trim().is_empty() {
        return Err("El ID del lugar no puede estar vacío.".to_string());
    }
    if lugar.name.trim().is_empty() {
        return Err("El nombre del lugar no puede estar vacío.".to_string());
    }
    let lugares_dir = Path::new(&proyecto_path).join("lugares");
    let lugar_file = lugares_dir.join(format!("{}.json", lugar.id));
    // Reject duplicates
    if lugar_file.exists() {
        return Err(format!("El lugar '{}' ya existe.", lugar.id));
    }
    // Ensure directory exists
    std::fs::create_dir_all(&lugares_dir)
        .map_err(|e| format!("No se pudo crear el directorio lugares: {}", e))?;
    // Write place file
    let lugar_json = serde_json::to_string_pretty(&lugar)
        .map_err(|e| format!("Error al serializar el lugar: {}", e))?;
    std::fs::write(&lugar_file, lugar_json)
        .map_err(|e| format!("Error al crear el lugar: {}", e))?;
    // Update index
    let index_path = lugares_dir.join("index.json");
    let mut index: Vec<LugarIndexItem> = if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de lugares: {}", e))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        vec![]
    };
    index.push(LugarIndexItem {
        id: lugar.id.clone(),
        name: lugar.name.clone(),
    });
    let index_json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("Error al serializar el índice de lugares: {}", e))?;
    std::fs::write(&index_path, index_json)
        .map_err(|e| format!("Error al escribir el índice de lugares: {}", e))?;
    Ok(format!("Lugar '{}' creado.", lugar.name))
}
/// Load a place by ID.
///
/// Reads `lugares/{id}.json` and returns the full JSON string.
#[tauri::command]
pub fn cargar_lugar(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del lugar no puede estar vacío.".to_string());
    }
    let lugar_path = Path::new(&proyecto_path)
        .join("lugares")
        .join(format!("{}.json", id));
    if !lugar_path.exists() {
        return Err(format!("Lugar '{}' no encontrado.", id));
    }
    std::fs::read_to_string(&lugar_path)
        .map_err(|e| format!("Error al leer el lugar: {}", e))
}
/// Update a place.
///
/// Overwrites `lugares/{id}.json`. If the name changed, updates the index entry.
#[tauri::command]
pub fn actualizar_lugar(
    proyecto_path: String,
    id: String,
    lugar_json: String,
) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del lugar no puede estar vacío.".to_string());
    }
    let lugares_dir = Path::new(&proyecto_path).join("lugares");
    let lugar_path = lugares_dir.join(format!("{}.json", id));
    if !lugar_path.exists() {
        return Err(format!("Lugar '{}' no encontrado.", id));
    }
    // Read old place to detect name change
    let old_raw = std::fs::read_to_string(&lugar_path)
        .map_err(|e| format!("Error al leer el lugar existente: {}", e))?;
    let old_lugar: Lugar = serde_json::from_str(&old_raw)
        .map_err(|e| format!("Error al parsear el lugar existente: {}", e))?;
    let lugar: Lugar = serde_json::from_str(&lugar_json)
        .map_err(|e| format!("Error al parsear el lugar actualizado: {}", e))?;
    // Overwrite file
    let lugar_json = serde_json::to_string_pretty(&lugar)
        .map_err(|e| format!("Error al serializar el lugar: {}", e))?;
    std::fs::write(&lugar_path, lugar_json)
        .map_err(|e| format!("Error al guardar el lugar: {}", e))?;
    // Update index if name changed
    if old_lugar.name != lugar.name {
        let index_path = lugares_dir.join("index.json");
        if index_path.exists() {
            let raw = std::fs::read_to_string(&index_path)
                .map_err(|e| format!("Error al leer el índice de lugares: {}", e))?;
            let mut index: Vec<LugarIndexItem> =
                serde_json::from_str(&raw).unwrap_or_default();
            for item in &mut index {
                if item.id == id {
                    item.name = lugar.name.clone();
                    break;
                }
            }
            let index_json = serde_json::to_string_pretty(&index)
                .map_err(|e| format!("Error al serializar el índice de lugares: {}", e))?;
            std::fs::write(&index_path, index_json)
                .map_err(|e| format!("Error al escribir el índice de lugares: {}", e))?;
        }
    }
    Ok(format!("Lugar '{}' actualizado.", lugar.name))
}
/// Delete a place.
///
/// Deletes `lugares/{id}.json` and removes from `lugares/index.json`.
#[tauri::command]
pub fn eliminar_lugar(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del lugar no puede estar vacío.".to_string());
    }
    let lugares_dir = Path::new(&proyecto_path).join("lugares");
    let lugar_path = lugares_dir.join(format!("{}.json", id));
    if !lugar_path.exists() {
        return Err(format!("Lugar '{}' no encontrado.", id));
    }
    // Delete the file
    std::fs::remove_file(&lugar_path)
        .map_err(|e| format!("Error al eliminar el lugar: {}", e))?;
    // Remove from index
    let index_path = lugares_dir.join("index.json");
    if index_path.exists() {
        let raw = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("Error al leer el índice de lugares: {}", e))?;
        let mut index: Vec<LugarIndexItem> =
            serde_json::from_str(&raw).unwrap_or_default();
        index.retain(|item| item.id != id);
        let index_json = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Error al serializar el índice de lugares: {}", e))?;
        std::fs::write(&index_path, index_json)
            .map_err(|e| format!("Error al escribir el índice de lugares: {}", e))?;
    }
    // Clean references from timeline events
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if timeline_path.exists() {
        let raw = std::fs::read_to_string(&timeline_path)
            .map_err(|e| format!("Error al leer timeline: {}", e))?;
        let mut timeline: Vec<TimelineEvent> =
            serde_json::from_str(&raw).unwrap_or_default();
        for event in &mut timeline {
            event.relatedPlaces.retain(|pid| pid != &id);
        }
        let timeline_json = serde_json::to_string_pretty(&timeline)
            .map_err(|e| format!("Error al serializar timeline: {}", e))?;
        std::fs::write(&timeline_path, timeline_json)
            .map_err(|e| format!("Error al escribir timeline: {}", e))?;
    }
    Ok(format!("Lugar '{}' eliminado.", id))
}

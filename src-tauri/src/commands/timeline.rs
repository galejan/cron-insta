use chrono::Local;
use std::path::Path;
use crate::models::*;

/// Read the timeline.
///
/// Reads `.config/timeline.json` and returns the JSON array.
#[tauri::command]
pub fn cargar_timeline(proyecto_path: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if !timeline_path.exists() {
        return Ok("[]".to_string());
    }
    std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))
}
/// Add an event to the timeline.
///
/// Parses the event JSON. Generates an `id` if not provided.
/// Appends to the timeline array in `.config/timeline.json`.
#[tauri::command]
pub fn agregar_evento_timeline(proyecto_path: String, evento_json: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let mut event: TimelineEvent = serde_json::from_str(&evento_json)
        .map_err(|e| format!("Error al parsear el evento: {}", e))?;
    // Generate ID if missing
    if event.id.trim().is_empty() {
        event.id = format!("evt-{}", Local::now().timestamp_millis());
    }
    if event.title.trim().is_empty() {
        return Err("El título del evento no puede estar vacío.".to_string());
    }
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    let mut timeline: Vec<TimelineEvent> = if timeline_path.exists() {
        let raw = std::fs::read_to_string(&timeline_path)
            .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        vec![]
    };
    // Reject duplicate IDs
    if timeline.iter().any(|e| e.id == event.id) {
        return Err(format!("Ya existe un evento con el ID '{}'.", event.id));
    }
    let event_id = event.id.clone();
    timeline.push(event);
    let timeline_json = serde_json::to_string_pretty(&timeline)
        .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
    std::fs::write(&timeline_path, timeline_json)
        .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    Ok(format!("Evento '{}' agregado a la línea de tiempo.", event_id))
}
/// Update an existing timeline event by ID.
///
/// `evento_json` must include the event's `id`. All other fields are replaced.
#[tauri::command]
pub fn actualizar_evento_timeline(proyecto_path: String, evento_json: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let updated: TimelineEvent = serde_json::from_str(&evento_json)
        .map_err(|e| format!("Error al parsear el evento: {}", e))?;
    if updated.id.trim().is_empty() {
        return Err("El ID del evento no puede estar vacío.".to_string());
    }
    if updated.title.trim().is_empty() {
        return Err("El título del evento no puede estar vacío.".to_string());
    }
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    let raw = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
    let mut timeline: Vec<TimelineEvent> = serde_json::from_str(&raw)
        .map_err(|e| format!("Error al parsear la línea de tiempo: {}", e))?;
    let idx = timeline.iter()
        .position(|e| e.id == updated.id)
        .ok_or_else(|| format!("No se encontró el evento con ID '{}'.", updated.id))?;
    let event_id = updated.id.clone();
    timeline[idx] = updated;
    let timeline_json = serde_json::to_string_pretty(&timeline)
        .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
    std::fs::write(&timeline_path, timeline_json)
        .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    Ok(format!("Evento '{}' actualizado.", event_id))
}
/// Reorder timeline events to match the given ID order.
///
/// `ids_json` is a JSON array of event IDs in the desired order.
/// Events with IDs not in the input are appended at the end.
/// IDs in the input that don't exist in the timeline are silently skipped.
#[tauri::command]
pub fn reordenar_timeline(proyecto_path: String, ids_json: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    let desired: Vec<String> = serde_json::from_str(&ids_json)
        .map_err(|e| format!("Error al parsear la lista de IDs: {}", e))?;
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if !timeline_path.exists() {
        return Err("El archivo de línea de tiempo no existe.".to_string());
    }
    let raw = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
    let mut timeline: Vec<TimelineEvent> = serde_json::from_str(&raw).unwrap_or_default();
    // Build a lookup: id -> event (take ownership, remove from vec)
    let mut event_map: std::collections::HashMap<String, TimelineEvent> = timeline
        .drain(..)
        .map(|e| (e.id.clone(), e))
        .collect();
    let mut reordered: Vec<TimelineEvent> = Vec::with_capacity(event_map.len());
    // Place events in the desired order
    for id in &desired {
        if let Some(event) = event_map.remove(id) {
            reordered.push(event);
        }
    }
    // Append any remaining events (IDs not in the input)
    for (_, event) in event_map {
        reordered.push(event);
    }
    let timeline_json = serde_json::to_string_pretty(&reordered)
        .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
    std::fs::write(&timeline_path, timeline_json)
        .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    Ok("Línea de tiempo reordenada correctamente.".to_string())
}
/// Remove an event from the timeline.
///
/// Deletes the event with the matching `id` from `.config/timeline.json`.
#[tauri::command]
pub fn eliminar_evento_timeline(proyecto_path: String, id: String) -> Result<String, String> {
    if proyecto_path.trim().is_empty() {
        return Err("La ruta del proyecto no puede estar vacía.".to_string());
    }
    if id.trim().is_empty() {
        return Err("El ID del evento no puede estar vacío.".to_string());
    }
    let timeline_path = Path::new(&proyecto_path).join(".config").join("timeline.json");
    if !timeline_path.exists() {
        return Err("El archivo de línea de tiempo no existe.".to_string());
    }
    let raw = std::fs::read_to_string(&timeline_path)
        .map_err(|e| format!("Error al leer la línea de tiempo: {}", e))?;
    let mut timeline: Vec<TimelineEvent> = serde_json::from_str(&raw).unwrap_or_default();
    let len_before = timeline.len();
    timeline.retain(|e| e.id != id);
    if timeline.len() == len_before {
        return Err(format!(
            "Evento '{}' no encontrado en la línea de tiempo.",
            id
        ));
    }
    let timeline_json = serde_json::to_string_pretty(&timeline)
        .map_err(|e| format!("Error al serializar la línea de tiempo: {}", e))?;
    std::fs::write(&timeline_path, timeline_json)
        .map_err(|e| format!("Error al escribir la línea de tiempo: {}", e))?;
    Ok(format!("Evento '{}' eliminado de la línea de tiempo.", id))
}

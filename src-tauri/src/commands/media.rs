use std::path::Path;

/// List files in the media directory with their sizes (bytes).
#[tauri::command]
pub fn listar_media(proyecto_path: String) -> Result<String, String> {
    let media_dir = Path::new(&proyecto_path).join("media");
    if !media_dir.is_dir() {
        return Ok("[]".to_string());
    }
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&media_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                files.push(serde_json::json!({ "name": name, "size": size }));
            }
        }
    }
    serde_json::to_string(&files).map_err(|e| format!("Error serializing media list: {}", e))
}
/// Copy a file into the project's media directory.
#[tauri::command]
pub fn copiar_a_media(proyecto_path: String, source_path: String) -> Result<String, String> {
    let source = Path::new(&source_path);
    if !source.is_file() {
        return Err("El archivo de origen no existe.".to_string());
    }
    let media_dir = Path::new(&proyecto_path).join("media");
    std::fs::create_dir_all(&media_dir)
        .map_err(|e| format!("No se pudo crear media/: {}", e))?;
    let fname = source.file_name().and_then(|n| n.to_str()).unwrap_or("image");
    let dest = media_dir.join(fname);
    // Avoid overwriting: append _1, _2, etc.
    let dest = if dest.exists() {
        let stem = fname.rfind('.').map(|i| &fname[..i]).unwrap_or(fname);
        let ext = fname.rfind('.').map(|i| &fname[i..]).unwrap_or("");
        let mut n = 1;
        loop {
            let candidate = media_dir.join(format!("{}_{}{}", stem, n, ext));
            if !candidate.exists() { break candidate; }
            n += 1;
        }
    } else { dest };
    std::fs::copy(source, &dest)
        .map_err(|e| format!("Error copiando archivo: {}", e))?;
    Ok(dest.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string())
}
/// Delete a media file and clean up all references in character/place JSONs.
#[tauri::command]
pub fn eliminar_media(proyecto_path: String, filename: String) -> Result<String, String> {
    let media_dir = Path::new(&proyecto_path).join("media");
    let file_path = media_dir.join(&filename);
    if !file_path.is_file() {
        return Err(format!("Archivo '{}' no encontrado en media/", filename));
    }
    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Error eliminando archivo: {}", e))?;

    // Clean up references in character JSONs
    let personajes_dir = Path::new(&proyecto_path).join("personajes");
    if personajes_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&personajes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json")
                    && path.file_name().and_then(|n| n.to_str()) != Some("index.json")
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if val.get("image").and_then(|v| v.as_str()) == Some(&filename) {
                                val["image"] = serde_json::Value::Null;
                                let _ = std::fs::write(&path, serde_json::to_string_pretty(&val).unwrap_or_default());
                            }
                        }
                    }
                }
            }
        }
    }

    // Clean up references in place JSONs
    let lugares_dir = Path::new(&proyecto_path).join("lugares");
    if lugares_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&lugares_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json")
                    && path.file_name().and_then(|n| n.to_str()) != Some("index.json")
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if val.get("image").and_then(|v| v.as_str()) == Some(&filename) {
                                val["image"] = serde_json::Value::Null;
                                let _ = std::fs::write(&path, serde_json::to_string_pretty(&val).unwrap_or_default());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(filename)
}

/// Read a media file and return it as a base64 data URL.
///
/// Tries an exact filename match first.  If that fails, falls back to a
/// case-insensitive lookup in the media directory — this protects against
/// cross-platform case-sensitivity differences (Windows/macOS are
/// case-insensitive; Linux is case-sensitive).
#[tauri::command]
pub fn leer_media_base64(proyecto_path: String, filename: String) -> Result<String, String> {
    use std::io::Read;
    let media_dir = Path::new(&proyecto_path).join("media");

    // 1) Exact match
    let exact_path = media_dir.join(&filename);
    let (resolved_path, resolved_name) = if exact_path.exists() {
        (exact_path, filename.clone())
    } else {
        // 2) Case-insensitive fallback: scan media/ for a matching filename
        let lower = filename.to_lowercase();
        let mut found: Option<(std::path::PathBuf, String)> = None;
        if let Ok(entries) = std::fs::read_dir(&media_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase() == lower {
                            found = Some((path.clone(), name.to_string()));
                            break;
                        }
                    }
                }
            }
        }
        match found {
            Some((p, n)) => (p, n),
            None => {
                // Neither exact nor case-insensitive match found
                return Err(format!(
                    "Archivo '{}' no encontrado en media/",
                    filename
                ));
            }
        }
    };

    let mut file = std::fs::File::open(&resolved_path)
        .map_err(|e| format!("Error abriendo archivo: {}", e))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("Error leyendo archivo: {}", e))?;
    let ext = resolved_name.split('.').last().unwrap_or("png").to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

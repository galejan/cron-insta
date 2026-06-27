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
/// Read a media file and return it as a base64 data URL.
#[tauri::command]
pub fn leer_media_base64(proyecto_path: String, filename: String) -> Result<String, String> {
    use std::io::Read;
    let path = Path::new(&proyecto_path).join("media").join(&filename);
    let mut file = std::fs::File::open(&path)
        .map_err(|e| format!("Error abriendo archivo: {}", e))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("Error leyendo archivo: {}", e))?;
    let ext = filename.split('.').last().unwrap_or("png").to_lowercase();
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

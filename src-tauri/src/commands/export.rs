use std::io::Write;
use chrono::Local;
use std::path::Path;
use crate::models::*;
use crate::utils::*;

/// Export the entire project as a .zip file.
///
/// Creates `exportaciones/` inside the project, then compresses all files
/// (including .git) into `{project_name}_{YYYY-MM-DD}.zip`.
#[tauri::command]
pub fn exportar_proyecto_zip(proyecto_path: String) -> Result<String, String> {
    use zip::write::FileOptions;
    let base = Path::new(&proyecto_path);
    let export_dir = base.join("exportaciones");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("No se pudo crear exportaciones/: {}", e))?;
    let metadata = read_metadata(base)?;
    let project_name = metadata.project_name.replace(' ', "_");
    let date = Local::now().format("%Y-%m-%d");
    let zip_name = format!("{}_{}.zip", project_name, date);
    let zip_path = export_dir.join(&zip_name);
    let file = std::fs::File::create(&zip_path)
        .map_err(|e| format!("Error al crear zip: {}", e))?;
    let mut zip_writer = zip::ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);
    // Walk the project directory and add all files, prefixed with project name
    let zip_prefix = format!("{}/", project_name);
    add_dir_to_zip(base, base, &zip_prefix, &mut zip_writer, &options)
        .map_err(|e| format!("Error al comprimir: {}", e))?;
    zip_writer.finish()
        .map_err(|e| format!("Error al finalizar zip: {}", e))?;
    Ok(zip_path.display().to_string())
}
/// Export all chapters as a single .md file.
///
/// Concatenates every chapter in the order stored in metadata,
/// separated by a divider. Writes to `exportaciones/{project}_{date}.md`.
#[tauri::command]
pub fn exportar_proyecto_md(proyecto_path: String) -> Result<String, String> {
    let base = Path::new(&proyecto_path);
    let export_dir = base.join("exportaciones");
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("No se pudo crear exportaciones/: {}", e))?;
    let metadata = read_metadata(base)?;
    let project_name = metadata.project_name.replace(' ', "_");
    let date = Local::now().format("%Y-%m-%d");
    let md_name = format!("{}_{}.md", project_name, date);
    let md_path = export_dir.join(&md_name);
    let cap_dir = base.join("capitulos");
    let mut output = String::new();
    output.push_str(&format!("# {}\n\n", metadata.project_name));
    output.push_str(&format!("*Exportado el {}*\n\n---\n\n", Local::now().format("%d/%m/%Y")));
    for filename in &metadata.chapters_order {
        let file_path = cap_dir.join(filename);
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let title = filename.trim_end_matches(".md").to_string();
            output.push_str(&format!("## {}\n\n", title));
            output.push_str(&content.trim());
            output.push_str("\n\n---\n\n");
        }
    }
    std::fs::write(&md_path, output)
        .map_err(|e| format!("Error al escribir .md: {}", e))?;
    Ok(md_path.display().to_string())
}
/// Import a Cron-Insta project from a .zip file.
///
/// Extracts all contents into the chosen destination directory.
/// A well-formed Cron-Insta ZIP wraps files in a project folder;
/// this function finds that folder by scanning for .config/metadata.json
/// inside the first level of subdirectories.  Falls back to the
/// destination root for legacy ZIPs without a wrapping folder.
///
/// Returns the actual project path (e.g. Documents/Hammet) on success.
#[tauri::command]
pub fn importar_proyecto(zip_path: String, destino: String) -> Result<String, String> {
    let zip_file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("No se pudo abrir el archivo ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("El archivo no es un ZIP válido: {}", e))?;
    let destino_path = std::path::Path::new(&destino);
    // Create destination if it doesn't exist
    std::fs::create_dir_all(destino_path)
        .map_err(|e| format!("No se pudo crear la carpeta de destino (comprobá los permisos): {}", e))?;
    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Error al leer entrada del ZIP: {}", e))?;
        let out_path = match file.enclosed_name() {
            Some(path) => destino_path.join(path),
            None => continue,
        };
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("Error al crear directorio {}: {}", out_path.display(), e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Error al crear directorio {}: {}", parent.display(), e))?;
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("Error al crear archivo {} (comprobá los permisos): {}", out_path.display(), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Error al extraer {}: {}", out_path.display(), e))?;
        }
    }
    // Find the project root: scan first-level subdirectories for .config/metadata.json.
    // Also check the destination root itself (legacy ZIPs without wrapping folder).
    let mut project_root = destino_path.to_path_buf();
    let mut found = false;
    // Check destination root first
    if destino_path.join(".config").join("metadata.json").exists() {
        found = true;
    } else if let Ok(entries) = std::fs::read_dir(destino_path) {
        for entry in entries.flatten() {
            let sub = entry.path();
            if sub.is_dir() && sub.join(".config").join("metadata.json").exists() {
                project_root = sub;
                found = true;
                break;
            }
        }
    }
    if !found {
        return Err("El archivo ZIP no parece ser un proyecto de Cron-Insta (falta .config/metadata.json).".to_string());
    }
    // Read project name for the success message
    let raw = std::fs::read_to_string(project_root.join(".config").join("metadata.json"))
        .map_err(|e| format!("Proyecto extraído pero no se pudo leer metadata: {}", e))?;
    let _metadata: Metadata = serde_json::from_str(&raw)
        .map_err(|e| format!("Proyecto extraído pero metadata.json es inválido: {}", e))?;
    Ok(project_root.display().to_string())
}
/// Recursively add directory contents to a zip writer, under a prefix folder.
pub fn add_dir_to_zip(
    base: &Path,
    dir: &Path,
    prefix: &str,
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: &zip::write::FileOptions<()>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir)
        .map_err(|e| format!("Error al leer directorio: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Error: {}", e))?;
        let path = entry.path();
        // Skip the exportaciones directory itself
        if path.file_name().map(|n| n == "exportaciones").unwrap_or(false) {
            continue;
        }
        let relative = path.strip_prefix(base)
            .map_err(|e| format!("Error: {}", e))?;
        let name = format!("{}{}", prefix, relative.to_string_lossy());
        if path.is_dir() {
            zip.add_directory(&name, *options)
                .map_err(|e| format!("Error al añadir directorio: {}", e))?;
            add_dir_to_zip(base, &path, prefix, zip, options)?;
        } else {
            zip.start_file(&name, *options)
                .map_err(|e| format!("Error al iniciar archivo: {}", e))?;
            let contents = std::fs::read(&path)
                .map_err(|e| format!("Error al leer {}: {}", path.display(), e))?;
            zip.write(&contents)
                .map_err(|e| format!("Error al escribir en zip: {}", e))?;
        }
    }
    Ok(())
}

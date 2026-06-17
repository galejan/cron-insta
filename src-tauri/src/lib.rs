// Cronista — Tauri backend
//
// Phase 2: Rust backend commands for project management and git abstraction.
// All five Tauri commands + find_git() helper live here per the single-module design.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    project_name: String,
    last_modified: String,
    chapters_order: Vec<String>,
    characters_index: Vec<CharacterIndex>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CharacterIndex {
    id: String,
    file: String,
    name: String,
}

// ---------------------------------------------------------------------------
// Helper: locate the git binary across platforms
// ---------------------------------------------------------------------------

/// Locate the `git` executable on the system.
///
/// **Linux**: uses `which git`.
/// **Windows**: tries `PATH` via `where git`, then falls back to the two
/// standard Git-for-Windows installation paths.
///
/// Returns `Ok(path)` when found, or `Err(msg)` with a user-facing Spanish
/// message when Git is unavailable.
fn find_git() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("which")
            .arg("git")
            .output()
            .map_err(|e| format!("Error al buscar git: {}", e))?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 1) Try PATH via `where git`
        if let Ok(output) = Command::new("where").arg("git").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // `where` may return multiple lines — take the first
                if let Some(first) = stdout.lines().next() {
                    let trimmed = first.trim();
                    if !trimmed.is_empty() {
                        return Ok(trimmed.to_string());
                    }
                }
            }
        }

        // 2) Fallback to well-known Git-for-Windows paths
        let fallbacks = [
            r"C:\Program Files\Git\bin\git.exe",
            r"C:\Program Files (x86)\Git\bin\git.exe",
        ];
        for fb in &fallbacks {
            if Path::new(fb).exists() {
                return Ok(fb.to_string());
            }
        }
    }

    Err("Git no está disponible. El control de versiones permanecerá inactivo.".to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Create a new Cronista literary project.
///
/// Creates the base directory plus four subdirectories (`.config/`,
/// `capitulos/`, `personajes/`, `notas/`), seeds `.config/metadata.json`
/// and `.config/timeline.json`, then automatically initialises a Git
/// repository (silently — disk structure is created regardless of Git
/// availability).
#[tauri::command]
fn crear_proyecto(path: String, nombre: String) -> Result<String, String> {
    // Normalise trailing separators
    let path = path.trim_end_matches('/').trim_end_matches('\\').to_string();
    let base = Path::new(&path);

    // Create base directory
    std::fs::create_dir_all(base)
        .map_err(|e| format!("No se pudo crear el directorio del proyecto: {}", e))?;

    // Create subdirectories
    let subdirs = [".config", "capitulos", "personajes", "notas"];
    for sub in &subdirs {
        std::fs::create_dir_all(base.join(sub))
            .map_err(|e| format!("No se pudo crear el directorio {}: {}", sub, e))?;
    }

    // Write metadata.json
    let metadata = Metadata {
        project_name: nombre.clone(),
        last_modified: Utc::now().to_rfc3339(),
        chapters_order: vec![],
        characters_index: vec![],
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Error al serializar metadata: {}", e))?;
    std::fs::write(base.join(".config/metadata.json"), metadata_json)
        .map_err(|e| format!("Error al escribir metadata.json: {}", e))?;

    // Write timeline.json (empty array)
    std::fs::write(base.join(".config/timeline.json"), "[]")
        .map_err(|e| format!("Error al escribir timeline.json: {}", e))?;

    // Auto-initialise git — silently ignore if git is unavailable
    let _ = inicializar_git(path.clone());

    Ok(format!("Proyecto '{}' creado en {}", nombre, path))
}

/// Initialise a Git repository in the given project path.
///
/// Returns success if `.git` already exists (reinit is safe) or if
/// `git init` succeeds.  Returns `Err` **only** when Git is unavailable —
/// callers can degrade gracefully.
#[tauri::command]
fn inicializar_git(path: String) -> Result<String, String> {
    let project_path = Path::new(&path);

    // Already initialised → silent success
    if project_path.join(".git").exists() {
        return Ok("El repositorio ya estaba inicializado.".to_string());
    }

    // Locate git binary (returns Err with user-facing message when absent)
    let git_path = find_git()?;

    let output = Command::new(&git_path)
        .arg("init")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git init: {}", e))?;

    if output.status.success() {
        Ok("Repositorio Git inicializado correctamente.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Error al inicializar Git: {}", stderr.trim()))
    }
}

/// Save chapter content to disk (Nivel 1 — no git commit).
///
/// Writes UTF-8 content to `{proyecto_path}/capitulos/{filename}`,
/// creating the parent directory if needed.  Overwrites any existing
/// file at the same path.
#[tauri::command]
fn guardar_capitulo(
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

/// Create a versioned checkpoint via Git (Nivel 2).
///
/// Stages all changes (`git add .`) and commits with a descriptive
/// progress message (`Progreso automático: {fecha} - {recuento} palabras`).
/// The word count is computed by counting whitespace-separated tokens in
/// every `.md` file under `capitulos/`.
///
/// Returns the commit hash on success, or a descriptive status when there
/// is nothing to commit (still `Ok` — not an error).
#[tauri::command]
fn crear_checkpoint(proyecto_path: String) -> Result<String, String> {
    let project_path = Path::new(&proyecto_path);
    let git_path = find_git()?;

    // Stage all changes
    let add_output = Command::new(&git_path)
        .arg("add")
        .arg(".")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git add: {}", e))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        return Err(format!("Error en git add: {}", stderr.trim()));
    }

    // Count words in chapter files for the commit message
    let word_count = count_words_in_chapters(project_path);
    let date = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let commit_msg = format!(
        "Progreso automático: {} - {} palabras",
        date, word_count
    );

    // Commit
    let commit_output = Command::new(&git_path)
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git commit: {}", e))?;

    if commit_output.status.success() {
        // Retrieve the commit hash
        let hash_output = Command::new(&git_path)
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Error al obtener el hash del commit: {}", e))?;

        let hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();
        Ok(hash)
    } else {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        // "nothing to commit" is a normal state, not an error
        if stderr.contains("nothing to commit") || stderr.contains("nothing added to commit") {
            Ok("Sin cambios para guardar.".to_string())
        } else {
            Err(format!("Error en git commit: {}", stderr.trim()))
        }
    }
}

/// Read and return the project metadata index.
///
/// Returns the raw contents of `.config/metadata.json` as a JSON string.
/// The frontend (caller) is responsible for parsing and validation.
#[tauri::command]
fn cargar_indice(proyecto_path: String) -> Result<String, String> {
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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Count whitespace-separated tokens across all `.md` files under
/// `{project_path}/capitulos/`.  Returns 0 when the directory is
/// missing or empty.
fn count_words_in_chapters(project_path: &Path) -> usize {
    let cap_dir = project_path.join("capitulos");
    if !cap_dir.exists() {
        return 0;
    }

    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(&cap_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    total += content.split_whitespace().count();
                }
            }
        }
    }
    total
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            crear_proyecto,
            inicializar_git,
            guardar_capitulo,
            crear_checkpoint,
            cargar_indice,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

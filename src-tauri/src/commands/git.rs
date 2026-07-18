use chrono::Local;
use tauri::Manager;
use serde::Serialize;
use std::path::{Path, PathBuf};
use crate::models::*;
use crate::utils::*;
use crate::commands::stats::finalizar_sesion_escritura;

/// Initialise a Git repository in the given project path.
///
/// Returns success if `.git` already exists (reinit is safe) or if
/// `git init` succeeds.  Returns `Err` **only** when Git is unavailable —
/// callers can degrade gracefully.
///
/// Reads the Git identity from the global config file. Falls back to
/// the default "Cron-Insta" / "cron-insta@local" identity when no config
/// exists (backward-compatible behaviour).
#[tauri::command]
pub fn inicializar_git(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let project_path = Path::new(&path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&path).to_path_buf());
    // Check if git already considers this a valid work tree
    if let Ok(git_path) = find_git() {
        let check = system_command(&git_path)
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .current_dir(&project_path)
            .output();
        if let Ok(out) = check {
            if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true" {
                return Ok("El repositorio ya estaba inicializado.".to_string());
            }
        }
    }
    // Locate git binary (returns Err with user-facing message when absent)
    let git_path = find_git()?;
    let output = system_command(&git_path)
        .arg("init")
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git init: {}", e))?;
    if output.status.success() {
        // Read identity from global config, fall back to defaults
        let (user_name, user_email) = read_identity_from_config(&app)
            .unwrap_or_else(|| ("Cron-Insta".to_string(), "cron-insta@local".to_string()));
        // Set user identity (best-effort, silent on failure)
        let _ = system_command(&git_path)
            .arg("config")
            .arg("user.name")
            .arg(&user_name)
            .current_dir(&project_path)
            .output();
        let _ = system_command(&git_path)
            .arg("config")
            .arg("user.email")
            .arg(&user_email)
            .current_dir(&project_path)
            .output();
        // First commit — "Primera piedra"
        let _ = system_command(&git_path)
            .arg("add")
            .arg(".")
            .current_dir(&project_path)
            .output();
        let commit_msg = "Primera piedra ✍️";
        let commit_output = system_command(&git_path)
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .current_dir(&project_path)
            .output()
            .map_err(|e| format!("Error en primer commit: {}", e))?;
        if commit_output.status.success() {
            // Ensure the branch is named "main" (git may default to "master")
            let _ = system_command(&git_path)
                .arg("branch")
                .arg("-M")
                .arg("main")
                .current_dir(&project_path)
                .output();
            Ok("Repositorio Git inicializado y primer commit creado.".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            if stderr.contains("nothing to commit") || stderr.contains("nothing added") {
                Ok("Repositorio Git inicializado (sin archivos para commit aún).".to_string())
            } else {
                Err(format!("Error en primer commit: {}", stderr.trim()))
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Error al inicializar Git: {}", stderr.trim()))
    }
}
/// Check if the project has a git repository initialized (.git directory).
///
/// Returns true when `<project>/.git` exists, regardless of whether git
/// the binary is installed.
#[tauri::command]
pub fn verificar_git_inicializado(path: String) -> Result<bool, String> {
    let project_path = Path::new(&path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&path).to_path_buf());
    // Use git itself to verify — catches worktrees, submodules,
    // and corrupt .git dirs that Path::exists() would miss.
    let git_path = match find_git() {
        Ok(g) => g,
        Err(_) => return Ok(Path::new(&path).join(".git").exists()),
    };
    let output = system_command(&git_path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Error ejecutando git: {}", e))?;
    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true")
}
/// Remove the .git directory from a project (used when importing a project
/// and the user wants to start a fresh history).
#[tauri::command]
pub fn eliminar_directorio_git(path: String) -> Result<(), String> {
    let git_dir = std::path::Path::new(&path).join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir)
            .map_err(|e| format!("No se pudo eliminar el historial Git: {}", e))?;
    }
    Ok(())
}
/// Return the last N git log entries for the project.
///
/// Each entry is a JSON object: { hash, date, message, words }.
/// Words are extracted from the commit message's "— N palabras" suffix
/// when present, otherwise shown as "—".
#[tauri::command]
pub fn obtener_git_log(path: String, limit: usize) -> Result<String, String> {
    eprintln!("[cron-insta] obtener_git_log called with path: '{}'", path);
    let project_path = Path::new(&path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&path).to_path_buf());
    eprintln!("[cron-insta] obtener_git_log canonicalized: '{}'", project_path.display());
    let git_path = find_git()?;
    let output = system_command(&git_path)
        .arg("log")
        .arg(format!("--format=%H|%ai|%s"))
        .arg(format!("-{}", limit.max(1).min(20)))
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Error al leer el historial: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error en git log: {}", stderr.trim()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            let hash_full = parts.first().map(|s| s.to_string()).unwrap_or_default();
            let hash = hash_full.chars().take(7).collect::<String>();
            let date = parts.get(1).unwrap_or(&"—").to_string();
            let raw_msg = parts.get(2).unwrap_or(&"—");
            // Extract word count from the commit message suffix
            let (message, words) = if let Some(pos) = raw_msg.rfind("—") {
                let suffix = raw_msg[pos..].trim();
                if suffix.contains("palabras") || suffix.contains("words") {
                    (raw_msg[..pos].trim().to_string(), suffix.to_string())
                } else {
                    (raw_msg.to_string(), "—".to_string())
                }
            } else {
                (raw_msg.to_string(), "—".to_string())
            };
            // Get changed .md files for this commit
            let files = get_changed_md_files(&project_path, &git_path, &hash_full);
            serde_json::json!({
                "hash": hash,
                "date": date,
                "message": message,
                "words": words,
                "files": files,
            })
        })
        .collect();
    serde_json::to_string(&entries)
        .map_err(|e| format!("Error al serializar el historial: {}", e))
}
/// Detect whether Git is installed on the system.
///
/// Returns `true` when `find_git()` locates a valid Git binary.
/// Lightweight command — no I/O beyond binary discovery.
#[tauri::command]
pub fn detectar_git() -> Result<bool, String> {
    Ok(find_git().is_ok())
}
/// Detect git identity and remote from `.git/config` for a project path.
///
/// Runs `git config user.name`, `git config user.email`,
/// and `git remote get-url origin` inside the project directory.
/// Best-effort only — never errors, missing data returns `None`.
#[tauri::command]
pub fn detectar_config_git(project_path: String) -> GitDetectedConfig {
    let base = Path::new(&project_path);
    let git_dir = base.join(".git");
    if !git_dir.exists() {
        return GitDetectedConfig {
            name: None,
            email: None,
            remote_url: None,
        };
    }
    let git_path = match find_git() {
        Ok(p) => p,
        Err(_) => {
            return GitDetectedConfig {
                name: None,
                email: None,
                remote_url: None,
            };
        }
    };
    let run_config = |key: &str| -> Option<String> {
        system_command(&git_path)
            .arg("config")
            .arg("--local")
            .arg(key)
            .current_dir(base)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let remote_url = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(base)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    GitDetectedConfig {
        name: run_config("user.name"),
        email: run_config("user.email"),
        remote_url,
    }
}
/// Create a versioned checkpoint via Git (Nivel 2).
///
/// Stages all changes (`git add .`) and commits with a descriptive
/// progress message (`Progreso automático: {fecha} - {recuento} palabras`).
/// The word count is computed by counting whitespace-separated tokens in
/// every `.md` file under `capitulos/`.
///
/// When `push_enabled: true` and a remote is configured, attempts
/// `git push origin main` after a successful commit. Push failures
/// are tracked via the 3-strike counter and surfaced as a warning
/// appended to the commit hash.
///
/// Returns the commit hash on success (with optional push warning),
/// or a descriptive status when there is nothing to commit (still
/// `Ok` — not an error).
#[tauri::command]
pub fn crear_checkpoint(_app: tauri::AppHandle, proyecto_path: String) -> Result<String, String> {
    let project_path = Path::new(&proyecto_path);
    let commit_result = perform_commit(project_path)?;
    // No auto-push here — only do_checkpoint (close handler) syncs.
    Ok(commit_result)
}
/// Load the per-project push state from the project's metadata.json.
///
/// Returns the serialised JSON `{push_enabled, consecutive_failures, url}`
/// when metadata exists. The `url` is read from `git remote get-url origin`
/// and is `null` when no remote is configured.
///
/// Returns the literal string `"null"` when metadata is missing or corrupted.
#[tauri::command]
pub fn cargar_config_remoto(_app: tauri::AppHandle, proyecto_path: String) -> Result<String, String> {
    let base = Path::new(&proyecto_path);
    let meta_path = base.join(".config").join("metadata.json");
    if !meta_path.exists() {
        return Ok("null".to_string());
    }
    let raw = match std::fs::read_to_string(&meta_path) {
        Ok(r) => r,
        Err(_) => return Ok("null".to_string()),
    };
    let meta: Metadata = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(_) => return Ok("null".to_string()),
    };
    // Read remote URL from git config (best-effort)
    let remote_url: Option<String> = if let Ok(git_path) = find_git() {
        system_command(&git_path)
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .current_dir(base)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    };
    #[derive(Serialize)]
    struct RemoteState {
        push_enabled: bool,
        consecutive_failures: u32,
        url: Option<String>,
    }
    let state = RemoteState {
        push_enabled: meta.push_enabled,
        consecutive_failures: meta.consecutive_failures,
        url: remote_url,
    };
    serde_json::to_string(&state)
        .map_err(|e| format!("Error serializing remote state: {}", e))
}
/// Persist the push state to the project's metadata.json.
///
/// Uses a read-modify-write pattern so existing metadata fields are
/// preserved. `consecutive_failures` is set to 0 when remote config is
/// saved (fresh start).
///
/// When `proyecto_path` is empty or metadata.json does not exist yet
/// (pre-creation flow), returns `Ok` without writing — the state will
/// be seeded by `crear_proyecto`.
///
/// The `url` parameter is accepted for backward-compatible signature
/// but is NOT stored — the remote URL lives in Git's own config.
#[tauri::command]
pub fn guardar_config_remoto(
    _app: tauri::AppHandle,
    proyecto_path: String,
    _url: String,
    push_enabled: bool,
) -> Result<String, String> {
    if proyecto_path.is_empty() {
        return Ok("No project path — state will be set after creation.".to_string());
    }
    let base = Path::new(&proyecto_path);
    let meta_path = base.join(".config").join("metadata.json");
    if !meta_path.exists() {
        return Ok("Metadata not created yet — state will be set after project creation.".to_string());
    }
    let raw = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Error reading metadata: {}", e))?;
    let mut meta: Metadata = serde_json::from_str(&raw)
        .map_err(|e| format!("Error parsing metadata: {}", e))?;
    meta.push_enabled = push_enabled;
    meta.consecutive_failures = 0;
    meta.last_modified = Local::now().to_rfc3339();
    let json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Error serializing metadata: {}", e))?;
    std::fs::write(&meta_path, json)
        .map_err(|e| format!("Error writing metadata: {}", e))?;
    Ok("Remote config saved successfully.".to_string())
}
/// Configure a Git remote and perform the initial push for a project.
///
/// Validates that the URL is an SSH URL (rejects HTTP/HTTPS). On valid
/// URL, adds the remote as `origin` and pushes the main branch with
/// upstream tracking.
///
/// If the push fails (e.g. remote unreachable), the local commit is
/// preserved and a warning is returned — the user can retry later.
#[tauri::command]
pub fn configurar_remoto(_app: tauri::AppHandle, path: String, url: String) -> Result<String, String> {
    // SSH URL validation: reject HTTP(S) — SSH is required
    let url_lower = url.to_lowercase();
    if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
        return Err(
            "Solo se admiten URLs SSH (git@... o ssh://...). Las URLs HTTPS no son compatibles."
                .to_string(),
        );
    }
    let project_path = Path::new(&path);
    let git_path = find_git()?;
    // 1a) Check if remote "origin" already exists
    let remote_exists = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map(|out| out.status.success() && !out.stdout.is_empty())
        .unwrap_or(false);
    // 1b) Add or set remote URL
    if remote_exists {
        // Remote already configured — just update the URL and fetch
        let set_output = system_command(&git_path)
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg(&url)
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Error al ejecutar git remote set-url: {}", e))?;
        if !set_output.status.success() {
            let stderr = String::from_utf8_lossy(&set_output.stderr);
            return Err(format!("Error al configurar el remoto: {}", stderr.trim()));
        }
        // Fetch the new remote to update tracking refs
        let _ = system_command(&git_path)
            .arg("fetch")
            .arg("origin")
            .current_dir(project_path)
            .output();
        return Ok("Remote actualizado correctamente.".to_string());
    }
    // Remote doesn't exist yet — add it and attempt initial push
    let add_output = system_command(&git_path)
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(&url)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git remote add: {}", e))?;
    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        return Err(format!("Error al configurar el remoto: {}", stderr.trim()));
    }
    // 1c) Check if remote already has commits (new remote flow only)
    let ls_output = system_command(&git_path)
        .arg("ls-remote")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git ls-remote: {}", e))?;
    if ls_output.status.success() {
        let ls_stdout = String::from_utf8_lossy(&ls_output.stdout);
        if ls_stdout.contains("refs/heads/main") || ls_stdout.contains("refs/heads/master") {
            // Remote has history — offer sync instead of failing on push
            return Err(format!(
                "REMOTE_HAS_COMMITS:El repositorio remoto ya contiene un historial previo. ¿Querés sincronizarlo con el proyecto local?"
            ));
        }
    }
    // If ls-remote fails (e.g. repo doesn't exist), we'll fall through to push
    // and let the push error handler deal with it
    // 2) git push -u origin main
    let push_output = system_command(&git_path)
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("main")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git push: {}", e))?;
    eprintln!("git push -u origin main output: {:?}", push_output);
    if push_output.status.success() {
        // Reset consecutive_failures on successful push
        let meta_path = project_path.join(".config").join("metadata.json");
        if let Ok(raw) = std::fs::read_to_string(&meta_path) {
            if let Ok(mut meta) = serde_json::from_str::<Metadata>(&raw) {
                meta.push_enabled = true;
                meta.consecutive_failures = 0;
                meta.last_modified = Local::now().to_rfc3339();
                if let Ok(json) = serde_json::to_string_pretty(&meta) {
                    let _ = std::fs::write(&meta_path, json);
                }
            }
        }
        Ok("Repositorio remoto configurado y sincronizado correctamente.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&push_output.stderr);
        let stderr_str = stderr.trim().to_lowercase();
        if stderr_str.contains("not found") || stderr_str.contains("repository not found") {
            Err(format!("REPO_NOT_FOUND:{}", stderr.trim()))
        } else {
            Err(format!("Error al sincronizar con remoto: {}", stderr.trim()))
        }
    }
}
/// Sync an existing remote repository that already has commits.
///
/// Called when `configurar_remoto` detects that the remote already has
/// a history (e.g. from another machine). Fetches the remote branch and
/// merges with `--allow-unrelated-histories --no-edit`.
///
/// On success: pushes the merged result to origin. On merge conflict:
/// aborts the merge and returns an error with the list of conflicted files.
#[tauri::command]
pub fn sincronizar_remoto(path: String) -> Result<String, String> {
    let project_path = Path::new(&path);
    let git_path = find_git()?;
    // 1) git fetch origin
    let fetch_output = system_command(&git_path)
        .arg("fetch")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git fetch: {}", e))?;
    if !fetch_output.status.success() {
        let stderr = String::from_utf8_lossy(&fetch_output.stderr);
        return Err(format!(
            "Error al obtener el historial remoto: {}",
            stderr.trim()
        ));
    }
    // 2) Determine the default branch on the remote
    let branch = "main"; // we always push to main
    // 3) git merge --allow-unrelated-histories --no-edit origin/main
    let merge_output = system_command(&git_path)
        .arg("merge")
        .arg("--allow-unrelated-histories")
        .arg("--no-edit")
        .arg(format!("origin/{}", branch))
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git merge: {}", e))?;
    if !merge_output.status.success() {
        // Conflict or other merge failure — abort
        let _ = system_command(&git_path)
            .arg("merge")
            .arg("--abort")
            .current_dir(project_path)
            .output();
        // Try to list conflicted files for a helpful message
        let conflict_info = if let Ok(diff) = system_command(&git_path)
            .arg("diff")
            .arg("--name-only")
            .arg("--diff-filter=U")
            .current_dir(project_path)
            .output()
        {
            let files = String::from_utf8_lossy(&diff.stdout);
            if !files.trim().is_empty() {
                format!("\nArchivos con diferencias:\n{}", files.trim())
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        return Err(format!(
            "No se pudo sincronizar automáticamente. Hay diferencias entre el historial local y el remoto que requieren resolución manual.{}",
            conflict_info
        ));
    }
    // 4) git push origin main
    let push_output = system_command(&git_path)
        .arg("push")
        .arg("origin")
        .arg(branch)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git push: {}", e))?;
    if !push_output.status.success() {
        let stderr = String::from_utf8_lossy(&push_output.stderr);
        return Err(format!(
            "Sincronización local completada, pero el push falló: {}",
            stderr.trim()
        ));
    }
    // Reset consecutive_failures on successful push
    let meta_path = project_path.join(".config").join("metadata.json");
    if let Ok(raw) = std::fs::read_to_string(&meta_path) {
        if let Ok(mut meta) = serde_json::from_str::<Metadata>(&raw) {
            meta.push_enabled = true;
            meta.consecutive_failures = 0;
            meta.last_modified = Local::now().to_rfc3339();
            if let Ok(json) = serde_json::to_string_pretty(&meta) {
                let _ = std::fs::write(&meta_path, json);
            }
        }
    }
    Ok("Historial remoto sincronizado correctamente.".to_string())
}
/// Retry a push to the configured remote after previous failures.
///
/// Resets the consecutive failure counter to 0 before attempting.
/// If no remote was ever configured, returns an error.
/// On success, the counter stays at 0. On failure, increments to 1
/// (starting a fresh strike count).
#[tauri::command]
pub fn reintentar_push(_app: tauri::AppHandle, path: String) -> Result<String, String> {
    let project_path = Path::new(&path);
    let meta_path = project_path.join(".config").join("metadata.json");
    if !meta_path.exists() {
        return Err("No hay metadata del proyecto.".to_string());
    }
    let raw = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Error reading metadata: {}", e))?;
    let mut meta: Metadata = serde_json::from_str(&raw)
        .map_err(|e| format!("Error parsing metadata: {}", e))?;
    // Check if remote is configured
    let git_path = find_git()?;
    let url_output = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git remote get-url: {}", e))?;
    if !url_output.status.success() {
        return Err("No hay un repositorio remoto configurado.".to_string());
    }
    let remote_url = String::from_utf8_lossy(&url_output.stdout).trim().to_string();
    if remote_url.is_empty() {
        return Err("No hay un repositorio remoto configurado.".to_string());
    }
    // Reset counter and enable push
    meta.consecutive_failures = 0;
    meta.push_enabled = true;
    let push_output = system_command(&git_path)
        .arg("push")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git push: {}", e))?;
    if push_output.status.success() {
        // Success: save with fresh counter
        meta.last_modified = Local::now().to_rfc3339();
        let json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Error serializing metadata: {}", e))?;
        std::fs::write(&meta_path, json)
            .map_err(|e| format!("Error writing metadata: {}", e))?;
        Ok("Sincronización exitosa.".to_string())
    } else {
        // Failure: increment to 1 (fresh count)
        meta.consecutive_failures = 1;
        meta.last_modified = Local::now().to_rfc3339();
        let json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Error serializing metadata: {}", e))?;
        std::fs::write(&meta_path, json)
            .map_err(|e| format!("Error writing metadata: {}", e))?;
        let stderr = String::from_utf8_lossy(&push_output.stderr);
        Err(format!("Error al sincronizar: {}", stderr.trim()))
    }
}
/// Save a checkpoint and push to the configured remote now.
///
/// Commits all pending changes (same as `crear_checkpoint`) and then
/// checks if local is ahead of remote. If ahead, pushes to `origin`.
/// Returns a combined result so the user gets immediate feedback.
#[tauri::command]
pub fn push_ahora(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let project_path = Path::new(&path);
    // 1) Commit pending changes (best-effort — never fails)
    let commit_msg = perform_commit(project_path).unwrap_or_default();
    // 2) Sync with remote: fetch → pull (if behind) → push (if ahead)
    match sync_with_remote(&app, &path, project_path) {
        Ok(warning) => {
            if warning.is_empty() {
                Ok(format!("✅ {}\n{}", commit_msg, "Sincronizado con el remoto."))
            } else {
                Ok(format!("⚠️ {}\n{}", commit_msg, warning))
            }
        }
        Err(e) => {
            Err(format!("Commit realizado, pero la sincronización falló: {}", e))
        }
    }
}
/// Fetch from origin and check if the remote has new commits.
///
/// Runs `git fetch origin` (best-effort — network failures are silent)
/// and compares `HEAD` with `origin/main`. Returns JSON with `has_updates`
/// (bool) and `behind_count` (number of commits local is behind).
#[tauri::command]
pub fn verificar_remoto(path: String) -> Result<String, String> {
    let project_path = Path::new(&path);
    let git_path = find_git()?;
    // Check remote exists
    let url_output = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git remote get-url: {}", e))?;
    if !url_output.status.success() {
        return Ok(r#"{"has_updates":false,"behind_count":0}"#.to_string());
    }
    // Fetch (silent — network errors are non-fatal)
    let _ = system_command(&git_path)
        .arg("fetch")
        .arg("origin")
        .current_dir(project_path)
        .output();
    // Count commits behind
    let count_output = system_command(&git_path)
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD..origin/main")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git rev-list: {}", e))?;
    let behind_str = String::from_utf8_lossy(&count_output.stdout).trim().to_string();
    let behind_count: u32 = behind_str.parse().unwrap_or(0);
    let has_updates = behind_count > 0;
    Ok(format!(
        r#"{{"has_updates":{},"behind_count":{}}}"#,
        has_updates, behind_count
    ))
}
/// Pull changes from the remote repository.
///
/// Runs `git pull origin main`. On success, returns a message with the
/// pull summary. On failure (conflicts, network), returns an error.
#[tauri::command]
pub fn traer_cambios(path: String) -> Result<String, String> {
    let project_path = Path::new(&path);
    let git_path = find_git()?;
    // Check for local uncommitted changes — pull could overwrite
    let status_output = system_command(&git_path)
        .arg("status")
        .arg("--porcelain")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git status: {}", e))?;
    let has_changes = !String::from_utf8_lossy(&status_output.stdout).trim().is_empty();
    if has_changes {
        return Err(
            "Hay cambios locales sin guardar. Guarda o descarta los cambios antes de sincronizar."
                .to_string(),
        );
    }
    let pull_output = system_command(&git_path)
        .arg("pull")
        .arg("origin")
        .arg("main")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git pull: {}", e))?;
    if pull_output.status.success() {
        let stdout = String::from_utf8_lossy(&pull_output.stdout);
        let summary: String = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        let msg = if summary.is_empty() {
            "Cambios sincronizados desde el remoto.".to_string()
        } else {
            format!("Cambios sincronizados: {}", summary)
        };
        Ok(msg)
    } else {
        let stderr = String::from_utf8_lossy(&pull_output.stderr);
        let stderr_str = stderr.trim();
        if stderr_str.contains("CONFLICT") || stderr_str.contains("conflict") {
            Err(format!(
                "Hay conflictos al sincronizar. Resuélvelos manualmente en la terminal:\n{}",
                stderr_str
            ))
        } else {
            Err(format!("Error al sincronizar: {}", stderr_str))
        }
    }
}
/// Internal checkpoint for close handler.
///
/// Commits local changes (best-effort), then collects session stats,
/// then checks if local is ahead of the remote and pushes if so.
/// Push warnings/errors are logged to stderr (the close handler cannot
/// surface UI to the user).
pub fn do_checkpoint(app: &tauri::AppHandle, project_path: &str) -> Result<String, String> {
    let path_buf = Path::new(project_path);
    eprintln!("[do_checkpoint] Starting checkpoint for: {}", project_path);
    // 1) Commit local changes (best-effort — never skips sync)
    let commit_result = perform_commit(path_buf);
    eprintln!("[do_checkpoint] Commit result: {:?}", commit_result);
    // 2) Collect session stats (best-effort — never blocks sync)
    {
        let state = app.state::<ProjectState>();
        let lock = state.session_tracker.lock();
        if let Ok(mut tracker) = lock {
            finalizar_sesion_escritura(&mut tracker, path_buf);
        }
    }
    // 3) Sync with remote — only if a remote is actually configured
    let has_remote = find_git().ok().map_or(false, |git| {
        system_command(&git)
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .current_dir(path_buf)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    });

    if has_remote {
        eprintln!("[do_checkpoint] Syncing with remote...");
        match sync_with_remote(app, project_path, path_buf) {
            Ok(warning) => {
                if !warning.is_empty() {
                    eprintln!("[do_checkpoint] Sync warning: {}", warning);
                } else {
                    eprintln!("[do_checkpoint] Sync completed successfully");
                }
            }
            Err(e) => {
                eprintln!("[do_checkpoint] Sync error: {}", e);
            }
        }
    } else {
        eprintln!("[do_checkpoint] No remote configured — skipping sync");
    }
    commit_result
}
/// Sync local branch with remote: fetch → pull (if behind) → push (if ahead).
///
/// Handles the full cycle so non-technical users never deal with diverged branches:
/// - Only behind: fast-forward pull to catch up
/// - Only ahead: push local commits
/// - Both ahead and behind: pull first (reduces divergence), then push what remains
/// - Up to date: nothing
///
/// Returns `Ok(warning)` if push produced a warning, `Ok("")` on clean sync.
/// Returns `Err` only on unexpected errors.
pub fn sync_with_remote(app: &tauri::AppHandle, path: &str, project_path: &Path) -> Result<String, String> {
    let git_path = find_git()?;
    // Check if origin remote exists
    let url_output = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output();
    if !url_output.map(|o| o.status.success()).unwrap_or(false) {
        return Ok("".to_string()); // No remote — nothing to sync
    }
    // Fetch (only if SSH agent is available)
    if ssh_available() {
        eprintln!("[sync] fetching origin...");
        let _ = system_command(&git_path)
            .arg("fetch")
            .arg("origin")
            .current_dir(project_path)
            .output();
    } else {
        eprintln!("[sync] no SSH agent, skipping fetch");
    }
    // Get upstream ref
    let upstream_ref = {
        let out = system_command(&git_path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("--symbolic-full-name")
            .arg("@{upstream}")
            .current_dir(project_path)
            .output();
        match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            _ => return Ok("".to_string()), // No upstream — nothing to sync
        }
    };
    eprintln!("[sync] upstream: {}", upstream_ref);
    // Get ahead/behind counts
    let (ahead, behind) = {
        let out = system_command(&git_path)
            .arg("rev-list")
            .arg("--count")
            .arg("--left-right")
            .arg(format!("{}...HEAD", upstream_ref))
            .current_dir(project_path)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let parts: Vec<&str> = s.split('\t').collect();
                let behind: u32 = parts.first().and_then(|p| p.parse().ok()).unwrap_or(0);
                let ahead: u32 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
                (ahead, behind)
            }
            _ => (0, 0),
        }
    };
    eprintln!("[sync] ahead={}, behind={}", ahead, behind);
    // If behind, pull first (fast-forward only — safe for non-technical users)
    if behind > 0 {
        eprintln!("[sync] behind by {} — pulling...", behind);
        let pull_out = system_command(&git_path)
            .arg("pull")
            .arg("--ff-only")
            .current_dir(project_path)
            .output();
        match pull_out {
            Ok(o) if o.status.success() => eprintln!("[sync] pull OK (fast-forward)"),
            _ => eprintln!("[sync] pull failed or not fast-forward (non-fatal)"),
        }
    }
    // Recalculate ahead after pull — HEAD may have moved forward
    let ahead_after_pull = {
        let out = system_command(&git_path)
            .arg("rev-list")
            .arg("--count")
            .arg("--left-right")
            .arg(format!("{}...HEAD", upstream_ref))
            .current_dir(project_path)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let parts: Vec<&str> = s.split('\t').collect();
                parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0)
            }
            _ => 0,
        }
    };
    eprintln!("[sync] ahead after pull={}", ahead_after_pull);
    // If still ahead, push
    if ahead_after_pull > 0 {
        eprintln!("[sync] ahead by {} — pushing...", ahead_after_pull);
        sincronizar_checkpoint(app, path)
    } else {
        eprintln!("[sync] nothing to push");
        Ok("".to_string())
    }
}
/// Internal helper: attempt to push to the configured remote.
///
/// Reads the remote URL from git, runs `git push`, and implements the
/// 3-strike rule (disables push after 3 consecutive failures).
///
/// Called by `do_checkpoint` (close) and `push_ahora` (button).
/// Does NOT check `push_enabled` — both callers are explicit user actions
/// that should always attempt push when ahead of remote.
pub fn sincronizar_checkpoint(_app: &tauri::AppHandle, path: &str) -> Result<String, String> {
    let project_path = Path::new(path);
    // Read state from project metadata (for 3-strike counter)
    let meta_path = project_path.join(".config").join("metadata.json");
    if !meta_path.exists() {
        return Ok("".to_string());
    }
    let raw = match std::fs::read_to_string(&meta_path) {
        Ok(r) => r,
        Err(_) => return Ok("".to_string()),
    };
    let mut meta: Metadata = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(_) => return Ok("".to_string()),
    };
    // Read remote URL from git
    let git_path = find_git()?;
    let url_output = system_command(&git_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git remote get-url: {}", e))?;
    if !url_output.status.success() {
        return Ok("".to_string()); // No remote configured
    }
    let remote_url = String::from_utf8_lossy(&url_output.stdout).trim().to_string();
    if remote_url.is_empty() {
        return Ok("".to_string());
    }
    // Attempt push
    let push_output = system_command(&git_path)
        .arg("push")
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Error al ejecutar git push: {}", e))?;
    eprintln!("git push output (sincronizar_checkpoint): {:?}", push_output);
    if push_output.status.success() {
        // Success: reset counter and re-enable (in case it was 3-strike disabled)
        meta.consecutive_failures = 0;
        meta.push_enabled = true;
    } else {
        // Failure: increment counter, apply 3-strike rule
        meta.consecutive_failures += 1;
        let failures = meta.consecutive_failures;
        let warning = if failures >= 3 {
            meta.push_enabled = false;
            "Sincronización remota desactivada tras 3 intentos fallidos. Podés reactivarla desde la barra de herramientas.".to_string()
        } else {
            format!(
                "No se pudo sincronizar con el remoto (intento {}/3).",
                failures
            )
        };
        meta.last_modified = Local::now().to_rfc3339();
        let json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Error serializing metadata: {}", e))?;
        std::fs::write(&meta_path, json)
            .map_err(|e| format!("Error writing metadata: {}", e))?;
        // Commit metadata changes so git status stays clean
        commit_metadata_file(project_path, &git_path);
        return Ok(warning);
    }
    // Common: write metadata after push success
    meta.last_modified = Local::now().to_rfc3339();
    let json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Error serializing metadata: {}", e))?;
    std::fs::write(&meta_path, json)
        .map_err(|e| format!("Error writing metadata: {}", e))?;
    // Commit metadata changes so git status stays clean
    commit_metadata_file(project_path, &git_path);
    Ok("".to_string())
}
/// Read the Git identity (name, email) from the global config file.
///
/// Returns `Some((name, email))` when a valid identity exists in the
/// config, or `None` when the config is missing, corrupted, or has no
/// identity section.
pub fn read_identity_from_config(app: &tauri::AppHandle) -> Option<(String, String)> {
    let config_path = get_config_path(app)?;
    if !config_path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&config_path).ok()?;
    let config: GitConfig = serde_json::from_str(&raw).ok()?;
    config.identity.map(|id| (id.name, id.email))
}
// ---------------------------------------------------------------------------
/// Resolve the path to the global git identity/remote config file.
///
/// Uses Tauri's platform-standard `app_config_dir()` under a `cron-insta/`
/// subdirectory. Returns `None` when the platform cannot determine the
/// config directory.
pub(crate) fn get_config_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|p| p.join("cron-insta").join("git-config.json"))
}
/// Return the list of .md files changed in a given commit.
pub fn get_changed_md_files(
    project_path: &Path,
    git_path: &str,
    hash: &str,
) -> Vec<String> {
    let output = system_command(git_path)
        .arg("show")
        .arg("--name-only")
        .arg("--format=")
        .arg(hash)
        .current_dir(project_path)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.is_empty() && l.ends_with(".md"))
                .map(|l| {
                    // Show just the filename, not the full path
                    Path::new(l)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| l.to_string())
                })
                .collect()
        }
        _ => vec![],
    }
}

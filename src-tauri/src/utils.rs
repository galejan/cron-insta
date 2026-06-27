// Cron-Insta — Utility functions
//
// Pure functions with no tauri::AppHandle dependency.
// Used by command modules for common operations.

use chrono::Local;
use std::path::Path;
use std::process::Command;

use crate::models::*;

// ---------------------------------------------------------------------------
// Process helper — hides terminal windows on Windows
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Create a `Command` pre-configured for headless execution:
/// - `CREATE_NO_WINDOW` on Windows (prevents console popups)
/// - `stdin` set to null (prevents accidental blocking on stdin reads)
pub fn system_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(std::process::Stdio::null());
    // Inherit SSH agent socket for git operations on Linux
    let ssh_sock = std::env::var("SSH_AUTH_SOCK").ok()
        .or_else(find_ssh_auth_sock_fallback);
    if let Some(sock) = ssh_sock {
        cmd.env("SSH_AUTH_SOCK", sock);
    }
    // Never prompt for password, timeout after 5s to avoid 30s hangs
    cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes -o ConnectTimeout=5");
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Fallback: try common SSH_AUTH_SOCK paths when the env var is not set.
/// This covers desktop-launched Tauri apps that don't inherit the terminal env.
pub fn find_ssh_auth_sock_fallback() -> Option<String> {
    let uid = std::fs::read_to_string("/proc/self/loginuid").ok()?;
    let uid = uid.trim();
    let candidates = [
        format!("/run/user/{}/keyring/ssh", uid),
        format!("/run/user/{}/ssh-agent.socket", uid),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }
    None
}

/// Check if SSH agent is available for git network operations.
/// Returns `true` if `SSH_AUTH_SOCK` is set (env or fallback path).
/// When unavailable, all SSH git ops (fetch, push) will fail — skip them early.
pub fn ssh_available() -> bool {
    std::env::var("SSH_AUTH_SOCK").is_ok() || find_ssh_auth_sock_fallback().is_some()
}

// ---------------------------------------------------------------------------
// Git binary locator
// ---------------------------------------------------------------------------

/// Locate the `git` executable on the system.
///
/// **Linux**: uses `which git`.
/// **Windows**: tries `PATH` via `where git`, then falls back to the two
/// standard Git-for-Windows installation paths.
///
/// Returns `Ok(path)` when found, or `Err(msg)` with a user-facing Spanish
/// message when Git is unavailable.
pub fn find_git() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let output = system_command("which")
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
        if let Ok(output) = system_command("where").arg("git").output() {
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
// Trama ID slugifier
// ---------------------------------------------------------------------------

/// Generate a unique, URL-safe trama ID from a display name.
///
/// Slugifies the name (lowercase, hyphens, strip non-alnum) and appends an
/// 8-char hex suffix derived from the current timestamp for uniqueness.
pub fn slugify_trama_id(nombre: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Strip accents by decomposing characters and removing combining marks
    let decomposed: String = nombre
        .chars()
        .flat_map(|c| {
            let mut buf = [0u8; 4];
            let _s = c.encode_utf8(&mut buf);
            // Check if this is a base letter + combining accent by
            // looking at the decomposition of common accented chars
            match c {
                'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => vec!['a'],
                'é' | 'è' | 'ë' | 'ê' => vec!['e'],
                'í' | 'ì' | 'ï' | 'î' => vec!['i'],
                'ó' | 'ò' | 'ö' | 'ô' | 'õ' => vec!['o'],
                'ú' | 'ù' | 'ü' | 'û' => vec!['u'],
                'ñ' => vec!['n'],
                'ç' => vec!['c'],
                'ý' | 'ÿ' => vec!['y'],
                _ if c.is_alphabetic() && c as u32 > 127 => {
                    // For unknown non-ASCII alphabetic chars, try NFKD decomposition
                    // using a simple approach: keep only ASCII letters
                    let lower = c.to_lowercase().to_string();
                    lower.chars().filter(|ch| ch.is_ascii_alphabetic()).collect()
                }
                _ => vec![c],
            }
        })
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect();
    let base = decomposed
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>();
    let base = base
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{}-{:08x}", base, nanos)
}

// ---------------------------------------------------------------------------
// Commit helpers
// ---------------------------------------------------------------------------

/// Core commit logic: stage changes, create a descriptive commit, and
/// return the commit hash (or a "no changes" message).
///
/// Used by both `crear_checkpoint` (Tauri command) and `do_checkpoint`
/// (close-handler helper) so the commit logic lives in one place.
pub fn perform_commit(project_path: &Path) -> Result<String, String> {
    // Best-effort: never returns Err. On failure, logs via eprintln! and
    // returns Ok with a status message so callers never skip push.
    let git_path = match find_git() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[perform_commit] find_git error: {}", e);
            return Ok("Git no está disponible.".to_string());
        }
    };

    // Stage all changes
    let add_output = match system_command(&git_path)
        .arg("add")
        .arg(".")
        .current_dir(project_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[perform_commit] git add error: {}", e);
            return Ok(format!("Error al ejecutar git add: {}", e));
        }
    };

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        let msg = format!("Error en git add: {}", stderr.trim());
        eprintln!("[perform_commit] {}", msg);
        return Ok(msg);
    }

    // Count words in chapter files for the commit message
    let word_count = count_words_in_chapters(project_path);
    let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let commit_msg = format!(
        "Progreso automático: {} - {} palabras",
        date, word_count
    );

    // Commit
    let commit_output = match system_command(&git_path)
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
        .current_dir(project_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[perform_commit] git commit error: {}", e);
            return Ok(format!("Error al ejecutar git commit: {}", e));
        }
    };

    if commit_output.status.success() {
        // Retrieve the commit hash
        let hash_output = match system_command(&git_path)
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[perform_commit] git rev-parse error: {}", e);
                return Ok("Commit realizado, pero no se pudo obtener el hash.".to_string());
            }
        };

        let hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();
        Ok(hash)
    } else {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        let stdout = String::from_utf8_lossy(&commit_output.stdout);
        let combined = format!("{}{}", stderr, stdout);
        // "nothing to commit" is a normal state, not an error.
        if combined.contains("nothing to commit")
            || combined.contains("nothing added to commit")
            || combined.contains("nada para confirmar")
            || combined.contains("nada que confirmar")
        {
            Ok("Sin cambios para guardar.".to_string())
        } else {
            let msg = format!(
                "Error en git commit: {}",
                combined.trim().lines().last().unwrap_or("")
            );
            eprintln!("[perform_commit] {}", msg);
            Ok(msg)
        }
    }
}

/// Stage and commit the `.config/metadata.json` file after a push state update.
/// This keeps the working tree clean — metadata changes are always versioned
/// alongside the content changes they describe.
pub fn commit_metadata_file(project_path: &Path, git_exe: &str) {
    // Stage metadata.json
    let meta_rel = Path::new(".config").join("metadata.json");
    let add_result = system_command(git_exe)
        .arg("add")
        .arg(&meta_rel)
        .current_dir(project_path)
        .output();
    match add_result {
        Ok(o) if o.status.success() => eprintln!("[commit_metadata] staged OK"),
        _ => {
            eprintln!("[commit_metadata] git add failed (non-fatal)");
            return;
        }
    }

    // Commit metadata. If nothing changed (already committed), "nothing to commit" is fine.
    let commit_result = system_command(git_exe)
        .arg("commit")
        .arg("-m")
        .arg("cron-insta: actualizar estado de sincronización")
        .current_dir(project_path)
        .output();
    match commit_result {
        Ok(o) if o.status.success() => eprintln!("[commit_metadata] committed OK"),
        _ => eprintln!("[commit_metadata] git commit skipped (no metadata changes)"),
    }
}

// ---------------------------------------------------------------------------
// Word counting helpers
// ---------------------------------------------------------------------------

/// Count text words inside HTML content by stripping tags first.
///
/// Two-state char-by-char machine: skip everything between `<` and `>`,
/// collect the remaining text, then split on Unicode whitespace.
/// No regex or HTML parser dependency.
pub fn count_words_in_html(html: &str) -> u64 {
    let mut inside_tag = false;
    let mut text = String::with_capacity(html.len());
    for ch in html.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(ch),
            _ => {}
        }
    }
    text.split_whitespace().count() as u64
}

/// Read a chapter file and return its word count via `count_words_in_html`.
/// Returns 0 when the file is missing or unreadable.
pub fn word_count_chapter(project_path: &Path, filename: &str) -> u64 {
    let file_path = project_path.join("capitulos").join(filename);
    match std::fs::read_to_string(&file_path) {
        Ok(content) => count_words_in_html(&content),
        Err(_) => 0,
    }
}

/// Count whitespace-separated tokens across all `.md` files under
/// `{project_path}/capitulos/`.  Returns 0 when the directory is
/// missing or empty.
pub fn count_words_in_chapters(project_path: &Path) -> usize {
    let cap_dir = project_path.join("capitulos");
    if !cap_dir.exists() {
        return 0;
    }

    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(&cap_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    total += content.split_whitespace().count();
                }
            }
        }
    }
    total
}

/// Helper: read project metadata from disk.
pub fn read_metadata(base: &Path) -> Result<Metadata, String> {
    let meta_path = base.join(".config").join("metadata.json");
    let raw = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Error al leer metadata: {}", e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Error al parsear metadata: {}", e))
}

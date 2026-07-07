use std::path::PathBuf;

/// A single keyboard shortcut binding.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ShortcutBinding {
    /// Unique action identifier (e.g., "new-character", "export-zip")
    pub id: String,
    /// Keyboard key name — use KeyboardEvent.key values ("c", "ArrowLeft", "Enter", "F1", etc.)
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// Human-readable description (Spanish)
    pub label_es: String,
    /// Human-readable description (English)
    pub label_en: String,
}

/// Get the path to the shortcuts config file in the app's config directory.
fn shortcuts_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    // Use the same config directory as the Git identity config.
    // Call the existing get_config_path function from commands/git.rs.
    // If it returns Some(p), replace the filename: p.parent() / "shortcuts.json"
    crate::commands::git::get_config_path(app).map(|p| {
        p.parent()
            .map(|parent| parent.join("shortcuts.json"))
            .unwrap_or_else(|| PathBuf::from("shortcuts.json"))
    })
}

/// Default shortcuts — the authoritative set. This is the single source of truth.
fn default_shortcuts() -> Vec<ShortcutBinding> {
    vec![
        // ── Sidebar ──────────────────────────────────────────
        ShortcutBinding {
            id: "sidebar-collapse".into(),
            key: "ArrowLeft".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Colapsar panel lateral".into(),
            label_en: "Collapse sidebar".into(),
        },
        ShortcutBinding {
            id: "sidebar-expand".into(),
            key: "ArrowRight".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Panel lateral completo".into(),
            label_en: "Full sidebar".into(),
        },
        ShortcutBinding {
            id: "sidebar-shrink".into(),
            key: "ArrowLeft".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Reducir panel lateral".into(),
            label_en: "Shrink sidebar".into(),
        },
        ShortcutBinding {
            id: "sidebar-grow".into(),
            key: "ArrowRight".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Ampliar panel lateral".into(),
            label_en: "Grow sidebar".into(),
        },
        // ── Navigation ───────────────────────────────────────
        ShortcutBinding {
            id: "prev-chapter".into(),
            key: "ArrowLeft".into(),
            ctrl: false,
            shift: false,
            alt: true,
            label_es: "Capítulo anterior".into(),
            label_en: "Previous chapter".into(),
        },
        ShortcutBinding {
            id: "next-chapter".into(),
            key: "ArrowRight".into(),
            ctrl: false,
            shift: false,
            alt: true,
            label_es: "Capítulo siguiente".into(),
            label_en: "Next chapter".into(),
        },
        ShortcutBinding {
            id: "cycle-tabs".into(),
            key: "t".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Cambiar pestaña lateral".into(),
            label_en: "Cycle sidebar tab".into(),
        },
        // ── Editor ───────────────────────────────────────────
        ShortcutBinding {
            id: "save".into(),
            key: "s".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Guardar ahora".into(),
            label_en: "Save now".into(),
        },
        ShortcutBinding {
            id: "heading-up".into(),
            key: "ArrowUp".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Subir nivel de título".into(),
            label_en: "Increase heading".into(),
        },
        ShortcutBinding {
            id: "heading-down".into(),
            key: "ArrowDown".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Bajar nivel de título".into(),
            label_en: "Decrease heading".into(),
        },
        ShortcutBinding {
            id: "zoom-in".into(),
            key: "+".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Aumentar zoom".into(),
            label_en: "Zoom in".into(),
        },
        ShortcutBinding {
            id: "zoom-out".into(),
            key: "-".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Reducir zoom".into(),
            label_en: "Zoom out".into(),
        },
        ShortcutBinding {
            id: "dialogue-dash".into(),
            key: "d".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Insertar guion de diálogo".into(),
            label_en: "Insert dialogue dash".into(),
        },
        ShortcutBinding {
            id: "bold".into(),
            key: "b".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Negrita".into(),
            label_en: "Bold".into(),
        },
        ShortcutBinding {
            id: "italic".into(),
            key: "i".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Cursiva".into(),
            label_en: "Italic".into(),
        },
        // ── Project operations ───────────────────────────────
        ShortcutBinding {
            id: "new-chapter".into(),
            key: "n".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Nuevo capítulo".into(),
            label_en: "New chapter".into(),
        },
        ShortcutBinding {
            id: "open-project".into(),
            key: "o".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Abrir proyecto".into(),
            label_en: "Open project".into(),
        },
        ShortcutBinding {
            id: "new-project".into(),
            key: "N".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nuevo proyecto".into(),
            label_en: "New project".into(),
        },
        ShortcutBinding {
            id: "import-project".into(),
            key: "i".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Importar proyecto".into(),
            label_en: "Import project".into(),
        },
        ShortcutBinding {
            id: "close-project".into(),
            key: "w".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Cerrar proyecto".into(),
            label_en: "Close project".into(),
        },
        ShortcutBinding {
            id: "export-zip".into(),
            key: "e".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Exportar proyecto (ZIP)".into(),
            label_en: "Export project (ZIP)".into(),
        },
        ShortcutBinding {
            id: "export-md".into(),
            key: "E".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Exportar proyecto (Markdown)".into(),
            label_en: "Export project (Markdown)".into(),
        },
        ShortcutBinding {
            id: "project-settings".into(),
            key: ",".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Configuración del proyecto".into(),
            label_en: "Project settings".into(),
        },
        ShortcutBinding {
            id: "global-settings".into(),
            key: ",".into(),
            ctrl: true,
            shift: false,
            alt: true,
            label_es: "Ajustes globales".into(),
            label_en: "Global settings".into(),
        },
        ShortcutBinding {
            id: "repair-project".into(),
            key: "R".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Reparar proyecto".into(),
            label_en: "Repair project".into(),
        },
        // ── Create entities ──────────────────────────────────
        ShortcutBinding {
            id: "new-character".into(),
            key: "C".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nuevo personaje".into(),
            label_en: "New character".into(),
        },
        ShortcutBinding {
            id: "new-place".into(),
            key: "L".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nuevo lugar".into(),
            label_en: "New place".into(),
        },
        ShortcutBinding {
            id: "new-note".into(),
            key: "M".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nueva nota".into(),
            label_en: "New note".into(),
        },
        ShortcutBinding {
            id: "new-event".into(),
            key: "E".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nuevo evento".into(),
            label_en: "New timeline event".into(),
        },
        ShortcutBinding {
            id: "new-trama".into(),
            key: "G".into(),
            ctrl: true,
            shift: true,
            alt: false,
            label_es: "Nueva trama".into(),
            label_en: "New plotline".into(),
        },
        // ── UI toggles ───────────────────────────────────────
        ShortcutBinding {
            id: "dock".into(),
            key: "Enter".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Pinear/despinear elemento".into(),
            label_en: "Dock/undock element".into(),
        },
        ShortcutBinding {
            id: "toggle-footer".into(),
            key: "p".into(),
            ctrl: true,
            shift: false,
            alt: false,
            label_es: "Mostrar/ocultar panel inferior".into(),
            label_en: "Toggle footer panel".into(),
        },
        ShortcutBinding {
            id: "toggle-help".into(),
            key: "F1".into(),
            ctrl: false,
            shift: false,
            alt: false,
            label_es: "Ayuda".into(),
            label_en: "Help".into(),
        },
        ShortcutBinding {
            id: "help-question".into(),
            key: "?".into(),
            ctrl: false,
            shift: true,
            alt: false,
            label_es: "Ayuda (tecla ?)".into(),
            label_en: "Help (? key)".into(),
        },
        ShortcutBinding {
            id: "toggle-fullscreen".into(),
            key: "F11".into(),
            ctrl: false,
            shift: false,
            alt: false,
            label_es: "Pantalla completa".into(),
            label_en: "Full screen".into(),
        },
    ]
}

/// Load shortcuts: merge defaults with user overrides from shortcuts.json.
/// Returns the merged shortcut bindings. Tauri serializes the Vec to JSON automatically.
#[tauri::command]
pub fn cargar_atajos(app: tauri::AppHandle) -> Result<Vec<ShortcutBinding>, String> {
    let defaults = default_shortcuts();
    let mut map: std::collections::HashMap<String, ShortcutBinding> =
        defaults.into_iter().map(|s| (s.id.clone(), s)).collect();

    if let Some(path) = shortcuts_path(&app) {
        if path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(overrides) = serde_json::from_str::<Vec<ShortcutBinding>>(&raw) {
                    for ov in overrides {
                        map.insert(ov.id.clone(), ov);
                    }
                }
            }
        }
    }

    let merged: Vec<ShortcutBinding> = map.into_values().collect();
    Ok(merged)
}

/// Save a single shortcut override to the config file.
/// Reads existing overrides, updates the one with matching id, writes back.
#[tauri::command]
pub fn guardar_atajo(app: tauri::AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let path =
        shortcuts_path(&app).ok_or("No se pudo determinar el directorio de configuración.")?;
    let mut overrides: Vec<ShortcutBinding> = if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("Error reading shortcuts: {}", e))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };
    overrides.retain(|s| s.id != binding.id);
    overrides.push(binding);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Error creating config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&overrides)
        .map_err(|e| format!("Error serializing: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Error writing shortcuts: {}", e))
}

/// Reset all shortcuts to defaults (delete overrides file).
#[tauri::command]
pub fn restaurar_atajos(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(path) = shortcuts_path(&app) {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Error removing shortcuts file: {}", e))?;
        }
    }
    Ok(())
}

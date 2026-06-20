# Cronista · Editor literario Local-First

**Cronista** es un editor de texto enriquecido diseñado para escribir novelas largas. Combina una zona de escritura libre de distracciones con un panel lateral que integra capítulos, fichas de personajes, notas y una línea de tiempo. Todo se guarda en local: archivos Markdown y JSON dentro de una carpeta de proyecto, sin depender de la nube.

---

## Qué lo hace distinto

| Principio | En la práctica |
|-----------|----------------|
| **Zona de escritura limpia** | La mayor parte de la pantalla es solo para escribir. Sin menús, sin barra de formato. |
| **Sidebar integrado** | Capítulos, personajes, notas y línea de tiempo en el panel lateral. Con drag and drop en la línea temporal. |
| **Local-First** | Todo en tu disco. `.md` para el texto, `.json` para índices y metadatos. |
| **Git invisible** | Cada cierre de la aplicación crea un checkpoint automático. Historial completo sin intervención manual. |
| **TipTap como motor** | Editor WYSIWYG con títulos semánticos (H1, H2). Limpio, sin distracciones de formato. |
| **Exportación integrada** | Exportá el proyecto completo en `.zip` o compartí solo los capítulos en un único `.md`. |
| **Accesibilidad** | Zoom de interfaz con `Ctrl+=` / `Ctrl+-`. Tres niveles para adaptarse a cada vista. |

---

## Instalación

### Requisitos

- [Rust](https://rustup.rs) (stable)
- [Node.js](https://nodejs.org) ≥ 18
- [pnpm](https://pnpm.io/installation)
- Dependencias de sistema para Tauri v2 ([guía oficial](https://v2.tauri.app/start/prerequisites/))

### Desarrollo

```bash
git clone git@github.com:galejan/cronista.git
cd cronista
pnpm install
pnpm tauri dev
```

### Build para producción

```bash
pnpm tauri build
```

El binario se genera en `src-tauri/target/release/`. En Arch Linux se puede ejecutar directamente (`./cronista`) o crear un archivo `.desktop` a mano.

---

## Estructura del proyecto

```
cronista/
├── src/                  # Frontend SvelteKit
│   ├── lib/
│   │   ├── components/   # Editor.svelte (TipTap wrapper)
│   │   ├── i18n.svelte.ts # Traducciones ES/EN con $state runes
│   │   ├── tauri.ts      # Wrappers de comandos Tauri
│   │   └── debounce.ts
│   └── routes/
│       └── +page.svelte  # Layout principal (sidebar + editor)
├── src-tauri/            # Backend Rust
│   └── src/
│       ├── lib.rs        # ~30 comandos Tauri + lógica de archivos
│       └── main.rs       # Entry point
├── docs/                 # Documentación de diseño
├── openspec/             # Artefactos SDD
└── static/               # Iconos y assets
```

### Stack técnico

| Capa | Tecnología |
|------|-----------|
| Escritorio | [Tauri v2](https://v2.tauri.app) (Rust) |
| Frontend | [SvelteKit](https://kit.svelte.dev) + [Svelte 5](https://svelte.dev) |
| Editor | [TipTap v3](https://tiptap.dev) (ProseMirror) |
| Estilos | CSS plano (sin dependencias de componentes) |
| Lenguajes | Rust, TypeScript |
| i18n | Sistema propio con Svelte 5 `$state` runes (ES/EN) |

---

## Atajos de teclado

| Atajo | Acción |
|-------|--------|
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>←</kbd> | Colapsar panel lateral |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>→</kbd> | Restaurar panel lateral |
| <kbd>Ctrl</kbd> + <kbd>←</kbd> / <kbd>→</kbd> | Reducir / ampliar panel lateral (5 %) |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> | Mostrar / ocultar panel de herramientas |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Guardar ahora |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Nuevo capítulo |
| <kbd>Ctrl</kbd> + <kbd>O</kbd> | Abrir otro proyecto (cierra el actual) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd> | Nuevo proyecto (cierra el actual) |
| <kbd>Ctrl</kbd> + <kbd>↑</kbd> / <kbd>↓</kbd> | Subir / bajar nivel de título |
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Insertar guion de diálogo (`—`) |
| <kbd>Ctrl</kbd> + <kbd>=</kbd> / <kbd>-</kbd> | Aumentar / reducir tamaño de letra |
| <kbd>F11</kbd> | Pantalla completa |
| <kbd>F1</kbd> o <kbd>?</kbd> | Mostrar / ocultar panel de ayuda |

---

## Comandos del backend

El backend Rust expone los siguientes comandos Tauri:

- **Proyecto**: `crear_proyecto`, `set_active_project`, `marcar_proyecto_cronista`
- **Git**: `detectar_git`, `inicializar_git`, `inicializar_git_con_autor`, `crear_checkpoint`, `verificar_git_inicializado`, `obtener_git_log`
- **Capítulos**: `guardar_capitulo`, `cargar_capitulo`, `crear_capitulo`, `eliminar_capitulo`, `cargar_indice`
- **Personajes**: `listar_personajes`, `crear_personaje`, `cargar_personaje`, `actualizar_personaje`, `eliminar_personaje`
- **Notas**: `listar_notas`, `crear_nota`, `cargar_nota`, `eliminar_nota`
- **Timeline**: `cargar_timeline`, `agregar_evento_timeline`, `reordenar_timeline`, `eliminar_evento_timeline`
- **Exportación**: `exportar_proyecto_zip`, `exportar_proyecto_md`

---

## Licencia

MIT © 2026 — [github.com/galejan/cronista](https://github.com/galejan/cronista)

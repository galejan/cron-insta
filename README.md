# Cronista · Editor literario Local-First

**Cronista** es un editor de texto enriquecido diseñado para escribir novelas largas. Combina una zona de escritura libre de distracciones con un panel lateral que integra capítulos, fichas de personajes, notas y una línea de tiempo. Todo se guarda en local: archivos Markdown y JSON dentro de una carpeta de proyecto, sin depender de la nube.

---

## Qué lo hace distinto

| Principio | En la práctica |
|-----------|----------------|
| **Zona de escritura sagrada** | El 60 % de la pantalla es solo para escribir. Sin menús, sin ruido. |
| **Sidebar integrado** | Capítulos, personajes, notas y línea de tiempo en el 40 % restante. Arrastrá, reordená, alterná. |
| **Local-First** | Todo en tu disco. `.md` para el texto, `.json` para índices y metadatos. |
| **Git invisible** | Cada checkpoint es un commit. Si no hay Git, no se intenta nada. |
| **TipTap como motor** | Editor WYSIWYG con formato semántico (Heading 1–3, negrita, cursiva, listas). |

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
│   │   ├── i18n.ts       # Traducciones ES/EN
│   │   ├── tauri.ts      # Wrappers de comandos Tauri
│   │   └── debounce.ts
│   └── routes/
│       └── +page.svelte  # Layout principal (60/40)
├── src-tauri/            # Backend Rust
│   └── src/
│       ├── lib.rs        # 22 comandos Tauri + lógica de archivos
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
| Estilos | Tailwind CSS v4 (PostCSS) |
| Lenguajes | Rust, TypeScript |
| i18n | Sistema propio con Svelte writable stores (ES/EN) |

---

## Atajos de teclado

| Atajo | Acción |
|-------|--------|
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Colapsar / restaurar panel lateral |
| <kbd>Ctrl</kbd> + <kbd>←</kbd> | Reducir panel lateral (5 %) |
| <kbd>Ctrl</kbd> + <kbd>→</kbd> | Ampliar panel lateral (5 %) |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Guardar ahora |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Nuevo capítulo |
| <kbd>Ctrl</kbd> + <kbd>O</kbd> | Abrir otro proyecto (cierra el actual) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd> | Nuevo proyecto (cierra el actual) |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>1</kbd> / <kbd>2</kbd> / <kbd>3</kbd> | Aplicar Título 1 / 2 / 3 al párrafo actual |
| <kbd>F11</kbd> | Pantalla completa |
| <kbd>F1</kbd> o <kbd>?</kbd> | Mostrar / ocultar panel de ayuda |

---

## Comandos del backend

El backend Rust expone 22 comandos Tauri organizados en estos grupos:

- **Proyecto**: `crear_proyecto`, `detectar_git`, `inicializar_git`, `crear_checkpoint`
- **Capítulos**: `guardar_capitulo`, `cargar_capitulo`, `crear_capitulo`, `eliminar_capitulo`, `cargar_indice`
- **Personajes**: `listar_personajes`, `crear_personaje`, `cargar_personaje`, `actualizar_personaje`, `eliminar_personaje`
- **Notas**: `listar_notas`, `crear_nota`, `cargar_nota`, `eliminar_nota`
- **Timeline**: `cargar_timeline`, `agregar_evento_timeline`, `reordenar_timeline`, `eliminar_evento_timeline`

---

## Licencia

MIT © 2026 — [github.com/galejan/cronista](https://github.com/galejan/cronista)

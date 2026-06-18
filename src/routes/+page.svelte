<script lang="ts">
  import Editor from "$lib/components/Editor.svelte";
  import { debounce } from "$lib/debounce";
  import { cargarCapitulo, crearCapitulo, guardarCapitulo } from "$lib/tauri";

  let sidebarVisible = $state(true);

  // ── Editor & project state ──────────────────────────────────
  let projectPath = $state("");
  let chapters = $state<string[]>([]);
  let activeChapter = $state("");
  let editorContent = $state("");
  let saveStatus = $state<"" | "saved" | "unsaved" | "saving">("");

  /** Editor component reference — exposes setContent(html). */
  let editorRef = $state<{ setContent(html: string): void }>();

  // ── Debounced auto-save (2 s after last keystroke) ──────────
  const save = debounce(async () => {
    if (!projectPath || !activeChapter) return;
    saveStatus = "saving";
    try {
      await guardarCapitulo(projectPath, activeChapter, editorContent);
      saveStatus = "saved";
    } catch (e) {
      console.error("Save failed:", e);
      saveStatus = "unsaved";
    }
  }, 2_000);

  // ── Editor callbacks ────────────────────────────────────────
  function handleEditorUpdate(html: string): void {
    editorContent = html;
    saveStatus = "unsaved";
    save.trigger();
  }

  // ── Chapter operations ──────────────────────────────────────
  async function cargarCapituloActual(filename: string): Promise<void> {
    if (!projectPath) return;
    save.cancel();
    try {
      const content = await cargarCapitulo(projectPath, filename);
      editorRef?.setContent(content);
      activeChapter = filename;
      editorContent = content;
      saveStatus = "saved";
    } catch (e) {
      console.error("Failed to load chapter:", e);
    }
  }

  async function crearCapituloNuevo(): Promise<void> {
    if (!projectPath) {
      const p = prompt("Ruta del proyecto (ej: /tmp/mi-novela):");
      if (!p) return;
      projectPath = p.trim();
    }

    const filename = prompt("Nombre del archivo (ej: 0003_capitulo_3.md):");
    if (!filename) return;

    // Simple heading + empty paragraph so the editor isn't blank.
    const initialHTML = "<h1>Sin título</h1><p></p>";

    try {
      await crearCapitulo(projectPath, filename, initialHTML);
      activeChapter = filename;
      editorRef?.setContent(initialHTML);
      editorContent = initialHTML;
      saveStatus = "saved";
      chapters = [...chapters, filename];
    } catch (e) {
      console.error("Create chapter failed:", e);
      alert(`Error al crear capítulo: ${e}`);
    }
  }

  // ── Keyboard shortcuts ──────────────────────────────────────
  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === "b") {
      e.preventDefault();
      sidebarVisible = !sidebarVisible;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-layout" class:sidebar-collapsed={!sidebarVisible}>
  <!-- Sidebar (40 % when visible) — placeholder, not modified per spec -->
  <aside class="sidebar">
    <nav class="tabs">
      <button class="tab active">Capítulos</button>
      <button class="tab">Personajes</button>
      <button class="tab">Notas</button>
    </nav>
    <div class="sidebar-content">
      <!-- Test button: load a chapter by filename (task 5.4) -->
      <div class="sidebar-toolbar">
        {#if chapters.length > 0}
          <p class="chapter-list-label">Capítulos:</p>
          <ul class="chapter-list">
            {#each chapters as ch}
              <li>
                <button
                  class="chapter-link"
                  class:active-chapter={activeChapter === ch}
                  onclick={() => cargarCapituloActual(ch)}
                >
                  {ch}
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        <button class="btn-load" onclick={() => {
          const fn = prompt("Nombre del archivo a cargar (ej: 0001_prologo.md):");
          if (fn) cargarCapituloActual(fn.trim());
        }}>
          Cargar capítulo
        </button>
      </div>
    </div>
  </aside>

  <!-- Editor area (60 % when visible, 100 % when sidebar collapsed) -->
  <main class="editor">
    {#if !projectPath}
      <!-- First launch: prompt for project path -->
      <div class="setup-prompt">
        <p class="setup-text">Seleccioná una carpeta de proyecto para comenzar</p>
        <button
          class="btn-primary"
          onclick={() => crearCapituloNuevo()}
        >
          Configurar proyecto
        </button>
      </div>
    {:else}
      <!-- Toolbar + Editor -->
      <div class="editor-pane">
        <div class="editor-toolbar">
          <div class="toolbar-left">
            <span class="project-label" title={projectPath}>
              {projectPath.split("/").pop() || projectPath}
            </span>
            <button class="toolbar-btn" onclick={crearCapituloNuevo}>
              + Nuevo capítulo
            </button>
          </div>

          <div class="toolbar-right">
            {#if activeChapter}
              <span class="chapter-label">{activeChapter}</span>
            {/if}
            <span
              class="save-indicator"
              class:saving={saveStatus === "saving"}
              class:saved={saveStatus === "saved"}
              class:unsaved={saveStatus === "unsaved"}
            >
              {saveStatus === "saving"
                ? "Guardando…"
                : saveStatus === "saved"
                  ? "Guardado"
                  : saveStatus === "unsaved"
                    ? "Sin guardar"
                    : ""}
            </span>
          </div>
        </div>

        <div class="editor-body">
          <Editor
            bind:this={editorRef}
            content={editorContent}
            onUpdate={handleEditorUpdate}
          />
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  /* ── Layout ────────────────────────────────────────────────── */
  .app-layout {
    display: grid;
    grid-template-columns: 40% 60%;
    height: 100vh;
    transition: grid-template-columns 300ms ease;
  }

  .app-layout.sidebar-collapsed {
    grid-template-columns: 0% 100%;
  }

  /* ── Sidebar ───────────────────────────────────────────────── */
  .sidebar {
    overflow: hidden;
    border-right: 1px solid #e2e8f0;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  @media (prefers-color-scheme: dark) {
    .sidebar {
      border-right-color: #334155;
    }
  }

  .sidebar-collapsed .sidebar {
    border-right: none;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid #e2e8f0;
  }

  @media (prefers-color-scheme: dark) {
    .tabs {
      border-bottom-color: #334155;
    }
  }

  .tab {
    flex: 1;
    padding: 0.75rem 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: #64748b;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 150ms, border-color 150ms;
  }

  .tab:hover {
    color: #1e293b;
  }

  .tab.active {
    color: #3b82f6;
    border-bottom-color: #3b82f6;
  }

  @media (prefers-color-scheme: dark) {
    .tab {
      color: #94a3b8;
    }
    .tab:hover {
      color: #e2e8f0;
    }
    .tab.active {
      color: #60a5fa;
      border-bottom-color: #60a5fa;
    }
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  /* ── Sidebar toolbar ──────────────────────────────────────── */
  .sidebar-toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .chapter-list-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #94a3b8;
    margin-bottom: 0.25rem;
  }

  .chapter-list {
    list-style: none;
    padding: 0;
    margin: 0 0 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .chapter-link {
    width: 100%;
    text-align: left;
    padding: 0.375rem 0.5rem;
    border: none;
    background: transparent;
    border-radius: 0.25rem;
    font-size: 0.8125rem;
    color: #475569;
    cursor: pointer;
    transition: background 120ms;
  }

  .chapter-link:hover {
    background: #f1f5f9;
  }

  .chapter-link.active-chapter {
    background: #eff6ff;
    color: #3b82f6;
    font-weight: 500;
  }

  @media (prefers-color-scheme: dark) {
    .chapter-link {
      color: #94a3b8;
    }
    .chapter-link:hover {
      background: #1e293b;
    }
    .chapter-link.active-chapter {
      background: #1e3a5f;
      color: #60a5fa;
    }
  }

  .btn-load {
    padding: 0.375rem 0.75rem;
    border: 1px solid #e2e8f0;
    border-radius: 0.375rem;
    background: #ffffff;
    font-size: 0.8125rem;
    color: #475569;
    cursor: pointer;
    transition: border-color 120ms, background 120ms;
  }

  .btn-load:hover {
    border-color: #3b82f6;
    background: #f8fafc;
  }

  @media (prefers-color-scheme: dark) {
    .btn-load {
      background: #1e293b;
      border-color: #334155;
      color: #94a3b8;
    }
    .btn-load:hover {
      border-color: #60a5fa;
    }
  }

  /* ── Editor area ───────────────────────────────────────────── */
  .editor {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  /* ── Setup prompt (no project yet) ─────────────────────────── */
  .setup-prompt {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    height: 100%;
  }

  .setup-text {
    color: #94a3b8;
    font-size: 1.125rem;
    font-style: italic;
  }

  @media (prefers-color-scheme: dark) {
    .setup-text {
      color: #64748b;
    }
  }

  .btn-primary {
    padding: 0.5rem 1.25rem;
    border: none;
    border-radius: 0.375rem;
    background: #3b82f6;
    color: #ffffff;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 150ms;
  }

  .btn-primary:hover {
    background: #2563eb;
  }

  /* ── Editor pane ───────────────────────────────────────────── */
  .editor-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid #e2e8f0;
    background: #f8fafc;
    flex-shrink: 0;
    gap: 0.75rem;
  }

  @media (prefers-color-scheme: dark) {
    .editor-toolbar {
      background: #0f172a;
      border-bottom-color: #334155;
    }
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .project-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: #1e293b;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (prefers-color-scheme: dark) {
    .project-label {
      color: #e2e8f0;
    }
  }

  .chapter-label {
    font-size: 0.75rem;
    color: #64748b;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (prefers-color-scheme: dark) {
    .chapter-label {
      color: #94a3b8;
    }
  }

  .toolbar-btn {
    padding: 0.25rem 0.625rem;
    border: 1px solid #e2e8f0;
    border-radius: 0.25rem;
    background: #ffffff;
    font-size: 0.75rem;
    color: #3b82f6;
    cursor: pointer;
    transition: background 120ms, border-color 120ms;
    white-space: nowrap;
  }

  .toolbar-btn:hover {
    background: #eff6ff;
    border-color: #3b82f6;
  }

  @media (prefers-color-scheme: dark) {
    .toolbar-btn {
      background: #1e293b;
      border-color: #334155;
      color: #60a5fa;
    }
    .toolbar-btn:hover {
      background: #1e3a5f;
    }
  }

  /* ── Save indicator ────────────────────────────────────────── */
  .save-indicator {
    font-size: 0.75rem;
    color: #94a3b8;
    transition: color 200ms;
  }

  .save-indicator.saving {
    color: #f59e0b;
  }

  .save-indicator.saved {
    color: #22c55e;
  }

  .save-indicator.unsaved {
    color: #ef4444;
  }

  /* ── Editor body ───────────────────────────────────────────── */
  .editor-body {
    flex: 1;
    overflow: hidden;
    min-width: 0;
  }
</style>

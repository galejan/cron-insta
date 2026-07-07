# Cron-Insta — Atajos de teclado (estado actual)

> Generado el 2026-07-07. Refleja los cambios aplicados hoy.
> La edición de atajos está temporalmente deshabilitada en la UI.

## Leyenda de plataformas

| Símbolo | Significado |
|---------|-------------|
| ✅ | Confirmado funcional |
| ⚠️ | No verificado (riesgo teórico) |
| ❌ | Confirmado roto / interceptado |
| 🔧 | Cambiado hoy (aún no verificado en todas las plataformas) |

---

## 1. Panel lateral — `Ctrl` + flechas

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 1 | `Ctrl+←` | `sidebar-shrink` | ✅ | ⚠️ | ⚠️ |
| 2 | `Ctrl+→` | `sidebar-grow` | ✅ | ⚠️ | ⚠️ |
| 3 | `Ctrl+Shift+←` | `sidebar-collapse` | ✅ | ⚠️ | ⚠️ |
| 4 | `Ctrl+Shift+→` | `sidebar-expand` | ✅ | ⚠️ | ⚠️ |

## 2. Navegación — `Alt`

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 5 | `Alt+←` | `prev-chapter` | ✅ | ⚠️ | ⚠️ |
| 6 | `Alt+→` | `next-chapter` | ✅ | ⚠️ | ⚠️ |
| 7 | `Alt+T` | `cycle-tabs` | 🔧 ❌ | 🔧 ⚠️ | 🔧 ⚠️ |

> **Alt+T**: Cambiado hoy desde `Ctrl+T`. En Linux no funciona (WebKitGTK intercepta Alt+letra). Pendiente decidir alternativa.

## 3. Edición — `Ctrl` (+ `Ctrl+Shift+D`)

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 8 | `Ctrl+S` | `save` | ✅ | ⚠️ | ⚠️ |
| 9 | `Ctrl+B` | `bold` | ✅ | ⚠️ | ⚠️ |
| 10 | `Ctrl+I` | `italic` | ✅ | ⚠️ | ⚠️ |
| 11 | `Ctrl+↑` | `heading-up` | ✅ | ⚠️ | ⚠️ |
| 12 | `Ctrl+↓` | `heading-down` | ✅ | ⚠️ | ⚠️ |
| 13 | `Ctrl++` | `zoom-in` | ✅ | ⚠️ | ⚠️ |
| 14 | `Ctrl+-` | `zoom-out` | ✅ | ⚠️ | ⚠️ |
| 15 | `Ctrl+Shift+D` | `dialogue-dash` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |

> **dialogue-dash**: Cambiado hoy desde `Ctrl+D` (bookmark browser).

## 4. Proyecto — `Ctrl+Alt`

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 16 | `Ctrl+Alt+N` | `new-chapter` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 17 | `Ctrl+Alt+O` | `open-project` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 18 | `Ctrl+Alt+W` | `close-project` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 19 | `Ctrl+Alt+E` | `export-zip` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 20 | `Ctrl+Alt+R` | `repair-project` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 21 | `Ctrl+Alt+,` | `global-settings` | ✅ | ⚠️ | ⚠️ |
| 22 | `Ctrl+Alt+Shift+N` | `new-project` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| — | `Ctrl+I` | `import-project` | ✅ | ⚠️ | ⚠️ |
| — | `Ctrl+Shift+E` | `export-md` | ✅ | ⚠️ | ⚠️ |
| — | `Ctrl+,` | `project-settings` | ✅ | ⚠️ | ⚠️ |

> Los últimos 3 (`import-project`, `export-md`, `project-settings`) no usan `Ctrl+Alt` pero están en el grupo Proyecto por función. Son atajos heredados que funcionan.

## 5. Entidades — `Ctrl+Shift`

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 23 | `Ctrl+Shift+C` | `new-character` | ✅ | ⚠️ | ⚠️ |
| 24 | `Ctrl+Shift+L` | `new-place` | ✅ | ⚠️ | ⚠️ |
| 25 | `Ctrl+Shift+M` | `new-note` | ✅ | ⚠️ | ⚠️ |
| 26 | `Ctrl+Shift+E` | `new-event` | ✅ | ⚠️ | ⚠️ |
| 27 | `Ctrl+Shift+G` | `new-trama` | ✅ | ⚠️ | ⚠️ |

## 6. Interfaz — misceláneo

| # | Atajo | ID | Linux | Windows | macOS |
|---|-------|----|-------|---------|-------|
| 28 | `Ctrl+Enter` | `dock` | ✅ | ⚠️ | ⚠️ |
| 29 | `Ctrl+Alt+P` | `toggle-footer` | 🔧 ⚠️ | 🔧 ⚠️ | 🔧 ⚠️ |
| 30 | `F1` | `toggle-help` | ✅ | ⚠️ | ⚠️ |
| 31 | `Shift+?` | `help-question` | ✅ | ⚠️ | ⚠️ |
| 32 | `F11` | `toggle-fullscreen` | ✅ | ⚠️ | ⚠️ |

---

## Lo que falta verificar

- **Linux**: `Alt+T` no funciona (WebKitGTK intercepta Alt+letra). Los 5 nuevos `Ctrl+Alt+letra` (#16–#20, #29) hay que probarlos.
- **Windows**: Todos los nuevos (#16–#20, #22, #29) necesitan verificación en WebView2.
- **macOS**: `Ctrl` ≠ `Cmd`. En macOS el modificador estándar es `Cmd`. Los usuarios de Mac esperan `Cmd+S`, `Cmd+B`, etc. Esto no se toca en este cambio pero es un gap conocido.

---

## Grupos por modificador (resumen visual)

```
Panel lateral   Ctrl + ← →           |   Ctrl+Shift + ← →
Navegación      Alt + ← → T          ← Alt+T roto en Linux
Edición         Ctrl + S B I ↑↓ + -  |   Ctrl+Shift + D
Proyecto        Ctrl+Alt + N O W E , R  |  Ctrl+Alt+Shift + N
Entidades       Ctrl+Shift + C L M E G
Interfaz        Ctrl+Enter  |  Ctrl+Alt+P  |  F1 F11 Shift+?
```

NUEVO ANALISIS PROPUESTO.
Propuesta de Atajos de Teclado Seguros para TauriEste documento analiza críticamente el estado actual de los atajos de teclado de Cron-Insta (según el reporte del 2026-07-07) y propone una reestructuración blindada contra colisiones de sistema operativo (OS) y del motor web (WebKitGTK/WebView2).1. Informe de Daños: ¿Por qué va a fallar el esquema actual?El esquema actual presenta tres vulnerabilidades críticas que romperán la experiencia de usuario según la plataforma:A. El colapso de Ctrl + Flechas en macOSProblema: En macOS, Ctrl + ←/→ y Ctrl + Shift + ←/→ están reservados por el sistema operativo para cambiar de espacio de trabajo (Mission Control / Spaces).Consecuencia: Tu panel lateral no se podrá redimensionar en Mac; el sistema operativo interceptará el evento antes de que llegue a Tauri.B. El infierno de Alt en Linux (WebKitGTK) y Windows (WebView2)Problema: Alt en solitario (como en Alt+T o Alt+←) está reservado para mnemónicos de menús del sistema. WebKitGTK intercepta de forma agresiva cualquier combinación de Alt + Letra. Además, Alt + Flecha Izquierda/Derecha es el comando universal de navegación de historial ("Atrás" / "Adelante") en los motores de renderizado web.Consecuencia: Comportamiento errático, pérdida de foco de la aplicación y navegación involuntaria de la página web interna.C. Conflicto de Herramientas de Desarrollo (DevTools)Problema: Ctrl+Shift+C (que propones para new-character) es el atajo universal para abrir el inspector de elementos en Chrome, Edge y WebKit.Consecuencia: Si dejas activadas las DevTools en producción (o en entornos de test), el usuario abrirá la consola en lugar de crear un personaje.2. La Regla de Oro en Tauri: Modificadores DinámicosPara solucionar esto sin duplicar la lógica, el frontend debe abstraer el modificador principal bajo el concepto de Mod (o Primary):Windows / Linux: Mod = CtrlmacOS: Mod = Cmd (Command $\mathcal{⌘}$)3. Matriz de Atajos Seguros RecomendadaEsta tabla sustituye las combinaciones conflictivas por alternativas blindadas que respetan las guías de interfaz humana (HIG) de Apple, Microsoft y GNOME.ID del AtajoAcciónWindows / LinuxmacOSJustificación Técnica de Seguridadsidebar-shrinkContraer panelCtrl + Alt + ←Cmd + Alt + ←Evita el conflicto con Mission Control en macOS.sidebar-growExpandir panelCtrl + Alt + →Cmd + Alt + →Evita el conflicto con Mission Control en macOS.prev-chapterCapítulos atrásAlt + PageUpCmd + Alt + ←Evita la navegación de historial (Alt+←) de los WebViews.next-chapterCapítulos adelanteAlt + PageDownCmd + Alt + →Evita la navegación de historial (Alt+→) de los WebViews.cycle-tabsRotar pestañasCtrl + TabCtrl + TabComportamiento estándar de sistemas operativos y navegadores.dialogue-dashGuión de diálogoCtrl + Shift + DCmd + Shift + DElude Ctrl+D (marcador del navegador).import-projectImportar proyectoCtrl + Alt + ICmd + Alt + ILibera Ctrl+I para cursiva (italic) sin colisiones.Opción A: Mapeo Numérico (La más segura)Sustituir las letras por números elimina cualquier colisión con el navegador.new-character $\rightarrow$ Mod + Shift + 1new-place     $\rightarrow$ Mod + Shift + 2new-note      $\rightarrow$ Mod + Shift + 3new-event     $\rightarrow$ Mod + Shift + 4new-trama     $\rightarrow$ Mod + Shift + 54. Guía de Implementación en TauriPara asegurar que el backend web de Tauri no intercepte y anule tus comandos locales, debes capturar los eventos de teclado de manera estricta en tu Frontend JavaScript/TypeScript.Implementación robusta en el Frontend// Detectar modificador dinámicamente
const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;

window.addEventListener('keydown', (e) => {
  const modKey = isMac ? e.metaKey : e.ctrlKey;
  const altKey = e.altKey;
  const shiftKey = e.shiftKey;

  // Ejemplo: Guardar Proyecto (Mod + S)
  if (modKey && e.key.toLowerCase() === 's' && !shiftKey && !altKey) {
    e.preventDefault(); // Detiene la acción por defecto del navegador
    e.stopPropagation();
    ejecutarAccion('save');
  }

  // Ejemplo: Navegación de pestañas segura (Ctrl + Tab)
  if (e.ctrlKey && e.key === 'Tab') {
    e.preventDefault();
    e.stopPropagation();
    ejecutarAccion('cycle-tabs');
  }
  
  // Ejemplo: Redimensionar Panel (Mod + Alt + Flechas)
  if (modKey && altKey && e.key === 'ArrowLeft') {
    e.preventDefault();
    e.stopPropagation();
    ejecutarAccion('sidebar-shrink');
  }
}, { capture: true }); // El flag capture asegura prioridad máxima
Registro de Atajos Globales en Rust (Opcional)Si necesitas que ciertos comandos (como abrir la ventana principal o pausar) funcionen incluso cuando la aplicación no está enfocada, regístralos en tu main.rs usando modificadores virtuales que Tauri traduce automáticamente para cada plataforma:use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

fn setup_shortcuts(app: &AppHandle) {
    let shortcuts = app.global_shortcut();
    
    // Tauri parsea "CommandOrControl" automáticamente:
    // - En macOS usará Cmd + Alt + P
    // - En Windows/Linux usará Ctrl + Alt + P
    let toggle_footer = Shortcut::new(
        Some(tauri_plugin_global_shortcut::Modifiers::COMMAND_OR_CONTROL | tauri_plugin_global_shortcut::Modifiers::ALT),
        tauri_plugin_global_shortcut::Code::KeyP
    );

    let _ = shortcuts.register(toggle_footer);
}

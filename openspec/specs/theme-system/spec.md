# theme-system Specification

## Purpose

A 4-theme CSS variable infrastructure applied via `data-theme` attribute on `<html>`, managed by a ThemeManager class following the existing i18n `$state` pattern, with backward compatibility for existing `.dark` class selectors.

## Requirements

| # | Requirement | Strength | Summary |
|---|------------|----------|---------|
| R1 | CSS Custom Property Themes | MUST | 4 `html[data-theme="..."]` blocks defining `--bg-app`, `--bg-editor`, `--bg-sidebar`, `--bg-active-tab`, `--text-main`, `--text-title`, `--text-muted`, `--border-color`, `--accent`, `--scrollbar-thumb` |
| R2 | Default Theme | SHALL | `dark-nordic` applies when no `data-theme` attribute is set (via bare `html` selector) |
| R3 | Theme Transition | MUST | `html { transition: background 0.2s ease, color 0.2s ease }` |
| R4 | ProseMirror Variable Usage | MUST | `.ProseMirror` uses `var(--bg-editor)` and `var(--text-main)`; headings use `var(--text-title)`; hardcoded `#1a1a1a` and `.dark .ProseMirror` selectors SHALL be removed |
| R5 | Dark Class Backward Compatibility | MUST | `setTheme()` adds class `"dark"` to `<html>` for dark themes (`dark-nordic`, `dark-amethyst`), removes for light themes; existing `:global(.dark)` component selectors (~60+) keep working |
| R6 | ThemeManager State Class | MUST | `ThemeManager` class in `theme.svelte.ts` with `$state` field `current` (type `AppTheme` union); constructor reads `cron-insta-theme`; `setTheme()` persists to localStorage and sets `data-theme` |
| R7 | Tauri Title Bar Mapping | MUST | `getCurrentWindow().setTheme("dark")` for dark-nordic/amethyst; `"light"` for light-nordic/sepia |

### Scenario: All 4 themes render correctly

- GIVEN `<html>` has `data-theme="dark-amethyst"`
- WHEN the app renders
- THEN backgrounds show purple-tinted dark tones
- AND text renders in `#e4def2`
- AND accent color is `#a855f7`

### Scenario: Theme applies instantly on setTheme()

- GIVEN `current` is `"dark-nordic"`
- WHEN `setTheme("light-nordic")` is called
- THEN `data-theme` attribute on `<html>` changes to `"light-nordic"`
- AND `.dark` class is removed from `<html>`
- AND localStorage key `cron-insta-theme` is set to `"light-nordic"`
- AND colors transition over 0.2s

### Scenario: ProseMirror renders readable text in light themes

- GIVEN theme is `"light-nordic"`
- WHEN a document renders in the editor
- THEN body text is dark (`var(--text-main)`) on light background (`var(--bg-editor)`)
- AND headings use `var(--text-title)` for distinct contrast
- AND no hardcoded `#1a1a1a` overrides apply

### Scenario: Existing .dark selectors continue working

- GIVEN theme is `"dark-amethyst"`
- WHEN `ProjectSettingsDialog` renders
- THEN its `:global(.dark)` scoped styles resolve via the `.dark` class on `<html>`
- AND the dialog renders with dark styling

### Scenario: ThemeManager syncs state and DOM

- GIVEN localStorage has `cron-insta-theme` = `"light-sepia"`
- WHEN the app mounts and ThemeManager constructs
- THEN `current` is `"light-sepia"`
- AND `document.documentElement` has `data-theme="light-sepia"`
- AND `.dark` class is absent from `<html>`

### Scenario: Tauri title bar matches theme family

- GIVEN theme changed to `"dark-amethyst"`
- WHEN `setTheme()` executes
- THEN `getCurrentWindow().setTheme("dark")` is called (not the full 4-theme value)

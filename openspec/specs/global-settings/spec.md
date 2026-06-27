# global-settings Specification

## Purpose

Global settings dialog for language and theme preferences, accessible from any app state, with first-run auto-open and live theme preview.

## Requirements

| # | Requirement | Strength | Summary |
|---|------------|----------|---------|
| R1 | Dialog Access | MUST | Gear icon in toolbar (between chapter label and `?` help), always visible |
| R2 | First-Run Auto-Open | SHALL | Open dialog when `cron-insta-has-launched` is absent; set key to `"true"` on dismiss |
| R3 | Language Selection | MUST | ES/EN selector reflecting current `lang` from i18n; calls `setLang()` on change; applies immediately |
| R4 | Theme Selection with Live Preview | MUST | 4 radio cards (dark-nordic, dark-amethyst, light-nordic, light-sepia); each shows sample title + body in theme colors via CSS vars |
| R5 | Live Theme Application | MUST | Selection sets `data-theme` on `<html>` instantly; no save step required |
| R6 | Theme Migration | MUST | Map existing `"dark"` → `"dark-nordic"`, `"light"` → `"light-nordic"`; keep 4-theme values as-is |
| R7 | Persistence | MUST | Language → `cron-insta-lang`; Theme → `cron-insta-theme` (full 4-theme value); survive restart |
| R8 | Modal Theming | MUST | Dialog inherits active theme via CSS variables (`--bg-app`, `--text-main`, `--border-color`, `--accent`) |

### Scenario: First launch auto-opens dialog

- GIVEN no `cron-insta-has-launched` key exists
- WHEN the app mounts (with or without a project)
- THEN the global settings dialog opens automatically

### Scenario: Subsequent launches skip auto-open

- GIVEN `cron-insta-has-launched` is `"true"`
- WHEN the app mounts
- THEN the dialog does NOT open

### Scenario: Gear icon opens dialog in any state

- GIVEN the app is running (project or no-project)
- WHEN the user clicks the Gear icon in the editor toolbar
- THEN the global settings dialog opens as a modal

### Scenario: Language change applies immediately

- GIVEN current language is "es" and the dialog is open
- WHEN the user selects "EN"
- THEN all UI text changes to English immediately
- AND localStorage key `cron-insta-lang` is updated

### Scenario: Theme preview reflects selection

- GIVEN the dialog is open on the Theme tab
- WHEN the user selects "light-sepia"
- THEN the preview card shows warm sepia tones
- AND `<html>` `data-theme` attribute updates to `"light-sepia"` immediately
- AND the dialog itself transitions to sepia styling

### Scenario: Legacy binary theme migrates on load

- GIVEN `cron-insta-theme` localStorage is `"dark"`
- WHEN ThemeManager initializes
- THEN theme becomes `"dark-nordic"` and is persisted back

### Scenario: Preferences survive app restart

- GIVEN the user selected "en" and "dark-amethyst"
- WHEN the app restarts
- THEN the UI renders in English with the amethyst theme applied

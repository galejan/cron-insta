# Delta for user-interface

## ADDED Requirements

### Requirement: Toolbar Gear Icon

The `.editor-toolbar` section MUST include a `Gear` (Phosphor) icon button positioned between the chapter label and the `?` help button. The icon SHALL follow the existing pattern: `weight="light" size={16} color="currentColor"` with a `title` attribute. The icon MUST render in all app states (with or without a project loaded).

#### Scenario: Gear icon opens global settings

- GIVEN the app is running
- WHEN the user clicks the Gear icon in the toolbar
- THEN `GlobalSettingsDialog` opens as a modal

#### Scenario: Gear icon has accessible title

- GIVEN a screen reader focuses the Gear button
- WHEN the button receives focus
- THEN the `title` attribute is announced

### Requirement: Sidebar Footer Language Toggle Removal

The language toggle currently in the sidebar footer MUST be removed. Language selection SHALL move to the Global Settings dialog, accessible via the toolbar Gear icon. The sidebar footer SHALL retain only the per-project settings button and the export button.

#### Scenario: Language toggle absent from sidebar footer

- GIVEN the sidebar footer renders
- WHEN inspecting the footer area
- THEN no language toggle element exists
- AND language changes are made via Global Settings only

### Requirement: First-Run Global Settings Auto-Open

The main page mount logic MUST check for the absence of `cron-insta-has-launched` in localStorage. If absent, the global settings dialog SHALL open automatically. The check MUST execute on mount regardless of project load state. After dialog dismiss, the key SHALL be set to `"true"`.

#### Scenario: Auto-open without any project

- GIVEN no project is loaded and `cron-insta-has-launched` is absent
- WHEN the app mounts
- THEN the global settings dialog opens
- AND `cron-insta-has-launched` is set to `"true"` after dismiss

#### Scenario: Auto-open skipped after first run

- GIVEN `cron-insta-has-launched` is `"true"`
- WHEN the app mounts
- THEN the global settings dialog does NOT auto-open

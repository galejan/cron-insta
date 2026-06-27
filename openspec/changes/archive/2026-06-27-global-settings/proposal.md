# Proposal: Global Settings

## Intent

Users cannot change app language without finding the sidebar footer toggle, and the theme system is a binary light/dark switch. There is no first-run guidance. A single, always-accessible global settings dialog with live theme previews and first-run auto-open gives users immediate control over appearance.

## Scope

### In Scope
- `GlobalSettingsDialog.svelte` — external modal component with Language + Theme tabs
- Gear icon (Phosphor `Gear`) in editor toolbar, next to the help `?` button, always visible
- ThemeManager (`theme.svelte.ts`) — Svelte 5 `$state` + localStorage, matching the i18n pattern
- 4-theme CSS variable system: `dark-nordic` (default), `dark-amethyst`, `light-nordic`, `light-sepia`
- Live theme preview cards with sample title + body text in each theme's colors
- First-run auto-open via `cron-insta-has-launched` localStorage flag
- Migration of existing `cron-insta-theme`: `"dark"` → `"dark-nordic"`, `"light"` → `"light-nordic"`
- Removal of sidebar footer light/dark toggle
- Hot application: theme applies instantly on selection via `data-theme` attribute

### Out of Scope
- Custom theme creator
- Font size/family in global settings (stays per-project)
- Tauri native title bar beyond dark/light mapping

## Capabilities

### New Capabilities
- `global-settings`: Always-accessible dialog for language and theme preferences, persisted to localStorage, with first-run auto-open behavior and live preview.
- `theme-system`: 4-theme CSS variable system controlled by `data-theme` attribute on `<html>`, managed via `ThemeManager` class with Svelte 5 `$state` runes.

### Modified Capabilities
- `user-interface`: Toolbar gains a Gear icon next to the help button. Sidebar footer light/dark toggle is removed. First-run detection triggers auto-open.

## Approach

1. **CSS**: Add 4 theme blocks from `docs/mejora temas y alineacion.md` to `app.css`; replace hardcoded `.dark` colors with CSS variables
2. **State**: Create `src/lib/theme.svelte.ts` with `ThemeManager` class (`$state`, localStorage init, `setTheme()`)
3. **Component**: `GlobalSettingsDialog.svelte` — follows `ProjectSettingsDialog` pattern (overlay + panel, Escape close, `bind:open`), two-tab layout with theme preview cards and language toggle
4. **Integration**: Gear button in toolbar, first-run `$effect`, migrate old localStorage value on first load
5. **Transition**: Keep `.dark` class alongside `data-theme` for existing component selectors (~30+ occurrences in `ProjectSettingsDialog`, `GitIdentityDialog`)

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/app.css` | Modified | Replace hardcoded `.dark` colors with CSS variables + 4 theme blocks |
| `src/routes/+page.svelte` | Modified | Remove sidebar light/dark toggle, add Gear icon in toolbar, first-run detection, wire dialog |
| `src/lib/theme.svelte.ts` | New | ThemeManager class with `$state`, localStorage, `data-theme` |
| `src/lib/components/GlobalSettingsDialog.svelte` | New | Global settings dialog (language + theme) |
| `src/lib/i18n.svelte.ts` | None | Reused as-is — no changes needed |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `.dark` selectors break in existing components | Medium | Keep `.dark` class alongside `data-theme` during transition; remove later |
| Tauri `setTheme()` rejects 4-theme values | Low | Map dark themes → `"dark"`, light themes → `"light"` for native title bar |
| `+page.svelte` line count grows | Medium | External dialog component; sidebar toggle removal nets negative lines |

## Rollback Plan

Revert `app.css` to hardcoded `.dark`/light (backed up in git), remove `data-theme` attribute logic, restore sidebar toggle, delete `theme.svelte.ts` and `GlobalSettingsDialog.svelte`.

## Dependencies

- Phosphor Icons `Gear` component (already installed in `phosphor-svelte`)

## Success Criteria

- [ ] Gear icon appears in toolbar (all projects and no-project state)
- [ ] First launch auto-opens global settings; subsequent launches do not
- [ ] All 4 themes apply instantly on selection; dialog reflects active theme
- [ ] Theme and language persist across app restarts
- [ ] Old `"light"`/`"dark"` localStorage values migrate to 4-theme values correctly
- [ ] Per-project settings dialog (Font/Identity/Remote) unaffected

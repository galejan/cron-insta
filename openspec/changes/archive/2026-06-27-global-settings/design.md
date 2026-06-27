# Design: Global Settings

## Technical Approach

Create a standalone `GlobalSettingsDialog.svelte` component mirroring the `ProjectSettingsDialog` pattern (overlay + panel, Escape close, `bind:open`). Introduce `ThemeManager` in `theme.svelte.ts` — a `$state` class following the exact `i18n.svelte.ts` pattern. Replace hardcoded `.dark` CSS with a 4-theme `data-theme` CSS variable system from `docs/mejora temas y alineacion.md`, preserving the `.dark` class for existing `:global(.dark)` selectors (~100+ in `+page.svelte` alone). Add a Gear icon to the toolbar and first-run detection via `cron-insta-has-launched` localStorage key.

## Architecture Decisions

| # | Decision | Option | Tradeoff | Chosen |
|---|----------|--------|----------|--------|
| AD1 | Dialog as external component vs inline | External `GlobalSettingsDialog.svelte` | +200 LOC in new file vs +200 LOC in 5257-line `+page.svelte` | External — keeps page smaller, reusable, testable |
| AD2 | `.dark` class: remove or keep | Dual approach: `data-theme` for CSS vars + `.dark` class for component selectors | Slightly redundant but avoids touching ~100+ `:global(.dark)` selectors across 4+ components | Keep `.dark` — zero-risk migration, remove later |
| AD3 | ThemeManager pattern | Svelte 5 `$state` class (mirror `i18n.svelte.ts`) | Identical reactivity pattern across state modules | `$state` class — consistency with existing conventions |
| AD4 | Theme preview cards | `<button>` wrappers with inline `style` using CSS vars | Each card self-applies its theme via inline custom properties; no DOM mutation needed for preview | Inline vars — no CSS class spray or `data-theme` conflict |
| AD5 | Grid adjustment for toolbar | Change to `grid-template-columns: 1fr auto auto 1fr` | Adds automatic column between chapter label and help button | 4-column grid — minimal change, natural fit |

## Data Flow

```
App mount
  ├─ ThemeManager constructor → reads cron-insta-theme → migrates "dark"/"light" → sets data-theme + .dark
  └─ First-run $effect → checks cron-insta-has-launched → if absent: opens dialog

Gear click → globalSettingsOpen = true → GlobalSettingsDialog renders

Theme tab click → themeManager.setTheme(theme)
  ├─ $state current updates → UI reacts
  ├─ document.documentElement.setAttribute('data-theme', theme)
  ├─ document.documentElement.classList.toggle('dark', theme.startsWith('dark'))
  ├─ localStorage.setItem('cron-insta-theme', theme)
  └─ try { getCurrentWindow().setTheme(theme.startsWith('dark') ? 'dark' : 'light') }

Language change → setLang() from i18n (no new logic)
```

## Sequence: First-Run Auto-Open

```
GIVEN cron-insta-has-launched is absent
WHEN app mounts
  +page.svelte $effect → globalSettingsOpen = true
  GlobalSettingsDialog renders overlay
  User clicks close/Escape → globalSettingsOpen = false
  $effect: onDestroy/dismiss → localStorage.setItem('cron-insta-has-launched', 'true')
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/lib/theme.svelte.ts` | **NEW** | ThemeManager class: `AppTheme` type union, `$state current`, localStorage init+migration, `setTheme()` → `data-theme` + `.dark` + Tauri |
| `src/lib/components/GlobalSettingsDialog.svelte` | **NEW** | Modal: overlay + panel, Language tab (ES/EN radio) + Theme tab (4 preview cards), `bind:open`, Escape close, z-index:200 |
| `src/app.css` | **MODIFY** | Replace `html.dark` + `.dark .ProseMirror` with `html { color: var(--text-main); background: var(--bg-app); transition: ... }` and `:root` 4-theme blocks. `.ProseMirror` uses `var(--bg-editor)` / `var(--text-main)`. Headings use `var(--text-title)` |
| `src/routes/+page.svelte` | **MODIFY** | (1) Remove `theme` binary state and 3 duplicate `$effect` blocks (lines 115-172) (2) Add `globalSettingsOpen` state (3) Gear icon button between chapter label and help button (4) `$effect` for first-run detection (5) Remove footer ES/EN language toggle (lines 2596-2608) (6) Remove footer Moon/Sun theme toggle (lines 2610-2614) (7) Import GlobalSettingsDialog, wire `bind:open` |
| `src/lib/i18n.svelte.ts` | **UNCHANGED** | No modifications — reuse existing `lang`, `setLang()`, `t()` |

## Interfaces / Contracts

```ts
// src/lib/theme.svelte.ts
export type AppTheme = 'dark-nordic' | 'dark-amethyst' | 'light-nordic' | 'light-sepia';

export class ThemeManager {
  current: AppTheme;        // $state — reactive
  constructor();             // reads localStorage, migrates, applies
  setTheme(theme: AppTheme): void;  // persists + applies
}

export const themeManager: ThemeManager;
```

```svelte
<!-- GlobalSettingsDialog props -->
let { open = $bindable<boolean>(false) } = $props();
```

## CSS Variable Migration

Replaces `app.css` lines 7-10 (`html.dark { background: #0f172a; color: #e2e8f0; }`) and lines 75-85 (`.dark .ProseMirror` + headings) with:

1. 4 `html[data-theme="..."]` blocks defining 10 custom properties each (per theme doc lines 38-92)
2. `html { background: var(--bg-app); color: var(--text-main); transition: background 0.2s ease, color 0.2s ease }`
3. `.ProseMirror { color: var(--text-main); background: var(--bg-editor); }` — removes hardcoded `#1a1a1a`
4. `.ProseMirror h1, h2, h3 { color: var(--text-title); }` — removes `.dark .ProseMirror h1` etc.

**Critical**: The `.dark` class on `<html>` is preserved via `setTheme()` for dark themes so existing `:global(.dark)` component selectors (~100+ in `+page.svelte`, ProjectSettingsDialog, ProjectConfigForm, GitIdentityDialog) continue resolving. The `data-theme` attribute is the primary theming mechanism; `.dark` class is a compatibility shim.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (Rust) | N/A — this change is frontend-only | No Rust tests affected |
| Manual | ThemeManager localStorage migration | Verify old `"dark"`/`"light"` map correctly; verify `data-theme` + `.dark` sync |
| Manual | 4 themes render correctly | Visual check: all 10 CSS vars apply; ProseMirror text readable in light themes |
| Manual | First-run auto-open | Clear localStorage, reload, verify dialog opens; reload again, verify it doesn't |
| Manual | Dialog keyboard/Escape | Press Escape, click overlay → dialog closes |
| Manual | Language change | Switch in dialog → UI reacts; restart app → persists |

## Migration / Rollout

- Existing `cron-insta-theme` values `"light"`/`"dark"` auto-migrated by ThemeManager constructor.
- `.dark` class stays on `<html>` for dark themes — no component breakage.
- Footer language/theme toggles removed; users access settings via toolbar Gear.
- Rollback: revert `app.css`, restore footer toggles, delete 2 new files.

## Open Questions

None — all patterns are proven (i18n state, ProjectSettingsDialog modal, Phosphor icons).

# Tasks: Global Settings

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~410 (add: ~360, del: ~50) |
| 400-line budget risk | Medium |
| Chained PRs recommended | No — fits user's 800-line review budget |
| Suggested split | Single PR with work-unit commits |
| Delivery strategy | auto-forecast |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Medium

## Phase 1: CSS Themes Foundation

- [x] 1.1 Replace `html.dark` block (lines 7-10) in `app.css` with `:root` + `html` base rules (`background: var(--bg-app); color: var(--text-main); transition: background 0.2s ease, color 0.2s ease`)
- [x] 1.2 Add 4 `html[data-theme="..."]` blocks (`dark-nordic`, `dark-amethyst`, `light-nordic`, `light-sepia`) from `docs/mejora temas y alineacion.md` lines 38-92 — 10 custom properties each (`--bg-app`, `--bg-editor`, `--bg-sidebar`, `--bg-active-tab`, `--text-main`, `--text-title`, `--text-muted`, `--border-color`, `--accent`, `--scrollbar-thumb`)
- [x] 1.3 Replace hardcoded `.ProseMirror` colors (line 18 `#1a1a1a`) and `.dark .ProseMirror` selectors (lines 75-85) with `var(--bg-editor)` / `var(--text-main)` / `var(--text-title)`; remove `.dark .ProseMirror h1/h2/h3` rules

## Phase 2: ThemeManager State

- [x] 2.1 Create `src/lib/theme.svelte.ts` — export `AppTheme` type union (`'dark-nordic' | 'dark-amethyst' | 'light-nordic' | 'light-sepia'`) and `ThemeManager` class with `$state current: AppTheme`
- [x] 2.2 Implement constructor: read `cron-insta-theme` localStorage, migrate `"dark"`→`"dark-nordic"` / `"light"`→`"light-nordic"`, persist back, apply `data-theme` attribute + `.dark` class on `<html>`
- [x] 2.3 Implement `setTheme()`: update `$state current`, `localStorage.setItem('cron-insta-theme')`, `document.documentElement.setAttribute('data-theme')`, `classList.toggle('dark')`, `getCurrentWindow().setTheme("dark"|"light")` mapping; export singleton `themeManager`

## Phase 3: GlobalSettingsDialog Component

- [x] 3.1 Create `src/lib/components/GlobalSettingsDialog.svelte` — overlay + panel modal following `ProjectSettingsDialog` pattern (`bind:open`, Escape close, `z-index: 200`, `role="dialog"`)
- [x] 3.2 Build Language tab: two `<button>` radio toggles (ES/EN) reading `lang.current` from i18n, calling `setLang()` on click
- [x] 3.3 Build Theme tab: 4 preview cards as `<button>` wrappers, each with inline `style` using the theme's CSS custom property values, showing sample title + body text in theme colors
- [x] 3.4 Wire theme card `onclick` → `themeManager.setTheme(theme)` for instant application; highlight active card via ring/accent border matching `themeManager.current`
- [x] 3.5 Add i18n keys for dialog labels (`globalSettings.title`, `globalSettings.language`, `globalSettings.theme`, `globalSettings.close`) — both ES and EN translations in `i18n.svelte.ts`

## Phase 4: Toolbar Integration + First-Run + Cleanup

- [x] 4.1 Add `globalSettingsOpen = $state(false)` in `+page.svelte`, import `GlobalSettingsDialog` and `Gear` (Phosphor), wire `<GlobalSettingsDialog bind:open={globalSettingsOpen} />` at page root
- [x] 4.2 Insert Gear icon button (`<button>` with Phosphor `Gear`, `weight="light" size={16} color="currentColor"`, `title` attr) in `.editor-toolbar` between chapter label span and `?` help button; adjust grid to `grid-template-columns: 1fr auto auto 1fr`
- [x] 4.3 Add `$effect` for first-run detection: check `localStorage.getItem('cron-insta-has-launched')`, if absent set `globalSettingsOpen = true` on mount
- [x] 4.4 Set `localStorage.setItem('cron-insta-has-launched', 'true')` on dialog dismiss (Escape/overlay-click/close button)
- [x] 4.5 Remove sidebar footer language toggle (lines 2596-2608) and theme toggle (lines 2610-2614) — language/theme now accessed via Gear dialog only
- [x] 4.6 Remove duplicate `$effect` theme blocks (lines 115-141) and old `theme` binary state (line 115) — replaced by ThemeManager

## Phase 5: Verification (Manual Smoke Tests)

- [x] 5.1 Clear localStorage, reload app → dialog auto-opens; reload again → dialog stays closed — *code ready, needs manual Tauri run*
- [x] 5.2 Cycle all 4 themes via cards → ProseMirror text readable, dialog inherits theme, `.dark` class toggles on `<html>` — *code ready, needs manual Tauri run*
- [x] 5.3 Switch language ES/EN → UI reacts immediately; restart app → language persists — *code ready, needs manual Tauri run*
- [x] 5.4 Gear icon visible in toolbar with and without project; Escape/overlay-click close dialog — *code ready, needs manual Tauri run*
- [x] 5.5 Set localStorage `cron-insta-theme` to `"dark"` → restarts as `"dark-nordic"`; per-project settings dialog unaffected — *code ready, needs manual Tauri run*
- [x] 5.6 Run `cargo test --manifest-path src-tauri/Cargo.toml` → no Rust test regressions — ✅ 127 passed, 0 failed

/**
 * Cron-Insta ThemeManager — reactive theme state for the 4-theme CSS variable system.
 *
 * Uses Svelte 5 $state runes for reactivity, mirroring the i18n.svelte.ts pattern.
 * Theme changes propagate immediately via data-theme attribute on <html>.
 *
 * Usage in components:
 *   import { themeManager, type AppTheme } from "$lib/theme.svelte";
 *   themeManager.setTheme("dark-amethyst");
 *   <span>{themeManager.current}</span>
 */

import { getCurrentWindow } from "@tauri-apps/api/window";

/** Supported theme identifiers. */
export type AppTheme = 'dark-nordic' | 'dark-amethyst' | 'light-nordic' | 'light-sepia';

class ThemeManager {
  /** Reactive current theme. Mutate via setTheme() to trigger UI updates. */
  current = $state<AppTheme>('dark-nordic');

  constructor() {
    if (typeof localStorage === 'undefined') return;

    const saved = localStorage.getItem('cron-insta-theme');

    // Migration: old binary "dark"/"light" → new 4-theme values
    let resolved: AppTheme;
    if (saved === 'dark') {
      resolved = 'dark-nordic';
    } else if (saved === 'light') {
      resolved = 'light-nordic';
    } else if (isValidTheme(saved)) {
      resolved = saved;
    } else {
      resolved = 'dark-nordic';
    }

    if (resolved !== saved) {
      // Persist the migrated value back
      localStorage.setItem('cron-insta-theme', resolved);
    }

    // Apply theme to DOM at init (runs synchronously)
    this.current = resolved;
    this.applyDom(resolved);
  }

  /** Change the active theme, persist, and apply to DOM immediately. */
  setTheme(theme: AppTheme): void {
    this.current = theme;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('cron-insta-theme', theme);
    }
    this.applyDom(theme);
  }

  // ── Private helpers ────────────────────────────────────────

  private applyDom(theme: AppTheme): void {
    const el = document.documentElement;
    el.setAttribute('data-theme', theme);

    // Maintain .dark class for backward compatibility with existing
    // :global(.dark) component selectors (~100+ across the app)
    const isDark = theme.startsWith('dark-');
    el.classList.toggle('dark', isDark);
    el.style.colorScheme = isDark ? 'dark' : 'light';

    // Sync Tauri native window theme (title bar on Windows/Linux)
    try {
      getCurrentWindow().setTheme(isDark ? 'dark' : 'light');
    } catch {
      /* not in Tauri environment */
    }
  }
}

/** Singleton theme manager instance. */
export const themeManager = new ThemeManager();

// ── Helpers ─────────────────────────────────────────────────

function isValidTheme(v: string | null): v is AppTheme {
  return v === 'dark-nordic' || v === 'dark-amethyst' || v === 'light-nordic' || v === 'light-sepia';
}

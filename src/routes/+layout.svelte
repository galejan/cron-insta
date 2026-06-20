<script lang="ts">
  import "../app.css";
  import { openUrl } from "@tauri-apps/plugin-opener";

  const { children } = $props();

  // Intercept external links and open them in the system browser
  // instead of the app webview. Applies to all <a href="http..."> tags,
  // including those rendered via {@html}.
  $effect(() => {
    function handleClick(e: MouseEvent) {
      const anchor = (e.target as HTMLElement).closest("a");
      if (!anchor) return;

      const href = anchor.getAttribute("href");
      if (href && (href.startsWith("http://") || href.startsWith("https://"))) {
        e.preventDefault();
        openUrl(href);
      }
    }

    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  });
</script>

{@render children()}

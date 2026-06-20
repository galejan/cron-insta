<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  const { children } = $props();

  onMount(() => {
    function handleClick(e: MouseEvent) {
      const anchor = (e.target as HTMLElement).closest("a");
      if (!anchor) return;

      const href = anchor.getAttribute("href");
      if (href && (href.startsWith("http://") || href.startsWith("https://"))) {
        e.preventDefault();
        e.stopPropagation();
        openUrl(href);
      }
    }

    // Capture phase — fires before Tauri's webview navigation handler
    document.addEventListener("click", handleClick, true);
    return () => document.removeEventListener("click", handleClick, true);
  });
</script>

{@render children()}

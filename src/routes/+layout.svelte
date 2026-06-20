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
        openUrl(href);
      }
    }

    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  });
</script>

{@render children()}

<script lang="ts">
  import ChartBar from "phosphor-svelte/lib/ChartBar";

  interface Props {
    projectStats: { total_sessions: number; total_hours: number; total_words: number };
    t: (key: string) => string;
    projectPath: string;
  }

  let { projectStats, t, projectPath }: Props = $props();
</script>

{#if projectPath}
  <div class="footer-row footer-stats">
    <ChartBar size={18} weight="light" color="currentColor" />
    <span class="stat-item" title={t("stats.sessions")}>{projectStats.total_sessions} {t("stats.sessionsLabel")}</span>
    <span class="stat-sep">·</span>
    <span class="stat-item" title={t("stats.hours")}>{projectStats.total_hours}h</span>
    <span class="stat-sep">·</span>
    <span class="stat-item" title={t("stats.words")}>{projectStats.total_words.toLocaleString()} {t("stats.wordsLabel")}</span>
  </div>
{/if}

<style>
  .footer-stats {
    gap: 0.375rem;
    font-size: 0.6875rem;
    color: #64748b;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid #e2e8f0;
  }

  :global(.dark) .footer-stats {
    color: #94a3b8;
    border-bottom-color: #334155;
  }

  .stat-item {
    white-space: nowrap;
  }

  .stat-sep {
    color: #cbd5e1;
  }

  :global(.dark) .stat-sep {
    color: #475569;
  }
</style>

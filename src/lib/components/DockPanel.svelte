<script lang="ts">
  import { fly } from "svelte/transition";
  import PushPin from "phosphor-svelte/lib/PushPin";
  import XCircle from "phosphor-svelte/lib/XCircle";

  interface Props {
    characterDocked: {
      id: string; name: string; physicalDescription: string;
      personality: string; traumas: string;
      relationships: Array<{targetName: string; type: string; notes: string}>;
    } | null;
    noteDocked: { id: string; title: string; content: string } | null;
    placeDocked: { id: string; name: string; description: string; notes: string } | null;
    mediaDocked: string | null;
    onCloseCharacter: () => void;
    onCloseNote: () => void;
    onClosePlace: () => void;
    onCloseMedia: () => void;
    onMediaClick: (filename: string) => void;
    mediaUrl: (filename: string) => string;
    t: (key: string) => string;
  }

  let {
    characterDocked,
    noteDocked,
    placeDocked,
    mediaDocked,
    onCloseCharacter,
    onCloseNote,
    onClosePlace,
    onCloseMedia,
    onMediaClick,
    mediaUrl,
    t,
  }: Props = $props();
</script>

{#if characterDocked}
  <div class="character-dock" transition:fly={{ x: 300, duration: 200 }}>
    <div class="character-dock-header">
      <h3><PushPin size={16} weight="light" color="currentColor" aria-hidden="true" /> {characterDocked.name}</h3>
      <button class="character-dock-close" onclick={onCloseCharacter}
        title={t("characters.undock")}><XCircle size={16} weight="light" color="currentColor" /></button>
    </div>
    <div class="character-dock-body">
      {#if characterDocked.physicalDescription}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("characters.physicalDescription")}</span>
          <p>{characterDocked.physicalDescription}</p>
        </div>
      {/if}
      {#if characterDocked.personality}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("characters.personality")}</span>
          <p>{characterDocked.personality}</p>
        </div>
      {/if}
      {#if characterDocked.traumas}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("characters.traumas")}</span>
          <p>{characterDocked.traumas}</p>
        </div>
      {/if}
      {#if characterDocked.relationships.length > 0}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("characters.relationships")}</span>
          <ul class="char-dock-rels">
            {#each characterDocked.relationships as rel}
              <li>{rel.targetName}{#if rel.type} — {rel.type}{/if}{#if rel.notes}: {rel.notes}{/if}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  </div>
{/if}

{#if noteDocked}
  <div class="character-dock" transition:fly={{ x: 300, duration: 200 }}>
    <div class="character-dock-header">
      <h3><PushPin size={16} weight="light" color="currentColor" aria-hidden="true" /> {noteDocked.title}</h3>
      <button class="character-dock-close" onclick={onCloseNote}
        title={t("characters.undock")}><XCircle size={16} weight="light" color="currentColor" /></button>
    </div>
    <div class="character-dock-body">
      <div class="char-dock-field">
        {@html noteDocked.content}
      </div>
    </div>
  </div>
{/if}

{#if placeDocked}
  <div class="character-dock" transition:fly={{ x: 300, duration: 200 }}>
    <div class="character-dock-header">
      <h3><PushPin size={16} weight="light" color="currentColor" aria-hidden="true" /> {placeDocked.name}</h3>
      <button class="character-dock-close" onclick={onClosePlace}
        title={t("characters.undock")}><XCircle size={16} weight="light" color="currentColor" /></button>
    </div>
    <div class="character-dock-body">
      {#if placeDocked.description}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("places.description")}</span>
          <p>{placeDocked.description}</p>
        </div>
      {/if}
      {#if placeDocked.notes}
        <div class="char-dock-field">
          <span class="char-dock-label">{t("places.notes")}</span>
          <p>{placeDocked.notes}</p>
        </div>
      {/if}
    </div>
  </div>
{/if}

{#if mediaDocked}
  <div class="character-dock" transition:fly={{ x: 300, duration: 200 }}>
    <div class="character-dock-header">
      <h3><PushPin size={16} weight="light" color="currentColor" aria-hidden="true" /> {mediaDocked}</h3>
      <button class="character-dock-close" onclick={onCloseMedia}
        title={t("characters.undock")}><XCircle size={16} weight="light" color="currentColor" /></button>
    </div>
    <div class="character-dock-body">
      <button
        style="border:none;background:transparent;padding:0;cursor:pointer;width:100%;"
        onclick={() => onMediaClick(mediaDocked!)}
      >
        <img src={mediaUrl(mediaDocked)} alt={mediaDocked}
          style="max-width:100%;max-height:60vh;min-height:120px;object-fit:contain;display:block;" />
      </button>
      <p style="font-size:0.6875rem;color:#64748b;margin-top:0.25rem;text-align:center;">{mediaDocked}</p>
    </div>
  </div>
{/if}

<style>
  /* ── Character dock panel ───────────────────────────────────── */
  .character-dock {
    position: absolute;
    top: 1rem;
    right: 1rem;
    width: 320px;
    max-height: calc(100% - 2rem);
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.12);
    z-index: 50;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  :global(.dark) .character-dock {
    background: #1e293b;
    border-color: #334155;
    box-shadow: 0 4px 24px rgba(0,0,0,0.4);
  }
  .character-dock-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e2e8f0;
    flex-shrink: 0;
  }
  :global(.dark) .character-dock-header {
    border-bottom-color: #334155;
  }
  .character-dock-header h3 {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
    color: #1e293b;
  }
  :global(.dark) .character-dock-header h3 {
    color: #f1f5f9;
  }
  .character-dock-close {
    background: none;
    border: none;
    font-size: 1.125rem;
    color: #64748b;
    cursor: pointer;
    padding: 0.25rem;
    line-height: 1;
    border-radius: 4px;
  }
  .character-dock-close:hover { color: #ef4444; }
  :global(.dark) .character-dock-close { color: #94a3b8; }
  :global(.dark) .character-dock-close:hover { color: #f87171; }
  .character-dock-body {
    padding: 1rem;
    overflow-y: auto;
    flex: 1;
  }
  .char-dock-field {
    margin-bottom: 0.75rem;
  }
  .char-dock-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #94a3b8;
    display: block;
    margin-bottom: 0.25rem;
  }
  .char-dock-field p {
    margin: 0;
    font-size: 0.8125rem;
    line-height: 1.5;
    color: #334155;
  }
  :global(.dark) .char-dock-field p {
    color: #cbd5e1;
  }
  .char-dock-rels {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .char-dock-rels li {
    font-size: 0.8125rem;
    color: #475569;
    padding: 0.25rem 0;
    border-bottom: 1px solid #f1f5f9;
  }
  :global(.dark) .char-dock-rels li {
    color: #94a3b8;
    border-bottom-color: #334155;
  }
</style>

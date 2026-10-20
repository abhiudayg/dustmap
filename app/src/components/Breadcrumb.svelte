<script lang="ts">
  import { appState, navigateToBreadcrumb } from '../lib/store.svelte';
</script>

<nav class="breadcrumb" aria-label="Folder navigation">
  {#each appState.breadcrumb as crumb, i}
    {#if i > 0}<span class="sep" aria-hidden="true">›</span>{/if}
    <button
      class="crumb"
      class:current={i === appState.breadcrumb.length - 1}
      onclick={() => navigateToBreadcrumb(i)}
    >
      {crumb.name}
    </button>
  {/each}
</nav>

<style>
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.4rem 1rem;
    background: #161B22;
    border-bottom: 1px solid #21262D;
    min-height: 36px;
    overflow-x: auto;
    user-select: none;
  }
  .sep {
    color: #6E7681;
    font-size: 0.8rem;
  }
  .crumb {
    background: none;
    border: none;
    color: #8B949E;
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    font-family: 'JetBrains Mono', monospace;
    white-space: nowrap;
    transition: all 0.1s ease;
  }
  .crumb:hover {
    color: #F0F6FC;
    background: #21262D;
  }
  .crumb.current {
    color: #F0F6FC;
    font-weight: 600;
    cursor: default;
  }
  .crumb.current:hover {
    background: none;
  }
</style>

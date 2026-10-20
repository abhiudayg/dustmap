<script lang="ts">
  import { appState, navigateToBreadcrumb } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  const mappedBytes = $derived(
    appState.summary
      ? (appState.metric === 'allocated' ? appState.summary.total_allocated : appState.summary.total_logical)
      : 0
  );
</script>

<nav class="breadcrumb-bar" aria-label="Folder navigation">
  <div class="crumbs-cluster">
    <span class="folder-icon">📂</span>
    {#each appState.breadcrumb as crumb, i}
      {#if i > 0}
        <span class="sep" aria-hidden="true">›</span>
      {/if}
      <button
        class="crumb"
        class:current={i === appState.breadcrumb.length - 1}
        onclick={() => navigateToBreadcrumb(i)}
      >
        {crumb.name}
      </button>
    {/each}
    {#if appState.summary}
      <span class="sep">/</span>
      <span class="mapped-size">{formatBytes(mappedBytes, appState.useBinaryUnits)} Mapped</span>
    {/if}
  </div>

  <div class="algo-badges">
    <span class="badge algo">Squarified Alg</span>
    <span class="badge depth">Depth: {appState.breadcrumb.length}</span>
  </div>
</nav>

<style>
  .breadcrumb-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.35rem 0.875rem;
    background: #101319;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    min-height: 32px;
    user-select: none;
    overflow-x: auto;
  }
  .crumbs-cluster {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    white-space: nowrap;
  }
  .folder-icon {
    font-size: 0.875rem;
  }
  .sep {
    color: #484F58;
    font-size: 0.75rem;
  }
  .crumb {
    background: none;
    border: none;
    color: #8B949E;
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    font-family: var(--font-mono);
    transition: all 0.12s;
  }
  .crumb:hover {
    color: #F0F6FC;
    background: #272A31;
  }
  .crumb.current {
    color: #F0F6FC;
    font-weight: 600;
    cursor: default;
  }
  .crumb.current:hover {
    background: none;
  }
  .mapped-size {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: #4FDCB8;
  }
  .algo-badges {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .badge {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    background: #1D2026;
    color: #8B949E;
    border: 1px solid #32353C;
  }
  .badge.algo {
    color: #F97316;
    background: rgba(249, 115, 22, 0.1);
    border-color: rgba(249, 115, 22, 0.3);
  }
</style>

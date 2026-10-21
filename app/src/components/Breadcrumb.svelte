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
      {#if appState.scanCachedAt}
        <span class="logical-size" title="Loaded from the saved scan. Press Scan Volume to rescan.">cached {new Date(appState.scanCachedAt * 1000).toLocaleString()}</span>
      {/if}
      {#if appState.summary && appState.metric === "allocated"}
        <span class="logical-size" title="Logical size: sum of declared file lengths, including sparse files">logical {formatBytes(appState.summary.total_logical, appState.useBinaryUnits)}</span>
      {/if}
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
    background: var(--color-surface);
    border-bottom: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.25 * 100%), transparent);
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
    font-size: var(--text-lg);
  }
  .sep {
    color: var(--color-outline-variant);
    font-size: var(--text-base);
  }
  .crumb {
    background: none;
    border: none;
    color: var(--color-outline);
    font-size: var(--text-base);
    cursor: pointer;
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    font-family: var(--font-sans);
    transition: all 0.12s;
  }
  .crumb:hover {
    color: var(--color-on-surface);
    background: var(--color-surface-container-high);
  }
  .crumb.current {
    color: var(--color-on-surface);
    font-weight: 600;
    cursor: default;
  }
  .crumb.current:hover {
    background: none;
  }
  .mapped-size {
    font-family: var(--font-sans);
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-secondary);
  }
  .algo-badges {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .badge {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    background: var(--color-surface-container-high);
    color: var(--color-outline);
    border: 1px solid var(--color-surface-container-highest);
  }
  .badge.algo {
    color: var(--color-primary-container);
    background: color-mix(in srgb, var(--color-primary-container) calc(0.1 * 100%), transparent);
    border-color: color-mix(in srgb, var(--color-primary-container) calc(0.3 * 100%), transparent);
  }
</style>

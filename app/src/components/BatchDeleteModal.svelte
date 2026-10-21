<script lang="ts">
  import { appState, performBatchDelete } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';
  import { getEntrySize } from '../lib/store.svelte';

  let understood = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  const items = $derived(Object.entries(appState.selection));
  const totalBytes = $derived(items.reduce((sum, [, entry]) => sum + getEntrySize(entry), 0));

  function close() {
    if (deleting) return;
    appState.batchDeleteOpen = false;
    understood = false;
    error = null;
  }

  async function run(permanent: boolean) {
    if (permanent && !understood) return;
    deleting = true;
    error = null;
    try {
      const results = await performBatchDelete(
        items.map(([path, entry]) => ({ path, entryId: entry.id })),
        permanent,
      );
      const failed = results.filter((result) => !result.success);
      if (failed.length > 0) {
        error = `Removed ${results.length - failed.length} of ${results.length}. Could not remove: ${failed
          .map((result) => `${result.path} (${result.error ?? 'unknown error'})`)
          .join('; ')}`;
      } else {
        close();
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      deleting = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

{#if appState.batchDeleteOpen && items.length > 0}
  <div class="modal-backdrop" onclick={close} role="presentation">
    <div class="modal-card" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Delete selected items" tabindex="-1">
      <div class="modal-header">
        <h2>Delete {items.length} selected item{items.length === 1 ? '' : 's'}</h2>
        <button class="close-btn" onclick={close} disabled={deleting} title="Close (Esc)">✕</button>
      </div>

      <p class="total">{formatBytes(totalBytes, appState.useBinaryUnits)} will be freed.</p>

      <div class="target-list" role="list">
        {#each items as [path, entry] (path)}
          <div class="target-row" role="listitem">
            <span class="target-path">{path}</span>
            <span class="target-size">{formatBytes(getEntrySize(entry), appState.useBinaryUnits)}</span>
          </div>
        {/each}
      </div>

      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}

      <label class="safety-check">
        <input type="checkbox" bind:checked={understood} />
        <span>I understand permanent deletion cannot be undone (needed only for Delete forever)</span>
      </label>

      <div class="actions">
        <button class="btn" onclick={() => run(false)} disabled={deleting}>
          {deleting ? 'Working…' : 'Move to Trash'}
        </button>
        <button class="btn danger" onclick={() => run(true)} disabled={deleting || !understood}>
          Delete forever
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
  }
  .modal-card {
    width: min(560px, 92vw);
    max-height: 86vh;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 10px;
    background: var(--color-surface-container-low);
    color: var(--color-on-surface);
    font-size: var(--text-base);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  h2 {
    font-size: var(--text-title, var(--text-xl));
    font-weight: 600;
  }
  .close-btn {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
  }
  .total {
    color: var(--color-on-surface-variant);
  }
  .target-list {
    min-height: 0;
    max-height: 220px;
    overflow-y: auto;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    background: var(--color-surface-container-lowest);
  }
  .target-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--color-outline-variant);
  }
  .target-path {
    min-width: 0;
    word-break: break-all;
  }
  .target-size {
    flex-shrink: 0;
    color: var(--color-on-surface-variant);
  }
  .error {
    color: var(--color-error);
  }
  .safety-check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .btn {
    padding: 6px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    background: var(--color-surface-container-high);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  .btn.danger {
    border-color: var(--color-error);
    color: var(--color-error);
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>

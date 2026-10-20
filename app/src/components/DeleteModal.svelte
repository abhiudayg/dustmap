<script lang="ts">
  import { appState, clearDeleteTarget, performDelete } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  let understood = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  async function doDelete(permanent: boolean) {
    if (!appState.deleteTarget) return;
    if (permanent && !understood) return;
    deleting = true;
    error = null;
    try {
      const fullPath = (appState.summary?.root ?? '') + '/' + appState.deleteTarget.path;
      await performDelete(fullPath, permanent);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      deleting = false;
    }
  }
</script>

{#if appState.deleteTarget}
  <div class="backdrop" onclick={clearDeleteTarget} role="presentation">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Confirm Deletion"
      tabindex="-1"
    >
      <div class="modal-header">
        <span class="warning-icon">⚠️</span>
        <h2>Confirm Deletion</h2>
      </div>

      <div class="modal-body">
        <p class="path">{appState.summary?.root}/{appState.deleteTarget.path}</p>
        <div class="size-display">
          {formatBytes(appState.deleteTarget.entry.total_logical ?? appState.deleteTarget.entry.logical_bytes)}
          <span class="size-label">will be reclaimed</span>
        </div>
        <p class="warning-text">
          DustMap adheres to a conservative, Trash-first policy. Items moved to Trash can be restored until emptied. Permanent deletion removes files immediately and cannot be undone.
        </p>
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <label class="understand-check">
          <input type="checkbox" bind:checked={understood} />
          <span>I understand that permanent deletion cannot be recovered</span>
        </label>
      </div>

      <div class="modal-actions">
        <button class="btn-cancel" onclick={clearDeleteTarget}>Cancel</button>
        <button class="btn-trash" onclick={() => doDelete(false)} disabled={deleting}>
          🗑 Move to Trash
        </button>
        <button class="btn-delete" onclick={() => doDelete(true)} disabled={deleting || !understood}>
          ☠ Delete Permanently
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(14, 17, 23, 0.85);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #161B22;
    border: 1px solid #F97316;
    border-radius: 10px;
    width: 540px;
    max-width: 90vw;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.6);
  }
  .modal-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .warning-icon {
    font-size: 1.25rem;
  }
  h2 {
    font-size: 1.125rem;
    font-weight: 700;
    color: #F0F6FC;
    margin: 0;
  }
  .path {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.8125rem;
    color: #8B949E;
    word-break: break-all;
    background: #0E1117;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #21262D;
    margin-bottom: 0.75rem;
  }
  .size-display {
    font-size: 2rem;
    font-weight: 700;
    color: #F97316;
    font-family: 'JetBrains Mono', monospace;
    margin-bottom: 0.5rem;
  }
  .size-label {
    font-size: 0.875rem;
    color: #8B949E;
    font-family: Inter, -apple-system, sans-serif;
    margin-left: 0.5rem;
  }
  .warning-text {
    color: #8B949E;
    font-size: 0.85rem;
    line-height: 1.45;
    margin-bottom: 0.75rem;
  }
  .error {
    color: #EF4444;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }
  .understand-check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #F0F6FC;
    font-size: 0.85rem;
    cursor: pointer;
    user-select: none;
  }
  .modal-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    padding-top: 0.75rem;
    border-top: 1px solid #21262D;
  }
  .btn-cancel {
    background: none;
    border: 1px solid #21262D;
    color: #8B949E;
    padding: 0.45rem 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.1s ease;
  }
  .btn-cancel:hover {
    border-color: #8B949E;
    color: #F0F6FC;
  }
  .btn-trash {
    background: none;
    border: 1px solid #14B8A6;
    color: #14B8A6;
    padding: 0.45rem 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: all 0.1s ease;
  }
  .btn-trash:hover {
    background: rgba(20, 184, 166, 0.12);
  }
  .btn-delete {
    background: #EF4444;
    border: none;
    color: white;
    padding: 0.45rem 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: background 0.15s ease;
  }
  .btn-delete:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .btn-delete:hover:not(:disabled) {
    background: #DC2626;
  }
</style>

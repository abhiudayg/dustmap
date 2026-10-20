<script lang="ts">
  import { appState, navigateTo, setDeleteTarget } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  const sorted = $derived(
    [...appState.currentChildren]
      .sort((a, b) => ((b.total_logical ?? b.logical_bytes) || 0) - ((a.total_logical ?? a.logical_bytes) || 0))
      .slice(0, 20)
  );

  const maxSize = $derived(
    sorted.length > 0 ? (sorted[0].total_logical ?? sorted[0].logical_bytes) || 1 : 1
  );
</script>

<div class="panel">
  <div class="panel-header">
    <span>Top Items</span>
    <span class="item-count">{appState.currentChildren.length} items</span>
  </div>
  <div class="list">
    {#each sorted as entry, i}
      {@const size = entry.total_logical ?? entry.logical_bytes ?? 0}
      {@const pct = Math.round((size / maxSize) * 100)}
      <div
        class="row"
        role="button"
        tabindex="0"
        onclick={() => navigateTo(entry)}
        onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && navigateTo(entry)}
      >
        <span class="rank">{i + 1}</span>
        <span class="icon">{entry.kind === 'Directory' ? (entry.is_hidden ? '🔒' : '📁') : '📄'}</span>
        <div class="name-col">
          <span class="name" class:hidden-name={entry.is_hidden}>{entry.name}</span>
          {#if entry.child_count !== undefined && entry.kind === 'Directory'}
            <span class="child-count">{entry.child_count} items</span>
          {/if}
        </div>
        <span class="size">{formatBytes(size)}</span>
        <div class="bar-track">
          <div class="bar-fill" style="width: {pct}%"></div>
        </div>
        <button
          class="del-btn"
          onclick={(e) => {
            e.stopPropagation();
            setDeleteTarget(entry.name, entry);
          }}
          title="Delete or Move to Trash"
          aria-label="Delete {entry.name}"
        >
          🗑
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #161B22;
    border-left: 1px solid #21262D;
    user-select: none;
  }
  .panel-header {
    padding: 0.75rem 1rem;
    font-weight: 600;
    font-size: 0.875rem;
    color: #F0F6FC;
    border-bottom: 1px solid #21262D;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .item-count {
    color: #8B949E;
    font-size: 0.75rem;
    font-weight: 400;
    font-family: 'JetBrains Mono', monospace;
  }
  .list {
    flex: 1;
    overflow-y: auto;
  }
  .row {
    display: grid;
    grid-template-columns: 22px 22px 1fr auto 50px 24px;
    gap: 0.5rem;
    align-items: center;
    padding: 0.45rem 0.75rem;
    border-bottom: 1px solid #21262D;
    cursor: pointer;
    transition: background 0.1s ease;
  }
  .row:hover {
    background: #1C2128;
  }
  .row:hover .del-btn {
    opacity: 1;
  }
  .rank {
    color: #6E7681;
    font-size: 0.75rem;
    font-family: 'JetBrains Mono', monospace;
  }
  .icon {
    font-size: 0.875rem;
  }
  .name-col {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .name {
    font-size: 0.8125rem;
    color: #F0F6FC;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hidden-name {
    color: #8B949E;
    font-style: italic;
  }
  .child-count {
    font-size: 0.6875rem;
    color: #6E7681;
  }
  .size {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: #8B949E;
    white-space: nowrap;
  }
  .bar-track {
    height: 4px;
    background: #21262D;
    border-radius: 2px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    background: #F97316;
    border-radius: 2px;
  }
  .del-btn {
    background: none;
    border: none;
    cursor: pointer;
    opacity: 0;
    font-size: 0.875rem;
    padding: 0;
    transition: opacity 0.15s ease;
    color: #8B949E;
  }
  .del-btn:hover {
    color: #EF4444;
  }
</style>

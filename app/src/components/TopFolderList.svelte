<script lang="ts">
  import { appState, navigateTo, setDeleteTarget, getEntrySize } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  const sorted = $derived(
    [...appState.currentChildren]
      .sort((a, b) => getEntrySize(b) - getEntrySize(a))
      .slice(0, 30)
  );

  const maxSize = $derived(
    sorted.length > 0 ? getEntrySize(sorted[0]) || 1 : 1
  );
</script>

<div class="top-consumers-panel">
  <!-- Right Pane Header Toolbar matching Stitch -->
  <div class="panel-header">
    <div class="title-cluster">
      <span class="header-icon">📊</span>
      <h2 class="panel-title">Top Consumers</h2>
      <span class="count-chip">{appState.currentChildren.length} items</span>
    </div>
    <div class="sort-action">
      <span class="sort-badge">Size: Desc ↓</span>
    </div>
  </div>

  <!-- High-Density Telemetry Table Header -->
  <div class="table-header">
    <div class="col-rank">#</div>
    <div class="col-node">Path / Node</div>
    <div class="col-size">Size</div>
    <div class="col-actions">Actions</div>
  </div>

  <!-- Ranked Scrollable List Container -->
  <div class="table-body">
    {#each sorted as entry, i}
      {@const size = getEntrySize(entry)}
      {@const pct = Math.max(2, Math.round((size / maxSize) * 100))}
      <div
        class="table-row"
        class:is-active={i === 0}
        role="button"
        tabindex="0"
        onclick={() => navigateTo(entry)}
        onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && navigateTo(entry)}
      >
        <!-- Rank Number -->
        <div class="col-rank rank-num">{String(i + 1).padStart(2, '0')}</div>

        <!-- Node & Gauge -->
        <div class="col-node node-cell">
          <div class="node-meta">
            <span class="node-dot" class:orange={size > 10_000_000_000} class:teal={size <= 10_000_000_000}></span>
            <span class="node-icon">{entry.kind === 'Directory' ? (entry.is_hidden ? '🔒' : '📁') : '📄'}</span>
            <span class="node-name" class:hidden-node={entry.is_hidden}>{entry.name}</span>
            {#if entry.child_count !== undefined && entry.kind === 'Directory'}
              <span class="child-tag">{entry.child_count}</span>
            {/if}
          </div>
          <!-- Horizontal Mini Proportion Gauge directly under name -->
          <div class="mini-gauge-track">
            <div
              class="mini-gauge-fill"
              class:high={size > 20_000_000_000}
              style="width: {pct}%"
            ></div>
          </div>
        </div>

        <!-- Formatted Byte Size -->
        <div class="col-size size-cell">
          {formatBytes(size, appState.useBinaryUnits)}
        </div>

        <!-- Actions -->
        <div class="col-actions actions-cell">
          <button
            class="action-trash-btn"
            onclick={(e) => {
              e.stopPropagation();
              setDeleteTarget(entry.path, entry);
            }}
            title="Move {entry.name} to Trash"
            aria-label="Delete {entry.name}"
          >
            🗑
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .top-consumers-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #101319;
    border-left: 1px solid rgba(88, 66, 55, 0.25);
    user-select: none;
    overflow: hidden;
  }
  .panel-header {
    height: 38px;
    padding: 0 12px;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #161B22;
  }
  .title-cluster {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .header-icon {
    font-size: 0.9375rem;
  }
  .panel-title {
    font-family: Inter, -apple-system, sans-serif;
    font-size: 0.8125rem;
    font-weight: 600;
    color: #F0F6FC;
    margin: 0;
  }
  .count-chip {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    padding: 1px 6px;
    border-radius: 4px;
    background: #272A31;
    color: #8B949E;
  }
  .sort-action {
    display: flex;
    align-items: center;
  }
  .sort-badge {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    color: #8B949E;
    background: #1D2026;
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid #32353C;
  }

  /* Table Header */
  .table-header {
    height: 24px;
    min-height: 24px;
    background: #161B22;
    padding: 0 12px;
    display: flex;
    align-items: center;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    color: #8B949E;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .col-rank {
    width: 28px;
    text-align: center;
  }
  .col-node {
    flex: 1;
    min-width: 0;
    padding: 0 8px;
  }
  .col-size {
    width: 80px;
    text-align: right;
  }
  .col-actions {
    width: 38px;
    text-align: right;
  }

  /* Table Body & Rows */
  .table-body {
    flex: 1;
    overflow-y: auto;
    divide-y: 1px solid rgba(88, 66, 55, 0.15);
  }
  .table-row {
    height: 40px;
    min-height: 40px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    cursor: pointer;
    transition: background 0.1s ease;
    border-bottom: 1px solid rgba(88, 66, 55, 0.12);
  }
  .table-row:hover {
    background: #1D2026;
  }
  .table-row.is-active {
    border-left: 2px solid #F97316;
    background: rgba(249, 115, 22, 0.05);
  }
  .table-row:hover .action-trash-btn {
    opacity: 1;
  }
  .rank-num {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.6875rem;
    color: #8B949E;
  }
  .table-row.is-active .rank-num {
    color: #F97316;
    font-weight: 700;
  }

  /* Node Cell */
  .node-cell {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
  }
  .node-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .node-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .node-dot.orange {
    background: #F97316;
    box-shadow: 0 0 6px rgba(249, 115, 22, 0.5);
  }
  .node-dot.teal {
    background: #14B8A6;
  }
  .node-icon {
    font-size: 0.8125rem;
    flex-shrink: 0;
  }
  .node-name {
    font-family: Inter, -apple-system, sans-serif;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #F0F6FC;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .node-name.hidden-node {
    color: #8B949E;
    font-style: italic;
  }
  .child-tag {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.5625rem;
    color: #8B949E;
    background: #1D2026;
    padding: 0 4px;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .mini-gauge-track {
    width: 100%;
    height: 3px;
    background: #1D2026;
    border-radius: 2px;
    overflow: hidden;
  }
  .mini-gauge-fill {
    height: 100%;
    background: #4FDCB8;
    border-radius: 2px;
  }
  .mini-gauge-fill.high {
    background: #F97316;
  }

  /* Size Cell */
  .size-cell {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .table-row.is-active .size-cell {
    color: #F97316;
  }

  /* Actions */
  .actions-cell {
    display: flex;
    justify-content: flex-end;
  }
  .action-trash-btn {
    background: none;
    border: none;
    font-size: 0.875rem;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
    padding: 2px 4px;
    border-radius: 4px;
  }
  .action-trash-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    filter: drop-shadow(0 0 4px rgba(239, 68, 68, 0.5));
  }
</style>

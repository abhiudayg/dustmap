<script lang="ts">
  import { appState } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';
</script>

<div class="status-bar">
  {#if appState.summary}
    <div class="status-group">
      <span class="status-dot"></span>
      <span>Scanned: {appState.summary.root}</span>
    </div>
    <span class="sep">·</span>
    <span>{appState.summary.total_items.toLocaleString()} entries</span>
    <span class="sep">·</span>
    <span>{formatBytes(appState.summary.total_logical)} accounted</span>
    <span class="sep">·</span>
    <span>{appState.summary.duration_secs.toFixed(2)}s scan</span>
    {#if appState.summary.error_count > 0}
      <span class="sep">·</span>
      <span class="warn">{appState.summary.error_count} permission warnings</span>
    {/if}
  {:else}
    <span>Ready — select a volume or folder to scan</span>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 1rem;
    height: 26px;
    background: #161B22;
    border-top: 1px solid #21262D;
    font-size: 0.6875rem;
    color: #8B949E;
    font-family: 'JetBrains Mono', monospace;
    user-select: none;
  }
  .status-group {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #14B8A6;
  }
  .sep {
    color: #30363D;
  }
  .warn {
    color: #F59E0B;
  }
</style>

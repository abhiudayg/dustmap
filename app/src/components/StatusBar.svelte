<script lang="ts">
  import { appState, toggleMetric, toggleUnits } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  const accountedBytes = $derived(
    appState.summary
      ? (appState.metric === 'allocated' ? appState.summary.total_allocated : appState.summary.total_logical)
      : 0
  );
</script>

<footer class="status-footer">
  <!-- Left Telemetry Cluster -->
  <div class="status-left">
    <span class="pulse-status-dot"></span>
    {#if appState.summary}
      <span class="telemetry-text">
        INDEXED: {appState.summary.total_items.toLocaleString()} ITEMS | {formatBytes(accountedBytes, appState.useBinaryUnits)} {appState.metric === 'allocated' ? 'ALLOCATED' : 'LOGICAL'} | {appState.summary.duration_secs.toFixed(1)}S
      </span>
    {:else if appState.isScanning}
      <span class="telemetry-text scanning">
        SCANNING INODE NODES | PARALLEL ENGINE ACTIVE…
      </span>
    {:else}
      <span class="telemetry-text ready">
        SYSTEM READY | SELECT OR SCAN MOUNTED VOLUME
      </span>
    {/if}
  </div>

  <!-- Right Metadata & Privacy Warnings Cluster -->
  <div class="status-right">
    {#if appState.summary && appState.summary.error_count > 0}
      <button
        class="perm-warning-btn"
        onclick={() => (appState.showPermissionDialog = true)}
        title="macOS Full Disk Access (TCC) Privacy Controls"
      >
        ⚠️ {appState.summary.error_count.toLocaleString()} PERMISSION WARNINGS
      </button>
    {/if}

    <button
      class="meta-toggle-btn"
      class:active={appState.metric === 'allocated'}
      onclick={toggleMetric}
      title="Toggle Physical Storage (Allocated) vs Virtual Size (Logical)"
    >
      METRIC: <strong>{appState.metric === 'allocated' ? 'ALLOC' : 'RAW'}</strong>
    </button>

    <button
      class="meta-toggle-btn"
      onclick={toggleUnits}
      title="Toggle Decimal 1000 base vs Binary 1024 base"
    >
      UNITS: <strong>{appState.useBinaryUnits ? 'GiB' : 'GB'}</strong>
    </button>

    <span class="fs-tag">
      {appState.currentVolume?.fs_type || 'APFS'} (Encrypted)
    </span>
  </div>
</footer>

<style>
  .status-footer {
    height: 26px;
    min-height: 26px;
    background: #0B0E14;
    border-top: 1px solid rgba(88, 66, 55, 0.25);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    color: #8B949E;
    user-select: none;
    z-index: 40;
  }
  .status-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .pulse-status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #4FDCB8;
    box-shadow: 0 0 6px rgba(79, 220, 184, 0.5);
    flex-shrink: 0;
  }
  .telemetry-text {
    color: #8B949E;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .telemetry-text.scanning {
    color: #F97316;
  }
  .telemetry-text.ready {
    color: #4FDCB8;
  }
  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .perm-warning-btn {
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.35);
    color: #F59E0B;
    border-radius: 4px;
    padding: 1px 6px;
    font-family: inherit;
    font-size: 0.5625rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }
  .perm-warning-btn:hover {
    background: rgba(245, 158, 11, 0.25);
    color: #FBBF24;
  }
  .meta-toggle-btn {
    background: #161B22;
    border: 1px solid #272A31;
    color: #8B949E;
    border-radius: 4px;
    padding: 1px 5px;
    font-family: inherit;
    font-size: 0.5625rem;
    cursor: pointer;
    transition: all 0.15s;
  }
  .meta-toggle-btn:hover {
    color: #F0F6FC;
    border-color: #3D4450;
  }
  .meta-toggle-btn.active {
    color: #14B8A6;
    border-color: rgba(20, 184, 166, 0.4);
    background: rgba(20, 184, 166, 0.08);
  }
  .meta-toggle-btn strong {
    font-weight: 700;
  }
  .fs-tag {
    color: #484F58;
    font-size: 0.5625rem;
  }
</style>

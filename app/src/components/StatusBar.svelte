<script lang="ts">
  import { appState, toggleMetric, toggleUnits } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  const vol = $derived(appState.currentVolume);
  const summary = $derived(appState.summary);
  const isScanning = $derived(appState.isScanning);
  const errorCount = $derived(summary?.error_count ?? 0);

  // Approximate purgeable cache size from current items
  const purgeableBytes = $derived.by(() => {
    return appState.currentChildren
      .filter(e => {
        const n = e.name.toLowerCase();
        return n.includes('cache') || n.includes('tmp') || n.includes('deriveddata');
      })
      .reduce((sum, e) => sum + (e.total_allocated ?? e.allocated_bytes ?? e.total_logical ?? e.logical_bytes ?? 0), 0);
  });
</script>

<!-- Stitch Pinned Bottom Status Bar (26px single line) -->
<footer class="stitch-footer font-mono">
  <!-- Left Side: Status beacon + Indexed Telemetry -->
  <div class="footer-left">
    <div class="status-indicator">
      <span class="beacon-pulse" class:scanning={isScanning}></span>
      <span class="status-state-text" class:scanning={isScanning}>
        {isScanning ? 'Telemetry Active…' : (summary ? 'Telemetry Idle' : 'System Ready')}
      </span>
    </div>

    <span class="pipe-sep">|</span>

    <span class="telemetry-item">
      INDEXED: {summary ? summary.total_items.toLocaleString() : '0'} ITEMS
    </span>

    {#if summary}
      <span class="pipe-sep">|</span>
      <span class="telemetry-item">
        DURATION: {summary.duration_secs.toFixed(1)}S
      </span>

      <span class="pipe-sep">|</span>
      <span class="telemetry-item">
        MOUNT: {vol?.mount_point || '/'}
      </span>
    {/if}
  </div>

  <!-- Right Side: Filesystem + Purgeable Pill + Warning + Metric Toggle -->
  <div class="footer-right">
    <span class="fs-label">{vol?.fs_type || 'APFS'} (Encrypted)</span>

    {#if purgeableBytes > 0}
      <span class="dot-sep">•</span>
      <span class="purgeable-pill">
        {formatBytes(purgeableBytes, appState.useBinaryUnits)} Purgeable
      </span>
    {/if}

    {#if errorCount > 0}
      <span class="dot-sep">•</span>
      <button
        class="perm-badge-btn"
        onclick={() => appState.showPermissionDialog = true}
        title="Click to view macOS Full Disk Access guide"
      >
        ⚠️ {errorCount.toLocaleString()} FDA Warnings
      </button>
    {/if}

    <span class="dot-sep">•</span>
    <button
      class="quick-toggle-btn"
      onclick={toggleMetric}
      title="Toggle allocated physical disk blocks vs logical size"
    >
      Metric: {appState.metric === 'allocated' ? 'Allocated' : 'Logical'}
    </button>

    <button
      class="quick-toggle-btn"
      onclick={toggleUnits}
      title="Toggle decimal (1000) vs binary (1024) byte units"
    >
      {appState.useBinaryUnits ? 'GiB (1024)' : 'GB (1000)'}
    </button>
  </div>
</footer>

<style>
  .stitch-footer {
    height: 26px;
    min-height: 26px;
    background: #0B0E14;
    border-top: 1px solid rgba(88, 66, 55, 0.3);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-size: 0.625rem;
    color: #8B949E;
    user-select: none;
    z-index: 40;
    white-space: nowrap;
    overflow: hidden;
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    overflow: hidden;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .beacon-pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #10B981;
    box-shadow: 0 0 6px #10B981;
    flex-shrink: 0;
    animation: beacon 2s infinite ease-in-out;
  }
  .beacon-pulse.scanning {
    background: #F97316;
    box-shadow: 0 0 8px #F97316;
    animation: beacon 0.8s infinite ease-in-out;
  }
  @keyframes beacon {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(1.2); }
  }

  .status-state-text {
    color: #F0F6FC;
    font-weight: 500;
  }
  .status-state-text.scanning {
    color: #F97316;
  }

  .pipe-sep {
    color: rgba(139, 148, 158, 0.3);
    margin: 0 2px;
  }

  .telemetry-item {
    color: #8B949E;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .fs-label {
    color: #8B949E;
  }

  .dot-sep {
    color: rgba(139, 148, 158, 0.4);
  }

  .purgeable-pill {
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(20, 184, 166, 0.15);
    color: #14B8A6;
    border: 1px solid rgba(20, 184, 166, 0.35);
    font-weight: 600;
  }

  .perm-badge-btn {
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.35);
    color: #F59E0B;
    border-radius: 4px;
    padding: 1px 6px;
    font-family: inherit;
    font-size: 0.5625rem;
    cursor: pointer;
    transition: all 0.12s;
  }
  .perm-badge-btn:hover {
    background: rgba(245, 158, 11, 0.25);
    color: #FBBF24;
  }

  .quick-toggle-btn {
    background: #191C22;
    border: 1px solid rgba(88, 66, 55, 0.35);
    color: #8B949E;
    border-radius: 4px;
    padding: 1px 6px;
    font-family: inherit;
    font-size: 0.5625rem;
    cursor: pointer;
    transition: all 0.12s;
  }
  .quick-toggle-btn:hover {
    color: #F0F6FC;
    border-color: #14B8A6;
  }
</style>
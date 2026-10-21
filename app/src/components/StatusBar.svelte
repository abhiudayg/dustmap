<script lang="ts">
  import { appState, toggleMetric, toggleUnits, getEntrySize } from '../lib/store.svelte';
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
      .reduce((sum, e) => sum + getEntrySize(e), 0);
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
    background: var(--color-surface-container-lowest);
    border-top: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.3 * 100%), transparent);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-size: var(--text-sm);
    color: var(--color-outline);
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
    background: var(--color-secondary);
    box-shadow: 0 0 6px var(--color-secondary);
    flex-shrink: 0;
    animation: beacon 2s infinite ease-in-out;
  }
  .beacon-pulse.scanning {
    background: var(--color-primary-container);
    box-shadow: 0 0 8px var(--color-primary-container);
    animation: beacon 0.8s infinite ease-in-out;
  }
  @keyframes beacon {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(1.2); }
  }

  .status-state-text {
    color: var(--color-on-surface);
    font-weight: 500;
  }
  .status-state-text.scanning {
    color: var(--color-primary-container);
  }

  .pipe-sep {
    color: color-mix(in srgb, var(--color-outline) calc(0.3 * 100%), transparent);
    margin: 0 2px;
  }

  .telemetry-item {
    color: var(--color-outline);
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
    color: var(--color-outline);
  }

  .dot-sep {
    color: color-mix(in srgb, var(--color-outline) calc(0.4 * 100%), transparent);
  }

  .purgeable-pill {
    padding: 1px 6px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--color-secondary) calc(0.15 * 100%), transparent);
    color: var(--color-secondary);
    border: 1px solid color-mix(in srgb, var(--color-secondary) calc(0.35 * 100%), transparent);
    font-weight: 600;
  }

  .perm-badge-btn {
    background: color-mix(in srgb, var(--color-warning) calc(0.12 * 100%), transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) calc(0.35 * 100%), transparent);
    color: var(--color-warning);
    border-radius: 4px;
    padding: 1px 6px;
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
    transition: all 0.12s;
  }
  .perm-badge-btn:hover {
    background: color-mix(in srgb, var(--color-warning) calc(0.25 * 100%), transparent);
    color: var(--color-warning);
  }

  .quick-toggle-btn {
    background: var(--color-surface-container-low);
    border: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.35 * 100%), transparent);
    color: var(--color-outline);
    border-radius: 4px;
    padding: 1px 6px;
    font-family: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
    transition: all 0.12s;
  }
  .quick-toggle-btn:hover {
    color: var(--color-on-surface);
    border-color: var(--color-secondary);
  }
</style>
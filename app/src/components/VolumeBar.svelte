<script lang="ts">
  import { appState, selectVolume, rescanCurrentVolume, loadVolumes, toggleMetric } from '../lib/store.svelte';
  import { formatBytes, usagePercent } from '../lib/format';
  import { onMount } from 'svelte';

  onMount(() => {
    loadVolumes();
  });
</script>

<!-- 1. macOS Pinned Titlebar (38px height matching Stitch design) -->
<header class="titlebar" data-tauri-drag-region>
  <!-- Traffic light window control offsets -->
  <div class="traffic-lights-area">
    <span class="light close"></span>
    <span class="light min"></span>
    <span class="light max"></span>
  </div>

  <!-- Centered Brand Identity Anchor -->
  <div class="brand-anchor">
    <img src="/favicon.svg" class="brand-icon" alt="DustMap" />
    <span class="brand-title">DustMap</span>
    <span class="brand-badge">v0.1.0</span>
  </div>

  <!-- Right Window Quick Actions -->
  <div class="header-actions">
    <button
      class="action-icon-btn"
      onclick={toggleMetric}
      title="Toggle Metric: {appState.metric === 'allocated' ? 'Allocated Physical Disk Space' : 'Logical Raw File Size'}"
    >
      <span class="metric-pill" class:alloc={appState.metric === 'allocated'}>
        {appState.metric === 'allocated' ? 'ALLOC' : 'RAW'}
      </span>
    </button>
  </div>
</header>

<!-- 2. Docked Top Toolbar (40px height with Volume Selector Chips matching Stitch) -->
<section class="toolbar">
  <!-- Left: Volume Selector Chips -->
  <div class="volumes-cluster">
    {#each appState.volumes as vol}
      {@const pct = usagePercent(vol.used_bytes, vol.total_bytes)}
      {@const active = appState.currentVolume?.mount_point === vol.mount_point}
      <button
        class="vol-chip"
        class:active
        onclick={() => selectVolume(vol, false)}
        title="Switch to {vol.name || vol.mount_point} (cached)"
      >
        {#if active}
          <span class="active-pulse"></span>
        {/if}
        <span class="vol-icon">💾</span>
        <span class="vol-label">{vol.name || vol.mount_point}</span>
        <span class="vol-stats">
          {formatBytes(vol.used_bytes, appState.useBinaryUnits)} / {formatBytes(vol.total_bytes, appState.useBinaryUnits)}
        </span>
        <div class="vol-gauge-track">
          <div class="vol-gauge-fill" style="width: {pct}%"></div>
        </div>
      </button>
    {/each}
  </div>

  <!-- Right: Rescan Volume Action Button -->
  <div class="toolbar-controls">
    <button
      class="rescan-btn"
      onclick={rescanCurrentVolume}
      disabled={appState.isScanning}
      title="Perform full rescan of current volume (⌘R)"
    >
      <span class="refresh-icon" class:spinning={appState.isScanning}>⟳</span>
      <span>{appState.isScanning ? 'Scanning…' : 'Rescan Volume'}</span>
      <kbd class="kbd-badge">⌘R</kbd>
    </button>
  </div>
</section>

<style>
  /* Titlebar 38px */
  .titlebar {
    height: 38px;
    min-height: 38px;
    background: rgba(11, 14, 20, 0.95);
    backdrop-filter: blur(16px);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    user-select: none;
    z-index: 50;
  }
  .traffic-lights-area {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 70px;
  }
  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
  .light.close {
    background: #FF5F56;
    border: 1px solid #E0443E;
  }
  .light.min {
    background: #FFBD2E;
    border: 1px solid #DEA123;
  }
  .light.max {
    background: #27C93F;
    border: 1px solid #1AAB29;
  }
  .brand-anchor {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand-icon {
    width: 18px;
    height: 18px;
    border-radius: 4px;
  }
  .brand-title {
    font-family: Inter, -apple-system, sans-serif;
    font-weight: 700;
    font-size: 0.9375rem;
    color: #F0F6FC;
    letter-spacing: -0.02em;
  }
  .brand-badge {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.6875rem;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 4px;
    background: #272A31;
    color: #4FDCB8;
    border: 1px solid rgba(88, 66, 55, 0.4);
  }
  .header-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    width: 70px;
  }
  .action-icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px;
  }
  .metric-pill {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    background: #1D2026;
    color: #8B949E;
    border: 1px solid #32353C;
    transition: all 0.15s;
  }
  .metric-pill.alloc {
    background: rgba(20, 184, 166, 0.15);
    color: #14B8A6;
    border-color: rgba(20, 184, 166, 0.4);
  }

  /* Toolbar 40px */
  .toolbar {
    height: 40px;
    min-height: 40px;
    background: rgba(16, 19, 25, 0.95);
    backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    user-select: none;
  }
  .volumes-cluster {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    overflow-x: auto;
  }
  .vol-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    border-radius: 8px;
    background: transparent;
    border: 1px solid transparent;
    color: #8B949E;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease-out;
    white-space: nowrap;
  }
  .vol-chip:hover {
    background: #272A31;
    color: #F0F6FC;
    border-color: rgba(88, 66, 55, 0.4);
  }
  .vol-chip.active {
    background: #272A31;
    color: #F0F6FC;
    border-color: rgba(249, 115, 22, 0.5);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }
  .active-pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #F97316;
    box-shadow: 0 0 8px #F97316;
    animation: pulse 1.8s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }
  .vol-icon {
    font-size: 0.8125rem;
  }
  .vol-label {
    font-family: Inter, -apple-system, sans-serif;
    font-weight: 600;
    color: inherit;
    font-size: 0.8125rem;
  }
  .vol-stats {
    font-size: 0.6875rem;
    color: #8B949E;
  }
  .vol-gauge-track {
    width: 44px;
    height: 5px;
    background: #1D2026;
    border-radius: 3px;
    overflow: hidden;
  }
  .vol-gauge-fill {
    height: 100%;
    background: #F97316;
    border-radius: 3px;
  }
  .toolbar-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .rescan-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 6px;
    background: #F97316;
    color: #0E1117;
    border: none;
    font-family: Inter, -apple-system, sans-serif;
    font-weight: 600;
    font-size: 0.78125rem;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(249, 115, 22, 0.35);
    transition: all 0.15s;
  }
  .rescan-btn:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-0.5px);
  }
  .rescan-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .refresh-icon {
    font-size: 0.875rem;
    font-weight: bold;
  }
  .refresh-icon.spinning {
    display: inline-block;
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .kbd-badge {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    background: rgba(0, 0, 0, 0.2);
    color: #0E1117;
    padding: 1px 4px;
    border-radius: 3px;
    font-weight: 700;
  }
</style>

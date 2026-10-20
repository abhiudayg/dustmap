<script lang="ts">
  import { appState, selectVolume, rescanCurrentVolume, loadVolumes, toggleMetric, startCurrentVolumeScan } from '../lib/store.svelte';
  import { formatBytes, usagePercent } from '../lib/format';
  import { onMount } from 'svelte';

  let searchInput: HTMLInputElement;

  onMount(() => {
    loadVolumes();

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'r') {
        e.preventDefault();
        if (!appState.isScanning) {
          rescanCurrentVolume();
        }
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {
        e.preventDefault();
        searchInput?.focus();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<!-- Stitch Unified TopBar: macOS Titlebar & Navigation (44px) -->
<header class="stitch-header" data-tauri-drag-region>
  <!-- Left: Traffic Lights & Branding & Search -->
  <div class="header-left">
    <!-- Traffic lights -->
    <div class="traffic-lights">
      <span class="light close" title="Close"></span>
      <span class="light min" title="Minimize"></span>
      <span class="light max" title="Fullscreen"></span>
    </div>

    <div class="header-divider"></div>

    <!-- App Brand -->
    <div class="brand-group">
      <div class="brand-badge-icon">
        <span class="material-symbols-outlined brand-glyph">radar</span>
        <span class="beacon-dot"></span>
      </div>
      <span class="brand-name">DustMap</span>
      <span class="brand-ver">v0.1.0</span>
    </div>

    <!-- Quick Search Bar -->
    <div class="search-container">
      <span class="material-symbols-outlined search-icon">search</span>
      <input
        bind:this={searchInput}
        bind:value={appState.searchQuery}
        type="text"
        placeholder="Search path, extension, or inode..."
        class="search-input"
      />
      {#if appState.searchQuery}
        <button class="search-clear" onclick={() => appState.searchQuery = ''}>×</button>
      {:else}
        <kbd class="search-kbd">⌘F</kbd>
      {/if}
    </div>
  </div>

  <!-- Center: Volume Selector Chips Row -->
  <div class="volume-selector-cluster">
    {#each appState.volumes as vol}
      {@const pct = usagePercent(vol.used_bytes, vol.total_bytes)}
      {@const active = appState.currentVolume?.mount_point === vol.mount_point}
      <button
        class="volume-chip"
        class:active
        onclick={() => selectVolume(vol, false)}
        title="Volume: {vol.name || vol.mount_point} ({vol.fs_type})"
      >
        <span class="material-symbols-outlined drive-icon">
          {vol.is_removable ? 'usb' : 'hard_drive'}
        </span>
        <div class="drive-meta">
          <div class="drive-row">
            <span class="drive-title">{vol.name || 'Macintosh HD'}</span>
            <span class="drive-type">({vol.is_removable ? 'External' : 'Internal SSD'})</span>
          </div>
          <div class="gauge-row">
            <div class="gauge-bar">
              <div class="gauge-fill" style="width: {pct}%"></div>
            </div>
            <span class="gauge-label font-mono">
              {formatBytes(vol.used_bytes, appState.useBinaryUnits)} / {formatBytes(vol.total_bytes, appState.useBinaryUnits)}
            </span>
          </div>
        </div>
      </button>
    {/each}
  </div>

  <!-- Right: Actions -->
  <div class="header-right">
    <!-- Purge Cache / Quick Clean -->
    <button
      class="btn-purge"
      onclick={() => {
        appState.activeFilter = appState.activeFilter === 'cache' ? 'all' : 'cache';
      }}
      title="Filter / highlight purgeable developer caches"
    >
      <span class="material-symbols-outlined icon-mop">mop</span>
      <span>Purge Cache</span>
    </button>

    <!-- Rescan Volume (Primary Accent) -->
    <button
      class="btn-rescan"
      onclick={() => {
        if (!appState.scanId) {
          startCurrentVolumeScan();
        } else {
          rescanCurrentVolume();
        }
      }}
      disabled={appState.isScanning}
      title="Scan or rescan active volume (⌘R)"
    >
      <span class="material-symbols-outlined icon-refresh" class:spinning={appState.isScanning}>
        refresh
      </span>
      <span>{appState.isScanning ? 'Scanning…' : (appState.scanId ? 'Rescan Volume' : 'Scan Volume')}</span>
      <kbd class="kbd-pill">⌘R</kbd>
    </button>

    <div class="header-divider"></div>

    <!-- Metric Toggle (Allocated Physical vs Logical Raw) -->
    <button
      class="btn-icon-square"
      onclick={toggleMetric}
      title="Toggle Metric: {appState.metric === 'allocated' ? 'Allocated Physical Blocks' : 'Logical Raw Bytes'}"
    >
      <span class="metric-flag" class:alloc={appState.metric === 'allocated'}>
        {appState.metric === 'allocated' ? 'ALLOC' : 'RAW'}
      </span>
    </button>

    <!-- Tune / Filter Rules -->
    <button
      class="btn-icon-square"
      onclick={() => {
        appState.viewMode = appState.viewMode === 'donut' ? 'treemap' : 'donut';
      }}
      title="Toggle View: {appState.viewMode === 'donut' ? 'Switch to Treemap' : 'Switch to Donut Ring'}"
    >
      <span class="material-symbols-outlined">
        {appState.viewMode === 'donut' ? 'grid_view' : 'donut_large'}
      </span>
    </button>
  </div>
</header>

<style>
  .stitch-header {
    height: 44px;
    min-height: 44px;
    background: rgba(16, 19, 25, 0.96);
    backdrop-filter: blur(16px);
    border-bottom: 1px solid rgba(88, 66, 55, 0.35);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    user-select: none;
    z-index: 50;
    gap: 8px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  /* macOS Traffic Lights */
  .traffic-lights {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 2px;
    padding-right: 4px;
  }
  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.2);
    cursor: pointer;
    transition: opacity 0.15s;
  }
  .light:hover {
    opacity: 0.8;
  }
  .light.close {
    background: #FF5F56;
    border-color: #E0443E;
  }
  .light.min {
    background: #FFBD2E;
    border-color: #DEA123;
  }
  .light.max {
    background: #27C93F;
    border-color: #1AAB29;
  }

  .header-divider {
    height: 16px;
    width: 1px;
    background: rgba(88, 66, 55, 0.35);
  }

  /* App Branding */
  .brand-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand-badge-icon {
    position: relative;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: #272A31;
    border: 1px solid rgba(249, 115, 22, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .brand-glyph {
    font-size: 14px;
    color: #F97316;
  }
  .beacon-dot {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #14B8A6;
    box-shadow: 0 0 4px #14B8A6;
    animation: beacon 2.2s infinite ease-in-out;
  }
  @keyframes beacon {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(1.2); }
  }
  .brand-name {
    font-family: var(--font-sans);
    font-weight: 700;
    font-size: 0.875rem;
    color: #F0F6FC;
    letter-spacing: -0.02em;
  }
  .brand-ver {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
    background: #272A31;
    padding: 1px 5px;
    border-radius: 4px;
    border: 1px solid rgba(88, 66, 55, 0.4);
  }

  /* Search Input */
  .search-container {
    position: relative;
    display: flex;
    align-items: center;
    margin-left: 4px;
  }
  .search-icon {
    position: absolute;
    left: 8px;
    font-size: 15px;
    color: #8B949E;
    pointer-events: none;
  }
  .search-input {
    width: 220px;
    height: 28px;
    background: #191C22;
    border: 1px solid rgba(88, 66, 55, 0.35);
    border-radius: 8px;
    padding-left: 28px;
    padding-right: 36px;
    font-family: var(--font-sans);
    font-size: 0.75rem;
    color: #F0F6FC;
    outline: none;
    transition: all 0.15s;
  }
  .search-input::placeholder {
    color: rgba(139, 148, 158, 0.6);
  }
  .search-input:focus {
    border-color: #14B8A6;
    box-shadow: 0 0 0 2px rgba(20, 184, 166, 0.2);
    width: 260px;
  }
  .search-kbd {
    position: absolute;
    right: 8px;
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #8B949E;
    background: #272A31;
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid #32353C;
    pointer-events: none;
  }
  .search-clear {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: #8B949E;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 4px;
  }

  /* Volume Selector Chips Row */
  .volume-selector-cluster {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #0B0E14;
    padding: 3px;
    border-radius: 8px;
    border: 1px solid rgba(88, 66, 55, 0.35);
    max-width: 460px;
    overflow-x: auto;
  }
  .volume-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    border-radius: 6px;
    background: transparent;
    border: 1px solid transparent;
    color: #8B949E;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
    text-align: left;
  }
  .volume-chip:hover {
    background: #1D2026;
    color: #F0F6FC;
  }
  .volume-chip.active {
    background: #272A31;
    color: #F0F6FC;
    border-color: rgba(249, 115, 22, 0.5);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }
  .drive-icon {
    font-size: 18px;
    color: #F97316;
  }
  .volume-chip:not(.active) .drive-icon {
    color: #8B949E;
  }
  .drive-meta {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .drive-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .drive-title {
    font-family: var(--font-sans);
    font-size: 0.6875rem;
    font-weight: 600;
    color: inherit;
  }
  .drive-type {
    font-size: 0.625rem;
    color: #8B949E;
  }
  .gauge-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .gauge-bar {
    width: 48px;
    height: 4px;
    background: #191C22;
    border-radius: 2px;
    overflow: hidden;
  }
  .gauge-fill {
    height: 100%;
    background: #F97316;
    border-radius: 2px;
  }
  .gauge-label {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #8B949E;
  }

  /* Right Action Controls */
  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .btn-purge {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    height: 28px;
    border-radius: 6px;
    background: #1D2026;
    border: 1px solid rgba(88, 66, 55, 0.35);
    color: #F0F6FC;
    font-family: var(--font-sans);
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-purge:hover {
    background: #272A31;
    border-color: #14B8A6;
  }
  .icon-mop {
    font-size: 15px;
    color: #14B8A6;
  }

  .btn-rescan {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    height: 28px;
    border-radius: 6px;
    background: #F97316;
    border: none;
    color: #0E1117;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 0.75rem;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(249, 115, 22, 0.4);
    transition: all 0.15s;
  }
  .btn-rescan:hover:not(:disabled) {
    background: #EA580C;
    transform: translateY(-0.5px);
  }
  .btn-rescan:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .icon-refresh {
    font-size: 16px;
    font-weight: bold;
  }
  .icon-refresh.spinning {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .kbd-pill {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    font-weight: 700;
    background: rgba(0, 0, 0, 0.25);
    padding: 1px 4px;
    border-radius: 3px;
    color: #0E1117;
  }

  .btn-icon-square {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: #1D2026;
    border: 1px solid rgba(88, 66, 55, 0.35);
    color: #8B949E;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-icon-square:hover {
    color: #F0F6FC;
    background: #272A31;
  }
  .metric-flag {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    font-weight: 700;
    color: #8B949E;
  }
  .metric-flag.alloc {
    color: #14B8A6;
  }
</style>
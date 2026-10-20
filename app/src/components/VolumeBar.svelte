<script lang="ts">
  import { appState, selectVolume, rescanCurrentVolume, loadVolumes, exportReport, startCurrentVolumeScan, fetchCacheEntries } from '../lib/store.svelte';
  import { formatBytes, usagePercent } from '../lib/format';
  import { onMount } from 'svelte';

  let searchInput: HTMLInputElement;
  let showDisksDropdown = $state(false);

  const activeVol = $derived(appState.currentVolume || appState.volumes[0]);
  const activePct = $derived(activeVol ? usagePercent(activeVol.used_bytes, activeVol.total_bytes) : 0);

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
      if (e.key === 'Escape') {
        showDisksDropdown = false;
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.volume-dropdown-container')) {
        showDisksDropdown = false;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('click', handleClickOutside);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('click', handleClickOutside);
    };
  });

  async function handlePurgeCacheClick() {
    if (appState.activeFilter === 'cache') {
      appState.activeFilter = 'all';
    } else {
      appState.activeFilter = 'cache';
      if (appState.scanId && appState.cacheEntries.length === 0) {
        await fetchCacheEntries();
      }
    }
  }
</script>

<!-- Stitch Unified TopBar: macOS Titlebar & Navigation (44px) -->
<header class="stitch-header" data-tauri-drag-region>
  <!-- Left: Reserved padding for native macOS Traffic Lights (76px) + Branding + Search -->
  <div class="header-left">
    <!-- App Brand -->
    <div class="brand-group">
      <div class="brand-badge-icon">
        <span class="material-symbols-outlined brand-glyph">radar</span>
        <span class="beacon-dot"></span>
      </div>
      <span class="brand-name">DustMap</span>
      <span class="brand-ver">v0.1.0</span>
    </div>

    <div class="header-divider"></div>

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

  <!-- Center: Volume Selector Dropdown (All Disks in a Clean Dropdown) -->
  <div class="volume-dropdown-container">
    <button
      class="volume-dropdown-trigger"
      class:open={showDisksDropdown}
      onclick={() => showDisksDropdown = !showDisksDropdown}
      title="Select Disk Volume ({appState.volumes.length} mounted) — click to switch"
    >
      <span class="material-symbols-outlined drive-icon">
        {activeVol?.is_removable ? 'usb' : 'hard_drive'}
      </span>
      <div class="drive-meta">
        <div class="drive-row">
          <span class="drive-title">{activeVol?.name || 'Macintosh HD'}</span>
          <span class="drive-type">({activeVol?.is_removable ? 'External' : 'Internal SSD'})</span>
        </div>
        <div class="gauge-row">
          <div class="gauge-bar">
            <div class="gauge-fill" style="width: {activePct}%"></div>
          </div>
          <span class="gauge-label font-mono">
            {formatBytes(activeVol?.used_bytes ?? 0, appState.useBinaryUnits)} / {formatBytes(activeVol?.total_bytes ?? 0, appState.useBinaryUnits)}
          </span>
        </div>
      </div>
      <span class="material-symbols-outlined dropdown-caret">
        {showDisksDropdown ? 'expand_less' : 'unfold_more'}
      </span>
    </button>

    {#if showDisksDropdown}
      <div class="disks-dropdown-menu">
        <div class="dropdown-header font-mono">
          <span>SELECT STORAGE VOLUME ({appState.volumes.length})</span>
        </div>
        {#each appState.volumes as vol}
          {@const isCur = appState.currentVolume?.mount_point === vol.mount_point}
          {@const pct = usagePercent(vol.used_bytes, vol.total_bytes)}
          <button
            class="dropdown-disk-item"
            class:active={isCur}
            onclick={() => {
              selectVolume(vol, false);
              showDisksDropdown = false;
            }}
          >
            <span class="material-symbols-outlined disk-icon">
              {vol.is_removable ? 'usb' : 'hard_drive'}
            </span>
            <div class="disk-info">
              <div class="disk-name-row">
                <span class="disk-name">{vol.name || vol.mount_point}</span>
                <span class="disk-fs font-mono">{vol.fs_type}</span>
              </div>
              <div class="disk-stats-row font-mono">
                <span>{formatBytes(vol.used_bytes, appState.useBinaryUnits)} / {formatBytes(vol.total_bytes, appState.useBinaryUnits)}</span>
                <span class="pct-tag">({pct}%)</span>
              </div>
            </div>
            {#if isCur}
              <span class="material-symbols-outlined checkmark">check</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Right: Actions -->
  <div class="header-right">
    <!-- Purge Cache / Quick Clean -->
    <button
      class="btn-purge"
      class:active={appState.activeFilter === 'cache'}
      onclick={handlePurgeCacheClick}
      title="Filter / highlight purgeable developer caches & build artifacts"
    >
      <span class="material-symbols-outlined icon-mop">mop</span>
      <span>{appState.activeFilter === 'cache' ? 'Showing Caches' : 'Purge Cache'}</span>
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

    <!-- Export Telemetry & Disk Report -->
    <button
      class="btn-export"
      onclick={exportReport}
      title="Export Telemetry & Storage Report to Clipboard"
    >
      <span class="material-symbols-outlined icon-export">ios_share</span>
      <span>Export</span>
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
    padding-left: 78px; /* Reserves clear native space for macOS Traffic Lights */
    padding-right: 12px;
    user-select: none;
    z-index: 50;
    gap: 8px;
    position: relative;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
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
  }
  .search-icon {
    position: absolute;
    left: 8px;
    font-size: 15px;
    color: #8B949E;
    pointer-events: none;
  }
  .search-input {
    width: 200px;
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
    width: 240px;
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

  /* Volume Selector Cluster */
  /* Unified Volume Selector Dropdown */
  .volume-dropdown-container {
    position: relative;
    display: flex;
    align-items: center;
  }
  .volume-dropdown-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 10px;
    border-radius: 8px;
    background: #191C22;
    border: 1px solid rgba(88, 66, 55, 0.4);
    color: #F0F6FC;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
    text-align: left;
  }
  .volume-dropdown-trigger:hover, .volume-dropdown-trigger.open {
    background: #272A31;
    border-color: #F97316;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.4);
  }
  .drive-icon {
    font-size: 17px;
    color: #F97316;
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
    width: 44px;
    height: 4px;
    background: #101319;
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
  .dropdown-caret {
    font-size: 16px;
    color: #8B949E;
    margin-left: 2px;
    transition: transform 0.15s;
  }
  .volume-dropdown-trigger:hover .dropdown-caret, .volume-dropdown-trigger.open .dropdown-caret {
    color: #F97316;
  }
  .disks-dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    width: 290px;
    background: #161B22;
    border: 1px solid rgba(249, 115, 22, 0.4);
    border-radius: 8px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.85);
    padding: 4px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .dropdown-header {
    padding: 6px 8px;
    font-size: 0.5625rem;
    color: #8B949E;
    border-bottom: 1px solid rgba(88, 66, 55, 0.2);
    letter-spacing: 0.05em;
  }
  .dropdown-disk-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    background: transparent;
    border: none;
    color: #8B949E;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s;
  }
  .dropdown-disk-item:hover {
    background: #21262D;
    color: #F0F6FC;
  }
  .dropdown-disk-item.active {
    background: rgba(249, 115, 22, 0.12);
    color: #F97316;
  }
  .disk-icon {
    font-size: 16px;
    color: inherit;
  }
  .disk-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .disk-name-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .disk-name {
    font-family: var(--font-sans);
    font-size: 0.6875rem;
    font-weight: 600;
    color: inherit;
  }
  .disk-fs {
    font-size: 0.5625rem;
    color: #8B949E;
  }
  .disk-stats-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.5625rem;
    color: #8B949E;
  }
  .pct-tag {
    color: #14B8A6;
  }
  .checkmark {
    font-size: 14px;
    color: #14B8A6;
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
  .btn-purge:hover, .btn-purge.active {
    background: #272A31;
    border-color: #14B8A6;
    color: #14B8A6;
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

  .btn-export {
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
  .btn-export:hover {
    background: #272A31;
    border-color: #F97316;
    color: #F97316;
  }
  .icon-export {
    font-size: 15px;
    color: #14B8A6;
  }
</style>
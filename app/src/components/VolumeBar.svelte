<script lang="ts">
  import { appState, doScan, loadVolumes } from '../lib/store.svelte';
  import { formatBytes, usagePercent } from '../lib/format';
  import { onMount } from 'svelte';

  onMount(() => {
    loadVolumes();
  });

  function selectVolume(vol: typeof appState.volumes[0]) {
    appState.currentVolume = vol;
    doScan(vol.mount_point);
  }

  async function rescan() {
    if (appState.currentVolume) {
      await doScan(appState.currentVolume.mount_point);
    }
  }
</script>

<div class="volume-bar" data-tauri-drag-region>
  <div class="traffic-lights-spacer"></div>
  <div class="logo">
    <span class="logo-icon">🗺️</span>
    <span class="logo-text">DustMap</span>
  </div>
  <div class="volumes">
    {#each appState.volumes as vol}
      {@const pct = usagePercent(vol.used_bytes, vol.total_bytes)}
      {@const active = appState.currentVolume?.mount_point === vol.mount_point}
      <button
        class="vol-chip"
        class:active
        onclick={() => selectVolume(vol)}
      >
        <div class="vol-header">
          <span class="vol-name">{vol.name || vol.mount_point}</span>
          <span class="vol-type">{vol.fs_type}</span>
        </div>
        <span class="vol-size">{formatBytes(vol.used_bytes)} / {formatBytes(vol.total_bytes)}</span>
        <div class="vol-bar-track">
          <div class="vol-bar-fill" style="width: {pct}%"></div>
        </div>
      </button>
    {/each}
  </div>
  <button class="rescan-btn" onclick={rescan} disabled={appState.isScanning}>
    {appState.isScanning ? '⟳ Scanning…' : '⟳ Rescan'}
  </button>
</div>

<style>
  .volume-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    background: #161B22;
    border-bottom: 1px solid #21262D;
    min-height: 52px;
    user-select: none;
  }
  .traffic-lights-spacer {
    width: 65px;
    height: 100%;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-right: 0.5rem;
  }
  .logo-icon {
    font-size: 1.25rem;
  }
  .logo-text {
    font-weight: 700;
    font-size: 1rem;
    color: #F0F6FC;
    letter-spacing: -0.02em;
  }
  .volumes {
    display: flex;
    gap: 0.5rem;
    flex: 1;
    overflow-x: auto;
  }
  .vol-chip {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 0.35rem 0.6rem;
    background: #0E1117;
    border: 1px solid #21262D;
    border-radius: 6px;
    cursor: pointer;
    color: #8B949E;
    font-size: 0.75rem;
    min-width: 140px;
    transition: all 0.15s ease;
    text-align: left;
  }
  .vol-chip:hover {
    border-color: #F97316;
    color: #F0F6FC;
  }
  .vol-chip.active {
    border-color: #F97316;
    background: rgba(249, 115, 22, 0.08);
    color: #F0F6FC;
  }
  .vol-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .vol-name {
    font-weight: 600;
    font-size: 0.8125rem;
    color: inherit;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .vol-type {
    font-size: 0.65rem;
    color: #6E7681;
    text-transform: uppercase;
  }
  .vol-size {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.6875rem;
    color: #8B949E;
  }
  .vol-bar-track {
    height: 3px;
    background: #21262D;
    border-radius: 2px;
    overflow: hidden;
  }
  .vol-bar-fill {
    height: 100%;
    background: #F97316;
    border-radius: 2px;
  }
  .rescan-btn {
    padding: 0.375rem 0.875rem;
    background: #F97316;
    color: #0E1117;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    font-size: 0.8125rem;
    cursor: pointer;
    transition: opacity 0.15s;
    white-space: nowrap;
  }
  .rescan-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .rescan-btn:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { appState, loadVolumes } from './lib/store.svelte';
  import VolumeBar from './components/VolumeBar.svelte';
  import Treemap from './components/Treemap.svelte';
  import TopFolderList from './components/TopFolderList.svelte';
  import DeleteModal from './components/DeleteModal.svelte';
  import ScanProgress from './components/ScanProgress.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import PermissionModal from './components/PermissionModal.svelte';
  import SplashScreen from './components/SplashScreen.svelte';
  import CleanupPreview from './components/CleanupPreview.svelte';
  import StartupBanner from './components/StartupBanner.svelte';
  import RemnantPanel from './components/RemnantPanel.svelte';
  import DataPanel from './components/DataPanel.svelte';
  import { doScan, recordScanProgress } from './lib/store.svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { ScanProgressInfo } from './lib/types';
  import Sidebar from './components/Sidebar.svelte';
  import type { View } from './lib/types';

  let view = $state<View>('explorer');

  onMount(() => {
    loadVolumes();
    const unlisten = listen<ScanProgressInfo>('scan-progress', (event) => recordScanProgress(event.payload));
    return () => {
      unlisten.then((stop) => stop());
    };
  });
</script>

<div class="app-shell">
  {#if appState.showSplash}
    <SplashScreen />
  {/if}

  <VolumeBar />

  <div class="subnav">
    <Sidebar view={appState.isScanning ? null : view} onselect={(next) => (view = next)} />
  </div>

  <div class="body">

    <main class="content">
      <StartupBanner onrescan={(root) => doScan(root)} />

      {#if appState.isScanning}
        <ScanProgress />
      {:else if view === 'cleanup'}
        <CleanupPreview />
      {:else if view === 'remnants'}
        <RemnantPanel />
      {:else if view === 'data'}
        <DataPanel />
      {:else}
        <div class="workspace-grid">
          <div class="vis-pane">
            <Treemap />
          </div>
          <div class="list-pane">
            <TopFolderList />
          </div>
        </div>
      {/if}
    </main>
  </div>

  <StatusBar />
  <DeleteModal />
  <PermissionModal />

  {#if appState.toastMessage}
    <div class="toast-notification">
      <span class="material-symbols-outlined toast-icon">check_circle</span>
      <span class="toast-text font-mono">{appState.toastMessage}</span>
    </div>
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: var(--color-surface);
    overflow: hidden;
    position: relative;
  }
  .subnav {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    border-bottom: 1px solid var(--color-outline-variant);
    background: var(--color-surface-container-low);
    flex-shrink: 0;
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .content {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }
  .workspace-grid {
    display: grid;
    grid-template-columns: 60% 40%;
    flex: 1;
    overflow: hidden;
  }
  .vis-pane {
    height: 100%;
    overflow: hidden;
    position: relative;
  }
  .list-pane {
    height: 100%;
    overflow: hidden;
  }

  .toast-notification {
    position: fixed;
    bottom: 38px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--color-surface-container-low);
    border: 1px solid var(--color-secondary);
    border-radius: 8px;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-on-surface);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7);
    z-index: 1000;
    animation: toastIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .toast-icon {
    color: var(--color-secondary);
    font-size: var(--text-xl);
  }
  .toast-text {
    font-size: var(--text-base);
    max-width: 500px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  @keyframes toastIn {
    from { opacity: 0; transform: translate(-50%, 8px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
</style>

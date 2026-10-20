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

  onMount(() => {
    loadVolumes();
  });
</script>

<div class="app-shell">
  {#if appState.showSplash}
    <SplashScreen />
  {/if}

  <VolumeBar />

  {#if appState.isScanning}
    <ScanProgress />
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
    background: #0E1117;
    overflow: hidden;
    position: relative;
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
    background: #161B22;
    border: 1px solid #14B8A6;
    border-radius: 8px;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: #F0F6FC;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.7);
    z-index: 1000;
    animation: toastIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .toast-icon {
    color: #14B8A6;
    font-size: 18px;
  }
  .toast-text {
    font-size: 0.75rem;
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
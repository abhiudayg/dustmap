<script lang="ts">
  import { onMount } from 'svelte';
  import { appState, loadVolumes } from './lib/store.svelte';
  import VolumeBar from './components/VolumeBar.svelte';
  import Treemap from './components/Treemap.svelte';
  import TopFolderList from './components/TopFolderList.svelte';
  import Breadcrumb from './components/Breadcrumb.svelte';
  import DeleteModal from './components/DeleteModal.svelte';
  import ScanProgress from './components/ScanProgress.svelte';
  import StatusBar from './components/StatusBar.svelte';

  onMount(() => {
    loadVolumes();
  });
</script>

<div class="app-shell">
  <VolumeBar />

  {#if appState.isScanning}
    <ScanProgress />
  {:else}
    <Breadcrumb />
    <div class="workspace-grid">
      <div class="treemap-pane">
        <Treemap />
      </div>
      <div class="list-pane">
        <TopFolderList />
      </div>
    </div>
  {/if}

  <StatusBar />
  <DeleteModal />
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: #0E1117;
    overflow: hidden;
  }
  .workspace-grid {
    display: grid;
    grid-template-columns: 65% 35%;
    flex: 1;
    overflow: hidden;
  }
  .treemap-pane {
    height: 100%;
    overflow: hidden;
    position: relative;
  }
  .list-pane {
    height: 100%;
    overflow: hidden;
  }
</style>

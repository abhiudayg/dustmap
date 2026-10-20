<script lang="ts">
  import { appState } from '../lib/store.svelte';
</script>

{#if appState.isScanning}
  <div class="hud">
    <div class="radar">
      <div class="radar-ring outer"></div>
      <div class="radar-ring middle"></div>
      <div class="radar-ring inner"></div>
      <div class="radar-sweep"></div>
      <div class="radar-center">🗺️</div>
    </div>
    <div class="hud-info">
      <p class="scanning-label">Scanning Storage Hierarchy…</p>
      <p class="current-path">{appState.currentVolume?.mount_point ?? 'Volumes'}</p>
      <div class="pulse-bar">
        <div class="pulse-line"></div>
      </div>
    </div>
  </div>
{/if}

<style>
  .hud {
    position: fixed;
    inset: 0;
    background: #0E1117;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 50;
    gap: 1.5rem;
    user-select: none;
  }
  .radar {
    position: relative;
    width: 180px;
    height: 180px;
  }
  .radar-ring {
    position: absolute;
    border-radius: 50%;
    border: 1px solid rgba(249, 115, 22, 0.25);
  }
  .outer {
    inset: 0;
  }
  .middle {
    inset: 25%;
  }
  .inner {
    inset: 45%;
  }
  .radar-sweep {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(from 0deg, transparent 270deg, rgba(249, 115, 22, 0.45) 360deg);
    animation: sweep 1.8s linear infinite;
  }
  @keyframes sweep {
    to {
      transform: rotate(360deg);
    }
  }
  .radar-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 2.2rem;
  }
  .hud-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }
  .scanning-label {
    font-size: 1.15rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .current-path {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    color: #8B949E;
    max-width: 480px;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pulse-bar {
    width: 200px;
    height: 3px;
    background: #21262D;
    border-radius: 2px;
    overflow: hidden;
    position: relative;
    margin-top: 0.5rem;
  }
  .pulse-line {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 40%;
    background: #F97316;
    border-radius: 2px;
    animation: pulse 1.2s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    0% { transform: translateX(0%); }
    100% { transform: translateX(150%); }
  }
</style>

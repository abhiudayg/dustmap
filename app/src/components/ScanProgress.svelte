<script lang="ts">
  import { appState } from '../lib/store.svelte';
</script>

{#if appState.isScanning}
  <div class="hud">
    <div class="telemetry-grid-bg"></div>

    <!-- Concentric Orbital Telemetry Rings -->
    <div class="radar-chamber">
      <div class="orbital-ring ring-3"></div>
      <div class="orbital-ring ring-2"></div>
      <div class="orbital-ring ring-1"></div>
      <div class="crosshair-h"></div>
      <div class="crosshair-v"></div>

      <!-- Rotating Telemetry Sweep -->
      <div class="radar-sweep"></div>

      <!-- Center Logo Core -->
      <div class="radar-core">
        <img src="/favicon.svg" class="core-icon" alt="DustMap" />
      </div>
    </div>

    <!-- HUD Diagnostics Box matching Stitch -->
    <div class="hud-card">
      <div class="hud-top-row">
        <div class="pulse-status">
          <span class="active-dot"></span>
          <span class="status-text">INITIALIZING SECTOR INODE SWEEP…</span>
        </div>
        <span class="scan-tag">APFS ACTIVE</span>
      </div>

      <div class="mount-row">
        <span class="mount-label">TARGET MOUNT:</span>
        <span class="mount-val">{appState.currentVolume?.mount_point ?? '/'}</span>
      </div>

      <div class="progress-track">
        <div class="progress-animated"></div>
      </div>

      <div class="hud-foot">
        <span>Parallel Rayon + WalkDir Scanner</span>
        <span>Trash-First Safety Gate</span>
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
    gap: 1.75rem;
    user-select: none;
    overflow: hidden;
  }
  .telemetry-grid-bg {
    position: absolute;
    inset: 0;
    opacity: 0.04;
    background-image:
      linear-gradient(to right, #4FDCB8 1px, transparent 1px),
      linear-gradient(to bottom, #4FDCB8 1px, transparent 1px);
    background-size: 32px 32px;
  }
  .radar-chamber {
    position: relative;
    width: 220px;
    height: 220px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .orbital-ring {
    position: absolute;
    border-radius: 50%;
  }
  .ring-3 {
    inset: 0;
    border: 1px dashed rgba(79, 220, 184, 0.2);
  }
  .ring-2 {
    inset: 22%;
    border: 1px solid rgba(249, 115, 22, 0.35);
  }
  .ring-1 {
    inset: 42%;
    border: 1px solid rgba(79, 220, 184, 0.3);
  }
  .crosshair-h {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: rgba(167, 139, 125, 0.2);
  }
  .crosshair-v {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(167, 139, 125, 0.2);
  }
  .radar-sweep {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(from 0deg, transparent 270deg, rgba(79, 220, 184, 0.55) 360deg);
    animation: sweep 2.6s linear infinite;
  }
  @keyframes sweep {
    to { transform: rotate(360deg); }
  }
  .radar-core {
    position: relative;
    z-index: 10;
    width: 52px;
    height: 52px;
    border-radius: 14px;
    background: #161B22;
    border: 1px solid rgba(255, 255, 255, 0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.7);
  }
  .core-icon {
    width: 32px;
    height: 32px;
  }
  .hud-card {
    position: relative;
    z-index: 10;
    width: 440px;
    max-width: 90vw;
    background: rgba(16, 19, 25, 0.9);
    border: 1px solid rgba(88, 66, 55, 0.35);
    border-radius: 10px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.6);
  }
  .hud-top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
  }
  .pulse-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .active-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #F97316;
    animation: pulse 1s infinite alternate;
  }
  @keyframes pulse {
    from { opacity: 0.4; }
    to { opacity: 1; }
  }
  .status-text {
    color: #F0F6FC;
    font-weight: 600;
    letter-spacing: 0.04em;
  }
  .scan-tag {
    color: #4FDCB8;
    background: rgba(79, 220, 184, 0.12);
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 600;
  }
  .mount-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: #8B949E;
  }
  .mount-val {
    color: #FFB690;
    font-weight: 600;
  }
  .progress-track {
    width: 100%;
    height: 4px;
    background: #1D2026;
    border-radius: 2px;
    overflow: hidden;
    position: relative;
  }
  .progress-animated {
    height: 100%;
    width: 45%;
    background: linear-gradient(90deg, #F97316, #4FDCB8);
    border-radius: 2px;
    animation: indeterminate 1.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(250%); }
  }
  .hud-foot {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #484F58;
  }
</style>

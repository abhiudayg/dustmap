<script lang="ts">
  import { onMount } from 'svelte';
  import { appState } from '../lib/store.svelte';
  import { APP_VERSION } from '../lib/version';

  let step = $state(0);
  let progressPct = $state(35);
  let exiting = $state(false);

  const statusMessages = [
    'MOUNTING APFS VOLUMES…',
    'INITIALIZING SECTOR ENGINE…',
    'CALIBRATING INODE INDICES…',
    'READY',
  ];

  onMount(() => {
    const timer1 = setTimeout(() => {
      step = 1;
      progressPct = 65;
    }, 400);

    const timer2 = setTimeout(() => {
      step = 2;
      progressPct = 90;
    }, 850);

    const timer3 = setTimeout(() => {
      step = 3;
      progressPct = 100;
      exiting = true;
    }, 1300);

    const timer4 = setTimeout(() => {
      appState.showSplash = false;
    }, 1600);

    return () => {
      clearTimeout(timer1);
      clearTimeout(timer2);
      clearTimeout(timer3);
      clearTimeout(timer4);
    };
  });
</script>

<div class="splash-backdrop" class:fade-out={exiting}>
  <div class="telemetry-grid"></div>

  <!-- Concentric Orbital Telemetry Rings -->
  <div class="orbital-ring ring-outer"></div>
  <div class="orbital-ring ring-middle"></div>
  <div class="orbital-ring ring-inner"></div>

  <div class="splash-card">
    <!-- App Icon Centerpiece -->
    <div class="icon-wrapper">
      <div class="icon-ambient-glow"></div>
      <div class="mac-squircle-icon">
        <div class="specular-bevel"></div>
        <div class="icon-inner-grid">
          <svg class="radar-svg" viewBox="0 0 100 100" fill="none">
            <!-- Outer Calibration Rings -->
            <circle cx="50" cy="50" r="44" style:stroke="var(--color-outline-variant)" stroke-width="0.75" stroke-dasharray="2 3" opacity="0.6" />
            <circle cx="50" cy="50" r="32" style:stroke="var(--color-secondary)" stroke-width="0.5" stroke-opacity="0.3" />
            <circle cx="50" cy="50" r="20" style:stroke="var(--color-primary-container)" stroke-width="0.5" stroke-opacity="0.4" />

            <!-- Crosshairs -->
            <line x1="50" y1="6" x2="50" y2="94" style:stroke="var(--color-outline)" stroke-width="0.5" stroke-opacity="0.4" />
            <line x1="6" y1="50" x2="94" y2="50" style:stroke="var(--color-outline)" stroke-width="0.5" stroke-opacity="0.4" />

            <!-- Sector 1: Primary Large Partition (Orange Amber) -->
            <path d="M 50 50 L 50 18 A 32 32 0 0 1 82 50 Z" style:fill="var(--color-primary-container)" fill-opacity="0.82" style:stroke="var(--color-primary)" stroke-width="0.75" />
            <!-- Sector 2: Secondary APFS Block (Teal Cyan) -->
            <path d="M 50 50 L 82 50 A 32 32 0 0 1 68 76 Z" style:fill="var(--color-secondary)" fill-opacity="0.78" style:stroke="var(--color-secondary)" stroke-width="0.75" />
            <!-- Sector 3: Purgeable Cluster (Tertiary Violet) -->
            <path d="M 50 50 L 68 76 A 32 32 0 0 1 32 78 Z" style:fill="var(--color-tertiary)" fill-opacity="0.6" style:stroke="var(--color-tertiary)" stroke-width="0.75" />
            <!-- Sector 4: System Inodes (Deep Slate) -->
            <path d="M 50 50 L 32 78 A 32 32 0 0 1 24 36 Z" style:fill="var(--color-surface-container-high)" fill-opacity="0.95" style:stroke="var(--color-outline-variant)" stroke-width="0.75" />

            <!-- Active Radar Sweep Cone -->
            <g class="radar-sweep-group">
              <path d="M 50 50 L 50 8 A 42 42 0 0 1 85 24 Z" fill="url(#splashSweepGrad)" />
              <line x1="50" y1="50" x2="85" y2="24" style:stroke="var(--color-secondary)" stroke-width="1.2" stroke-linecap="round" />
            </g>

            <!-- Central Node Core -->
            <circle cx="50" cy="50" r="5" style:fill="var(--color-surface)" style:stroke="var(--color-primary)" stroke-width="1.2" />
            <circle cx="50" cy="50" r="2" style:fill="var(--color-secondary)" />

            <defs>
              <linearGradient id="splashSweepGrad" x1="50" y1="50" x2="85" y2="24" gradientUnits="userSpaceOnUse">
                <stop stop-color="var(--color-secondary)" stop-opacity="0.45" />
                <stop offset="1" stop-color="var(--color-secondary)" stop-opacity="0" />
              </linearGradient>
            </defs>
          </svg>
        </div>
      </div>
      <div class="ssd-badge">
        <span class="ssd-dot"></span>
        <span class="ssd-text">SSD</span>
      </div>
    </div>

    <!-- Wordmark & Subtitle -->
    <div class="brand-group">
      <div class="brand-title-row">
        <h1 class="brand-title">DustMap</h1>
        <span class="version-badge">v{APP_VERSION}</span>
      </div>
      <p class="brand-subtitle">High-Precision macOS Storage Telemetry</p>
    </div>

    <!-- Status & Progress Indicator -->
    <div class="loading-box">
      <div class="loading-header">
        <span class="pulse-dot"></span>
        <span class="status-msg">{statusMessages[step]}</span>
        <span class="pct-num">{progressPct}%</span>
      </div>
      <div class="progress-track">
        <div class="progress-fill" style="width: {progressPct}%"></div>
      </div>
      <div class="meta-row">
        <span>APFS Encrypted • Apple Silicon Native</span>
        <span>Fast Inode Walk</span>
      </div>
    </div>
  </div>
</div>

<style>
  .splash-backdrop {
    position: fixed;
    inset: 0;
    background: var(--color-surface);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
    user-select: none;
    transition: opacity 0.35s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
  }
  .splash-backdrop.fade-out {
    opacity: 0;
    pointer-events: none;
  }
  .telemetry-grid {
    position: absolute;
    inset: 0;
    opacity: 0.04;
    background-image:
      linear-gradient(to right, var(--color-secondary) 1px, transparent 1px),
      linear-gradient(to bottom, var(--color-secondary) 1px, transparent 1px);
    background-size: 32px 32px;
  }
  .orbital-ring {
    position: absolute;
    border-radius: 50%;
    pointer-events: none;
  }
  .ring-outer {
    width: 680px;
    height: 680px;
    border: 1px solid color-mix(in srgb, var(--color-secondary) calc(0.08 * 100%), transparent);
  }
  .ring-middle {
    width: 480px;
    height: 480px;
    border: 1px dashed color-mix(in srgb, var(--color-primary-container) calc(0.12 * 100%), transparent);
  }
  .ring-inner {
    width: 320px;
    height: 320px;
    border: 1px solid color-mix(in srgb, var(--color-secondary) calc(0.12 * 100%), transparent);
  }
  .splash-card {
    position: relative;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 440px;
    width: 90vw;
  }
  .icon-wrapper {
    position: relative;
    margin-bottom: 1.5rem;
  }
  .icon-ambient-glow {
    position: absolute;
    inset: -8px;
    background: linear-gradient(135deg, color-mix(in srgb, var(--color-primary-container) calc(0.4 * 100%), transparent), transparent, color-mix(in srgb, var(--color-secondary) calc(0.4 * 100%), transparent));
    border-radius: 36px;
    filter: blur(14px);
    opacity: 0.8;
  }
  .mac-squircle-icon {
    position: relative;
    width: 132px;
    height: 132px;
    border-radius: 28px;
    background: linear-gradient(180deg, var(--color-surface-container-high) 0%, var(--color-surface-container-low) 50%, var(--color-surface) 100%);
    border: 1px solid color-mix(in srgb, var(--color-on-surface) calc(0.18 * 100%), transparent);
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.7);
    padding: 2px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .specular-bevel {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 50%;
    background: linear-gradient(180deg, color-mix(in srgb, var(--color-on-surface) calc(0.12 * 100%), transparent) 0%, transparent 100%);
    border-top-left-radius: 26px;
    border-top-right-radius: 26px;
    pointer-events: none;
  }
  .icon-inner-grid {
    position: absolute;
    inset: 6px;
    border-radius: 22px;
    border: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.4 * 100%), transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at center, var(--color-surface-container-high) 0%, var(--color-surface) 80%);
    overflow: hidden;
  }
  .radar-svg {
    width: 100%;
    height: 100%;
    padding: 6px;
  }
  .radar-sweep-group {
    transform-origin: 50px 50px;
    animation: radarSweep 3.2s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  @keyframes radarSweep {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  .ssd-badge {
    position: absolute;
    bottom: -4px;
    right: -4px;
    background: var(--color-surface-container-low);
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    padding: 2px 6px;
    display: flex;
    align-items: center;
    gap: 4px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.5);
  }
  .ssd-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-primary-container);
  }
  .ssd-text {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--color-primary);
    letter-spacing: 0.05em;
  }
  .brand-group {
    margin-bottom: 1.5rem;
  }
  .brand-title-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }
  .brand-title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-on-surface);
    letter-spacing: -0.025em;
  }
  .version-badge {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-secondary);
    background: color-mix(in srgb, var(--color-secondary) calc(0.12 * 100%), transparent);
    border: 1px solid color-mix(in srgb, var(--color-secondary) calc(0.35 * 100%), transparent);
    padding: 2px 6px;
    border-radius: 4px;
  }
  .brand-subtitle {
    font-size: var(--text-base);
    color: var(--color-outline);
    margin-top: 0.25rem;
  }
  .loading-box {
    width: 100%;
    background: color-mix(in srgb, var(--color-surface) calc(0.85 * 100%), transparent);
    border: 1px solid var(--color-surface-container-high);
    border-radius: 8px;
    padding: 0.875rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }
  .loading-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-outline);
  }
  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-primary-container);
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.3; transform: scale(0.9); }
    50% { opacity: 1; transform: scale(1.15); }
  }
  .status-msg {
    color: var(--color-on-surface);
    font-weight: 500;
    flex: 1;
    text-align: left;
    margin-left: 0.5rem;
  }
  .pct-num {
    color: var(--color-secondary);
    font-weight: 700;
  }
  .progress-track {
    width: 100%;
    height: 4px;
    background: var(--color-surface-container-high);
    border-radius: 2px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--color-primary-container), var(--color-secondary));
    border-radius: 2px;
    transition: width 0.3s ease-out;
  }
  .meta-row {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-outline-variant);
  }
</style>

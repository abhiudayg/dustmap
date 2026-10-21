<script lang="ts">
  import { onMount } from 'svelte';
  import { appState, cancelCurrentScan } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  onMount(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') cancelCurrentScan();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  const progress = $derived(appState.scanProgress);
  const elapsed = $derived(progress?.elapsed_secs ?? 0);
  const filesPerSecond = $derived(elapsed > 0 && progress ? Math.round(progress.entries_seen / elapsed) : 0);
  const depth = $derived(progress?.current_path ? progress.current_path.split('/').filter(Boolean).length : 0);

  function formatCount(value: number): string {
    return value.toLocaleString('en-US');
  }

  function formatElapsed(secs: number): string {
    const minutes = Math.floor(secs / 60);
    const seconds = (secs % 60).toFixed(2).padStart(5, '0');
    return `${String(minutes).padStart(2, '0')}:${seconds}s`;
  }
</script>

<section class="hud" aria-live="polite" aria-label="Scan progress">
  <header class="hud-top">
    <div class="target">
      <span class="pulse" aria-hidden="true"></span>
      <span class="muted">Scanning:</span>
      <span>{appState.currentVolume?.name ?? 'Volume'}</span>
      <span class="muted">{appState.currentVolume?.mount_point ?? '/'}</span>
    </div>
    <div class="timer">{formatElapsed(elapsed)}</div>
  </header>

  <div class="radar-wrap">
    <div class="sector-pill">
      <span class="dot" aria-hidden="true"></span>
      <span>Sector 0x7F · alloc map active</span>
      <span class="dot" aria-hidden="true"></span>
    </div>
    <div class="radar">
      <div class="grid" aria-hidden="true"></div>
      <div class="frame frame-outer" aria-hidden="true"></div>
      <div class="frame frame-mid" aria-hidden="true"></div>
      <div class="frame frame-inner" aria-hidden="true"></div>
      <span class="deg deg-top">000°</span>
      <span class="deg deg-right">090°</span>
      <span class="deg deg-bottom">180°</span>
      <span class="deg deg-left">270°</span>
      <div class="sweep" aria-hidden="true"></div>
      <span class="blip blip-amber" aria-hidden="true"></span>
      <span class="blip blip-teal" aria-hidden="true"></span>
      <span class="blip blip-amber large" aria-hidden="true"></span>
      <span class="blip blip-white" aria-hidden="true"></span>
      <div class="hub" aria-hidden="true"><span class="hub-dot"></span></div>
    </div>
    <div class="path-bar">
      <span class="material-symbols-outlined" aria-hidden="true">folder_open</span>
      <span class="path" title={progress?.current_path ?? ''}>{progress?.current_path || 'Starting traversal…'}</span>
    </div>
    <p class="meta">depth: {depth} · read-only direct pass</p>
  </div>

  <div class="cards">
    <div class="card">
      <div class="card-label">Files scanned</div>
      <div class="card-value">{formatCount(progress?.entries_seen ?? 0)}</div>
      <div class="card-sub">{formatCount(filesPerSecond)} / sec</div>
    </div>
    <div class="card">
      <div class="card-label">Total size found</div>
      <div class="card-value">{formatBytes(progress?.bytes_accounted ?? 0)}</div>
      <div class="card-sub">{progress?.errors ?? 0} errors</div>
    </div>
    <div class="card">
      <div class="card-label">Scan speed</div>
      <div class="card-value">{formatCount(filesPerSecond)}{' '}<span class="unit">files/sec</span></div>
      <div class="card-sub">Parallel traversal</div>
    </div>
  </div>

  <div class="stage">
    <div class="stage-label">Stage: Directory traversal</div>
    <div class="track"><div class="beam"></div></div>
  </div>

  <div class="stream" role="log" aria-label="Recent paths">
    <div class="stream-title">Real-time path stream</div>
    {#each appState.recentPaths as path (path)}
      <div class="stream-row">{path}</div>
    {:else}
      <div class="stream-row muted">Waiting for entries…</div>
    {/each}
  </div>

  <footer class="hud-bottom">
    <span class="muted">Scan runs read-only. Nothing is deleted while scanning.</span>
    <button class="cancel" onclick={() => cancelCurrentScan()}>Cancel scan <span class="kbd">ESC</span></button>
  </footer>
</section>

<style>
  .hud {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 14px 16px;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    color: var(--color-on-surface);
    background: var(--color-surface);
    font-size: var(--text-base);
  }
  .hud > * {
    flex-shrink: 0;
  }
  .hud-top,
  .hud-bottom {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .target {
    display: flex;
    gap: 8px;
    align-items: center;
    min-width: 0;
  }
  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-primary-container);
    animation: pulse 1.6s ease-in-out infinite;
  }
  .timer {
    padding: 2px 8px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
  }
  .radar-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }
  .sector-pill {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 4px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 999px;
    font-size: var(--text-sm);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-on-surface-variant);
  }
  .dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-primary-container);
  }
  .radar {
    position: relative;
    width: 260px;
    height: 260px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    overflow: hidden;
    background: var(--color-surface-container-lowest);
  }
  .grid {
    position: absolute;
    inset: 0;
    background-image: radial-gradient(var(--color-outline-variant) 1px, transparent 1px);
    background-size: 14px 14px;
    opacity: 0.5;
  }
  .frame {
    position: absolute;
    border: 1px solid var(--color-outline-variant);
    border-radius: 4px;
  }
  .frame-outer { inset: 12%; }
  .frame-mid { inset: 24%; }
  .frame-inner { inset: 34%; }
  .deg {
    position: absolute;
    font-size: var(--text-sm);
    color: var(--color-outline);
  }
  .deg-top,
  .deg-bottom {
    left: 50%;
    transform: translateX(-50%);
  }
  .deg-top { top: 6px; }
  .deg-bottom { bottom: 6px; }
  .deg-left,
  .deg-right {
    top: 50%;
    transform: translateY(-50%);
  }
  .deg-left { left: 6px; }
  .deg-right { right: 6px; }
  .sweep {
    position: absolute;
    inset: 0;
    background: conic-gradient(
      from 0deg at 50% 50%,
      transparent 0deg 270deg,
      color-mix(in srgb, var(--color-primary-container) 30%, transparent) 330deg,
      var(--color-primary-container) 359deg,
      var(--color-on-surface) 360deg
    );
    animation: sweep 3.6s linear infinite;
  }
  .blip {
    position: absolute;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    transform: translate(-50%, -50%);
  }
  .blip.large {
    width: 8px;
    height: 8px;
  }
  .blip-amber { left: 20%; top: 31%; background: var(--color-primary-container); }
  .blip-teal { left: 26%; top: 79%; background: var(--color-secondary); }
  .blip-amber.large { left: 81%; top: 74%; }
  .blip-white { left: 73%; top: 21%; background: var(--color-on-surface); }
  .hub {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 48px;
    height: 48px;
    transform: translate(-50%, -50%);
    display: grid;
    place-items: center;
    border: 1px solid var(--color-primary-container);
    border-radius: 8px;
    background: var(--color-surface-container-lowest);
  }
  .hub-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-primary-container);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--color-primary-container) 25%, transparent);
  }
  .path-bar {
    display: flex;
    gap: 8px;
    align-items: center;
    width: min(560px, 100%);
    padding: 8px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    background: var(--color-surface-container-low);
  }
  .path-bar .material-symbols-outlined {
    font-size: var(--text-base);
    color: var(--color-outline);
  }
  .path {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    font-size: var(--text-sm);
    color: var(--color-outline);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .card {
    padding: 10px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
  }
  .card-label {
    font-size: var(--text-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-outline);
  }
  .card-value {
    font-size: var(--text-2xl);
    font-weight: 600;
    margin: 4px 0;
  }
  .unit {
    font-size: var(--text-base);
    font-weight: 400;
  }
  .card-sub {
    font-size: var(--text-sm);
    color: var(--color-on-surface-variant);
  }
  .stage-label {
    font-size: var(--text-base);
    margin-bottom: 6px;
  }
  .track {
    position: relative;
    height: 6px;
    border-radius: 3px;
    overflow: hidden;
    background: var(--color-surface-container-high);
  }
  .beam {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    width: 30%;
    background: linear-gradient(90deg, transparent, var(--color-primary-container), transparent);
    animation: beam 2.2s ease-in-out infinite;
  }
  .stream {
    flex: 1 1 auto;
    min-height: 80px;
    overflow: auto;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-lowest);
    padding: 8px 10px;
    font-size: var(--text-sm);
  }
  .stream-title {
    color: var(--color-outline);
    margin-bottom: 4px;
  }
  .stream-row {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 2px 0;
  }
  .hud-bottom {
    margin-top: auto;
    border-top: 1px solid var(--color-outline-variant);
    padding-top: 10px;
    font-size: var(--text-base);
  }
  .cancel {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-error);
    background: none;
    color: var(--color-error);
    font: inherit;
    cursor: pointer;
  }
  .kbd {
    padding: 0 4px;
    border: 1px solid var(--color-error);
    border-radius: 3px;
    font-size: var(--text-sm);
  }
  .muted {
    color: var(--color-on-surface-variant);
  }
  @keyframes sweep {
    to { transform: rotate(360deg); }
  }
  @keyframes beam {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(340%); }
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.35; }
  }
</style>

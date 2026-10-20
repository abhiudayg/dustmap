<script lang="ts">
  import { appState, navigateTo, getEntrySize, startCurrentVolumeScan } from '../lib/store.svelte';
  import { computeTreemap, type TreemapRect } from '../lib/treemap';
  import { formatBytes, colorForEntry } from '../lib/format';

  let container = $state<HTMLDivElement | null>(null);
  let width = $state(0);
  let height = $state(0);
  let hoveredId = $state<number | null>(null);

  const nodes = $derived(
    appState.currentChildren
      .map(e => ({
        id: e.id,
        name: e.name,
        value: getEntrySize(e),
        color: colorForEntry(e.name, e.is_hidden),
        isHidden: e.is_hidden,
      }))
      .filter(n => n.value > 0)
  );

  const rects = $derived<TreemapRect[]>(
    width > 0 && height > 0 ? computeTreemap(nodes, width, height) : []
  );

  function handleClick(rect: TreemapRect) {
    const entry = appState.currentChildren.find(e => e.id === rect.node.id);
    if (entry) navigateTo(entry);
  }

  $effect(() => {
    if (!container) return;
    const ro = new ResizeObserver(entries => {
      if (entries.length > 0) {
        width = entries[0].contentRect.width;
        height = entries[0].contentRect.height;
      }
    });
    ro.observe(container);
    return () => ro.disconnect();
  });
</script>

<div class="treemap-container" bind:this={container}>
  {#if rects.length > 0}
    <svg {width} {height} class="treemap-svg">
      {#each rects as rect}
        {@const hovered = hoveredId === rect.node.id}
        <g
          role="button"
          tabindex="0"
          onclick={() => handleClick(rect)}
          onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleClick(rect)}
          onmouseenter={() => (hoveredId = rect.node.id)}
          onmouseleave={() => (hoveredId = null)}
          class="rect-group"
        >
          <rect
            x={rect.x + 1.5}
            y={rect.y + 1.5}
            width={Math.max(rect.width - 3, 0)}
            height={Math.max(rect.height - 3, 0)}
            fill={rect.node.color}
            fill-opacity={rect.node.isHidden ? 0.35 : 0.75}
            stroke={hovered ? '#F97316' : 'rgba(88, 66, 55, 0.4)'}
            stroke-width={hovered ? 2 : 1}
            rx="5"
          />
          {#if rect.width > 55 && rect.height > 28}
            <text
              x={rect.x + 8}
              y={rect.y + 20}
              fill="#F0F6FC"
              font-family="Inter, -apple-system, sans-serif"
              font-size={rect.width > 120 ? 12.5 : 11}
              font-weight="600"
            >
              {rect.node.isHidden ? '🔒 ' : ''}{rect.node.name}
            </text>
          {/if}
          {#if rect.width > 75 && rect.height > 45}
            <text
              x={rect.x + rect.width - 8}
              y={rect.y + rect.height - 8}
              fill="#F0F6FC"
              fill-opacity="0.75"
              font-family="'JetBrains Mono', monospace"
              font-size="10.5"
              text-anchor="end"
            >
              {formatBytes(rect.node.value, appState.useBinaryUnits)}
            </text>
          {/if}
        </g>
      {/each}
    </svg>
  {:else if appState.isScanning}
    <div class="empty-state">
      <span class="scanning-pulse-text">Scanning storage hierarchy…</span>
    </div>
  {:else if appState.summary}
    <div class="empty-state">No files or subdirectories found in this directory.</div>
  {:else}
    <!-- Stitch Sunburst / Orbital Dashboard Overview -->
    {@const vol = appState.currentVolume}
    <div class="dashboard-overview">
      <div class="telemetry-grid-bg"></div>

      <div class="sunburst-ring-outer">
        <div class="sunburst-ring-mid">
          <div class="sunburst-ring-inner">
            <div class="center-metrics">
              <span class="vol-heading">{vol?.name || 'Macintosh HD'}</span>
              <div class="free-space-val">
                {formatBytes(vol?.available_bytes, appState.useBinaryUnits)}
              </div>
              <span class="free-space-label">AVAILABLE STORAGE</span>
              <span class="used-sublabel">
                {formatBytes(vol?.used_bytes, appState.useBinaryUnits)} / {formatBytes(vol?.total_bytes, appState.useBinaryUnits)} used
              </span>
            </div>
          </div>
        </div>
      </div>

      <div class="cta-cluster">
        <button class="btn-start-scan" onclick={startCurrentVolumeScan}>
          <span class="btn-bolt">⚡</span>
          <span>Scan {vol?.name || 'Volume'}</span>
        </button>
        <p class="cta-note">
          Analyzes {vol?.fs_type || 'APFS'} containers with parallel Rayon scanner and squarified treemaps.
        </p>
      </div>
    </div>
  {/if}
</div>

<style>
  .treemap-container {
    flex: 1;
    width: 100%;
    height: 100%;
    background: #0E1117;
    position: relative;
    overflow: hidden;
  }
  .treemap-svg {
    display: block;
    width: 100%;
    height: 100%;
    padding: 4px;
  }
  .rect-group {
    cursor: pointer;
    outline: none;
    transition: filter 0.15s ease;
  }
  .rect-group:hover {
    filter: brightness(1.15);
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #8B949E;
    font-size: 0.9rem;
    padding: 2rem;
    font-family: 'JetBrains Mono', monospace;
  }
  .scanning-pulse-text {
    color: #F97316;
    animation: pulse 1.2s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }

  /* Stitch Dashboard Overview Stage */
  .dashboard-overview {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.75rem;
    user-select: none;
  }
  .telemetry-grid-bg {
    position: absolute;
    inset: 0;
    opacity: 0.03;
    background-image:
      linear-gradient(to right, #4FDCB8 1px, transparent 1px),
      linear-gradient(to bottom, #4FDCB8 1px, transparent 1px);
    background-size: 32px 32px;
  }
  .sunburst-ring-outer {
    position: relative;
    width: 250px;
    height: 250px;
    border-radius: 50%;
    border: 2px dashed rgba(79, 220, 184, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sunburst-ring-mid {
    width: 210px;
    height: 210px;
    border-radius: 50%;
    border: 1px solid rgba(249, 115, 22, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sunburst-ring-inner {
    width: 175px;
    height: 175px;
    border-radius: 50%;
    background: radial-gradient(circle at center, #1E232B 0%, #101319 80%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .center-metrics {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .vol-heading {
    font-family: Inter, -apple-system, sans-serif;
    font-size: 0.75rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .free-space-val {
    font-family: 'JetBrains Mono', monospace;
    font-size: 1.5rem;
    font-weight: 700;
    color: #F97316;
    letter-spacing: -0.02em;
  }
  .free-space-label {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.5625rem;
    font-weight: 600;
    color: #8B949E;
    letter-spacing: 0.06em;
  }
  .used-sublabel {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.625rem;
    color: #4FDCB8;
    margin-top: 3px;
  }
  .cta-cluster {
    position: relative;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }
  .btn-start-scan {
    background: linear-gradient(135deg, #F97316 0%, #EA580C 100%);
    color: #0E1117;
    border: none;
    border-radius: 8px;
    padding: 0.625rem 1.5rem;
    font-family: Inter, -apple-system, sans-serif;
    font-weight: 600;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(249, 115, 22, 0.4);
    transition: all 0.15s ease-out;
  }
  .btn-start-scan:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(249, 115, 22, 0.55);
  }
  .btn-bolt {
    font-size: 1rem;
  }
  .cta-note {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.6875rem;
    color: #8B949E;
    max-width: 360px;
    text-align: center;
  }
</style>

<script lang="ts">
  import { appState, navigateTo } from '../lib/store.svelte';
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
        value: e.total_logical ?? e.logical_bytes ?? 0,
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
            x={rect.x + 1}
            y={rect.y + 1}
            width={Math.max(rect.width - 2, 0)}
            height={Math.max(rect.height - 2, 0)}
            fill={rect.node.color}
            fill-opacity={rect.node.isHidden ? 0.45 : 0.75}
            stroke={hovered ? '#F97316' : '#0E1117'}
            stroke-width={hovered ? 2 : 1}
            rx="4"
          />
          {#if rect.width > 55 && rect.height > 28}
            <text
              x={rect.x + 8}
              y={rect.y + 20}
              fill="#F0F6FC"
              font-family="Inter, -apple-system, sans-serif"
              font-size={rect.width > 120 ? 13 : 11}
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
              {formatBytes(rect.node.value)}
            </text>
          {/if}
        </g>
      {/each}
    </svg>
  {:else if appState.isScanning}
    <div class="empty-state">Scanning directory tree…</div>
  {:else if appState.summary}
    <div class="empty-state">No files or subdirectories to display in this folder.</div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">🗺️</div>
      <div class="empty-title">Ready to Scan</div>
      <div class="empty-sub">Select an internal or external drive from the top bar to inspect disk usage.</div>
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
  }
  .rect-group {
    cursor: pointer;
    outline: none;
    transition: transform 0.1s ease;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #8B949E;
    font-size: 0.9rem;
    gap: 0.5rem;
    padding: 2rem;
    text-align: center;
  }
  .empty-icon {
    font-size: 2.5rem;
  }
  .empty-title {
    font-size: 1.1rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .empty-sub {
    font-size: 0.85rem;
    max-width: 380px;
    line-height: 1.4;
  }
</style>

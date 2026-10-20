<script lang="ts">
  import { appState, navigateTo, startCurrentVolumeScan, getEntrySize, goBack, goForward, fetchCacheEntries } from '../lib/store.svelte';
  import { computeTreemap, type TreemapRect } from '../lib/treemap';
  import { formatBytes, colorForEntry, usagePercent } from '../lib/format';
  import type { IpcEntry } from '../lib/types';

  let container: HTMLDivElement;
  let width = $state(0);
  let height = $state(0);
  let hoveredId = $state<number | null>(null);

  // Fetch cache entries when cache filter is activated
  $effect(() => {
    if (appState.activeFilter === 'cache' && appState.scanId && appState.cacheEntries.length === 0) {
      fetchCacheEntries();
    }
  });

  // Filtered children based on active filter and search query
  const filteredChildren = $derived.by(() => {
    let list: IpcEntry[];
    if (appState.activeFilter === 'cache' && appState.cacheEntries.length > 0) {
      list = [...appState.cacheEntries];
    } else {
      list = [...appState.currentChildren];
    }

    if (appState.searchQuery.trim()) {
      const q = appState.searchQuery.toLowerCase();
      list = list.filter(e => e.name.toLowerCase().includes(q) || (e.path && e.path.toLowerCase().includes(q)));
    }

    if (appState.activeFilter === 'cache' && appState.cacheEntries.length === 0) {
      list = list.filter(e => {
        const n = e.name.toLowerCase();
        return n.includes('cache') || n.includes('tmp') || n.includes('deriveddata') || n.includes('log');
      });
    } else if (appState.activeFilter === 'apps') {
      list = list.filter(e => e.name.toLowerCase().endsWith('.app') || e.name === 'Applications');
    }

    return list;
  });

  // Treemap nodes for treemap view mode
  const treemapNodes = $derived(
    filteredChildren
      .filter(e => getEntrySize(e) > 0)
      .map(e => ({
        id: e.id,
        name: e.name,
        value: getEntrySize(e),
        color: colorForEntry(e.name, e.isHidden),
        isHidden: e.isHidden,
      }))
  );

  const rects = $derived(
    width > 0 && height > 0 ? computeTreemap(treemapNodes, width, height) : []
  );

  function handleClick(rect: TreemapRect) {
    const entry = filteredChildren.find(e => e.id === rect.node.id) || appState.currentChildren.find(e => e.id === rect.node.id);
    if (entry) navigateTo(entry);
  }

  // --- Real Dynamic Donut Calculations ---
  const CIRCUMFERENCE = 578.05; // 2 * pi * 92
  const topCategories = $derived.by(() => {
    const list = appState.activeFilter === 'cache' && appState.cacheEntries.length > 0 
      ? appState.cacheEntries 
      : appState.currentChildren;
    const sorted = [...list]
      .sort((a, b) => getEntrySize(b) - getEntrySize(a));
    const totalAccounted = sorted.reduce((sum, e) => sum + getEntrySize(e), 0) || 1;

    const colors = ['#F97316', '#14B8A6', '#F59E0B', '#8B5CF6', '#F43F5E'];
    const top5 = sorted.slice(0, 5).map((entry, idx) => {
      const bytes = getEntrySize(entry);
      const ratio = bytes / totalAccounted;
      const arcLen = ratio * CIRCUMFERENCE;
      return {
        entry,
        name: entry.name,
        bytes,
        ratio,
        pct: Math.round(ratio * 100),
        arcLen: Math.max(arcLen - 4, 2),
        color: colors[idx % colors.length],
        itemCount: entry.child_count ?? 1,
      };
    });

    let currentOffset = 0;
    return top5.map(cat => {
      const offset = currentOffset;
      currentOffset -= (cat.arcLen + 4);
      return { ...cat, offset };
    });
  });

  const vol = $derived(appState.currentVolume);
  const totalVolumeBytes = $derived(vol?.total_bytes ?? 1);
  const usedVolumeBytes = $derived(vol?.used_bytes ?? 0);
  const freeVolumeBytes = $derived(vol?.available_bytes ?? 0);
  const usedPct = $derived(usagePercent(usedVolumeBytes, totalVolumeBytes));

  $effect(() => {
    if (!container) return;
    const ro = new ResizeObserver(entries => {
      const { width: w, height: h } = entries[0].contentRect;
      width = w;
      height = h;
    });
    ro.observe(container);
    return () => ro.disconnect();
  });
</script>

<div class="vis-container" bind:this={container}>
  <!-- Sub-Header: Breadcrumb & Filter Switchers -->
  <div class="sub-header">
    <div class="breadcrumb-trail">
      <!-- Back / Forward navigation controls -->
      <div class="nav-arrows">
        <button
          class="nav-btn"
          disabled={appState.historyIdx <= 0 && appState.breadcrumb.length <= 1}
          onclick={goBack}
          title="Go back"
        >
          <span class="material-symbols-outlined">arrow_back</span>
        </button>
        <button
          class="nav-btn"
          disabled={appState.historyIdx >= appState.history.length - 1}
          onclick={goForward}
          title="Go forward"
        >
          <span class="material-symbols-outlined">arrow_forward</span>
        </button>
      </div>

      <span class="material-symbols-outlined icon-mode">
        {appState.viewMode === 'donut' ? 'pie_chart' : 'grid_view'}
      </span>
      <span class="crumb-root">{vol?.name || 'Macintosh HD'}</span>
      <span class="crumb-sep">›</span>
      <span class="crumb-current">{appState.breadcrumb.at(-1)?.name || 'Root (/)'}</span>
      <span class="crumb-sep">›</span>
      <span class="crumb-badge">Breakdown</span>
    </div>

    <!-- Filter segment pills -->
    <div class="controls-cluster">
      <div class="filter-pills">
        <button
          class="pill"
          class:active={appState.activeFilter === 'all'}
          onclick={() => appState.activeFilter = 'all'}
        >
          All Files
        </button>
        <button
          class="pill"
          class:active={appState.activeFilter === 'cache'}
          onclick={() => appState.activeFilter = 'cache'}
        >
          Cache & Logs
        </button>
        <button
          class="pill"
          class:active={appState.activeFilter === 'apps'}
          onclick={() => appState.activeFilter = 'apps'}
        >
          Apps & Binaries
        </button>
      </div>

      <!-- View Switcher -->
      <div class="view-switcher">
        <button
          class="view-btn"
          class:active={appState.viewMode === 'donut'}
          onclick={() => appState.viewMode = 'donut'}
          title="Donut / Sector View"
        >
          <span class="material-symbols-outlined">donut_large</span>
        </button>
        <button
          class="view-btn"
          class:active={appState.viewMode === 'treemap'}
          onclick={() => appState.viewMode = 'treemap'}
          title="Treemap View"
        >
          <span class="material-symbols-outlined">grid_view</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Main Visual Area -->
  <div class="visual-viewport">
    {#if appState.isScanning}
      <div class="empty-state">
        <span class="material-symbols-outlined spin-icon">radar</span>
        <span class="scanning-label">Scanning storage hierarchy…</span>
        <span class="scanning-sub">Aggregating filesystem blocks across APFS container</span>
      </div>

    {:else if !appState.scanId}
      <!-- Stitch Sunburst / Orbital Dashboard Overview (Before Scan) -->
      <div class="orbital-overview">
        <div class="telemetry-grid-bg"></div>

        <div class="sunburst-ring-outer">
          <div class="sunburst-ring-mid">
            <div class="sunburst-ring-inner">
              <div class="center-metrics">
                <span class="vol-heading">{vol?.name || 'Macintosh HD'}</span>
                <div class="free-space-val font-mono">
                  {formatBytes(freeVolumeBytes, appState.useBinaryUnits)}
                </div>
                <span class="free-space-label font-mono">AVAILABLE STORAGE</span>
                <span class="used-sublabel font-mono">
                  {formatBytes(usedVolumeBytes, appState.useBinaryUnits)} / {formatBytes(totalVolumeBytes, appState.useBinaryUnits)} used ({usedPct}%)
                </span>
              </div>
            </div>
          </div>
        </div>

        <div class="cta-cluster">
          <button class="btn-start-scan" onclick={startCurrentVolumeScan}>
            <span class="material-symbols-outlined">bolt</span>
            <span>Scan {vol?.name || 'Volume'}</span>
          </button>
          <p class="cta-note font-mono">
            Analyzes {vol?.fs_type || 'APFS'} containers with parallel Rayon scanner and squarified sector maps.
          </p>
        </div>
      </div>

    {:else if appState.viewMode === 'donut'}
      <!-- Stitch Donut Ring Visualizer Screen 1 -->
      <div class="donut-stage">
        <!-- Ambient depth glow behind chart -->
        <div class="ambient-glow"></div>

        <div class="donut-chart-wrapper">
          <svg class="donut-svg" viewBox="0 0 240 240">
            <defs>
              <linearGradient id="grad-apps" x1="0%" x2="100%" y1="0%" y2="100%">
                <stop offset="0%" stop-color="#F97316" />
                <stop offset="100%" stop-color="#EA580C" />
              </linearGradient>
              <linearGradient id="grad-users" x1="0%" x2="100%" y1="0%" y2="100%">
                <stop offset="0%" stop-color="#14B8A6" />
                <stop offset="100%" stop-color="#0D9488" />
              </linearGradient>
              <linearGradient id="grad-lib" x1="0%" x2="100%" y1="0%" y2="100%">
                <stop offset="0%" stop-color="#F59E0B" />
                <stop offset="100%" stop-color="#D97706" />
              </linearGradient>
              <linearGradient id="grad-sys" x1="0%" x2="100%" y1="0%" y2="100%">
                <stop offset="0%" stop-color="#8B5CF6" />
                <stop offset="100%" stop-color="#7C3AED" />
              </linearGradient>
              <linearGradient id="grad-cache" x1="0%" x2="100%" y1="0%" y2="100%">
                <stop offset="0%" stop-color="#F43F5E" />
                <stop offset="100%" stop-color="#E11D48" />
              </linearGradient>
            </defs>

            <!-- Donut Background Track / Free Space -->
            <circle
              cx="120"
              cy="120"
              r="92"
              fill="none"
              stroke="#21262D"
              stroke-width="26"
            />

            <!-- Dynamic Category Arcs -->
            {#each topCategories as cat}
              <circle
                class="donut-segment"
                role="button"
                tabindex="0"
                cx="120"
                cy="120"
                r="92"
                fill="none"
                stroke={cat.color}
                stroke-width="26"
                stroke-dasharray="{cat.arcLen} {CIRCUMFERENCE - cat.arcLen}"
                stroke-dashoffset={cat.offset}
                stroke-linecap="round"
                onclick={() => navigateTo(cat.entry)}
                onkeydown={(e) => e.key === 'Enter' && navigateTo(cat.entry)}
              >
                <title>{cat.name}: {formatBytes(cat.bytes, appState.useBinaryUnits)} ({cat.pct}%)</title>
              </circle>
            {/each}
          </svg>

          <!-- Center Readout Badge inside donut hole -->
          <div class="donut-center-readout">
            <span class="readout-apfs font-mono">{vol?.fs_type || 'APFS'} VOLUME</span>
            <span class="readout-size font-mono">
              {formatBytes(usedVolumeBytes, appState.useBinaryUnits)}
            </span>
            <span class="readout-used font-mono">
              USED OF {formatBytes(totalVolumeBytes, appState.useBinaryUnits)} ({usedPct}%)
            </span>
            <div class="readout-free font-mono">
              <span class="free-dot"></span>
              <span>{formatBytes(freeVolumeBytes, appState.useBinaryUnits)} Free Space</span>
            </div>
          </div>
        </div>

        <!-- 5-Card Interactive Legend Grid -->
        <div class="legend-grid">
          {#each topCategories as cat}
            <div
              class="legend-card"
              role="button"
              tabindex="0"
              onclick={() => navigateTo(cat.entry)}
              onkeydown={(e) => e.key === 'Enter' && navigateTo(cat.entry)}
            >
              <div class="legend-card-header">
                <span class="legend-dot" style="background: {cat.color}; box-shadow: 0 0 6px {cat.color}80"></span>
                <span class="legend-title">{cat.name}</span>
              </div>
              <span class="legend-bytes font-mono">
                {formatBytes(cat.bytes, appState.useBinaryUnits)}
              </span>
              <div class="legend-footer font-mono">
                <span>{cat.pct}%</span>
                <span>{cat.itemCount.toLocaleString()} items</span>
              </div>
            </div>
          {/each}
        </div>
      </div>

    {:else}
      <!-- Squarified Treemap View Mode -->
      <div class="treemap-viewport">
        <svg {width} {height} class="treemap-svg">
          {#each rects as rect}
            {@const hovered = hoveredId === rect.node.id}
            <g
              role="button"
              tabindex="0"
              class="rect-group"
              onclick={() => handleClick(rect)}
              onkeydown={(e) => e.key === 'Enter' && handleClick(rect)}
              onmouseenter={() => hoveredId = rect.node.id}
              onmouseleave={() => hoveredId = null}
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
                  font-family="var(--font-sans)"
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
                  font-family="var(--font-mono)"
                  font-size="10.5"
                  text-anchor="end"
                >
                  {formatBytes(rect.node.value, appState.useBinaryUnits)}
                </text>
              {/if}
            </g>
          {/each}
        </svg>
      </div>
    {/if}
  </div>

  <!-- Scope Info Bar Below Visualization (Stitch Screen 1) -->
  <div class="quick-action-bar">
    <div class="action-bar-left">
      <span class="scope-tag font-mono">
        Active Scope: {appState.breadcrumb.at(-1)?.name || '/'}
      </span>
    </div>
    <div class="action-bar-right">
      <span class="scope-meta font-mono">
        {treemapNodes.length} items rendered
      </span>
    </div>
  </div>
</div>

<style>
  .vis-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #101319;
    border-right: 1px solid rgba(88, 66, 55, 0.35);
    overflow: hidden;
    user-select: none;
  }

  /* Sub-header with breadcrumb & view filters */
  .sub-header {
    height: 36px;
    min-height: 36px;
    background: #0B0E14;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
  }
  .breadcrumb-trail {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: #8B949E;
  }
  .nav-arrows {
    display: flex;
    align-items: center;
    gap: 3px;
    margin-right: 4px;
  }
  .nav-btn {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #191C22;
    border: 1px solid rgba(88, 66, 55, 0.35);
    border-radius: 4px;
    color: #8B949E;
    cursor: pointer;
    transition: all 0.12s;
    padding: 0;
  }
  .nav-btn:hover:not(:disabled) {
    background: #272A31;
    color: #F0F6FC;
    border-color: #F97316;
  }
  .nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .nav-btn span {
    font-size: 14px;
  }
  .icon-mode {
    font-size: 16px;
    color: #F97316;
  }
  .crumb-root {
    color: #F0F6FC;
    font-weight: 500;
  }
  .crumb-sep {
    color: #484F58;
  }
  .crumb-current {
    color: #F0F6FC;
    font-weight: 600;
  }
  .crumb-badge {
    color: #14B8A6;
    font-weight: 500;
  }

  .controls-cluster {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .filter-pills {
    display: flex;
    align-items: center;
    background: #272A31;
    padding: 2px;
    border-radius: 6px;
    border: 1px solid rgba(88, 66, 55, 0.35);
  }
  .pill {
    padding: 2px 8px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.12s;
  }
  .pill.active {
    background: #32353C;
    color: #F0F6FC;
    font-weight: 600;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }
  .view-switcher {
    display: flex;
    align-items: center;
    background: #272A31;
    padding: 2px;
    border-radius: 6px;
    border: 1px solid rgba(88, 66, 55, 0.35);
  }
  .view-btn {
    width: 24px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    background: transparent;
    border: none;
    color: #8B949E;
    cursor: pointer;
    transition: all 0.12s;
  }
  .view-btn.active {
    background: #32353C;
    color: #F97316;
  }
  .view-btn span {
    font-size: 14px;
  }

  /* Main Viewport */
  .visual-viewport {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  /* Empty / Scanning State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: #8B949E;
  }
  .spin-icon {
    font-size: 32px;
    color: #F97316;
    animation: spin 2s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .scanning-label {
    font-family: var(--font-sans);
    font-size: 0.875rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .scanning-sub {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: #8B949E;
  }

  /* Stitch Donut Stage (Screen 1) */
  .donut-stage {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    gap: 1.25rem;
  }
  .ambient-glow {
    position: absolute;
    width: 320px;
    height: 320px;
    border-radius: 50%;
    background: rgba(249, 115, 22, 0.08);
    filter: blur(48px);
    pointer-events: none;
  }
  .donut-chart-wrapper {
    position: relative;
    width: 250px;
    height: 250px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .donut-svg {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
    filter: drop-shadow(0 8px 24px rgba(0, 0, 0, 0.6));
  }
  .donut-segment {
    cursor: pointer;
    transition: opacity 0.15s, stroke-width 0.15s;
  }
  .donut-segment:hover {
    opacity: 0.88;
    stroke-width: 28;
  }
  .donut-center-readout {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    pointer-events: none;
  }
  .readout-apfs {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #8B949E;
    letter-spacing: 0.06em;
  }
  .readout-size {
    font-family: var(--font-mono);
    font-size: 1.75rem;
    font-weight: 700;
    color: #F0F6FC;
    letter-spacing: -0.02em;
    margin-top: 1px;
  }
  .readout-used {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    font-weight: 700;
    color: #F97316;
    margin-top: 2px;
  }
  .readout-free {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #14B8A6;
    margin-top: 3px;
  }
  .free-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #14B8A6;
  }

  /* 5-Card Interactive Legend Grid (Stitch Screen 1) */
  .legend-grid {
    width: 100%;
    max-width: 560px;
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
  }
  .legend-card {
    background: rgba(39, 42, 49, 0.6);
    border: 1px solid rgba(88, 66, 55, 0.3);
    border-radius: 8px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    cursor: pointer;
    transition: all 0.15s;
    outline: none;
  }
  .legend-card:hover {
    border-color: #F97316;
    background: #272A31;
    transform: translateY(-1px);
  }
  .legend-card-header {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .legend-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .legend-title {
    font-family: var(--font-sans);
    font-size: 0.6875rem;
    font-weight: 600;
    color: #F0F6FC;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .legend-bytes {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 700;
    color: #F0F6FC;
    margin-top: 2px;
  }
  .legend-footer {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #8B949E;
  }

  /* Treemap Viewport */
  .treemap-viewport {
    width: 100%;
    height: 100%;
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
    transition: filter 0.15s;
  }
  .rect-group:hover {
    filter: brightness(1.15);
  }

  /* Orbital Overview (Before Scan) */
  .orbital-overview {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.5rem;
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
    width: 240px;
    height: 240px;
    border-radius: 50%;
    border: 2px dashed rgba(79, 220, 184, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sunburst-ring-mid {
    width: 200px;
    height: 200px;
    border-radius: 50%;
    border: 1px solid rgba(249, 115, 22, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .sunburst-ring-inner {
    width: 165px;
    height: 165px;
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
    font-family: var(--font-sans);
    font-size: 0.75rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .free-space-val {
    font-family: var(--font-mono);
    font-size: 1.5rem;
    font-weight: 700;
    color: #F97316;
  }
  .free-space-label {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    font-weight: 600;
    color: #8B949E;
    letter-spacing: 0.06em;
  }
  .used-sublabel {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
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
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 0.8125rem;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(249, 115, 22, 0.4);
    transition: all 0.15s;
  }
  .btn-start-scan:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(249, 115, 22, 0.55);
  }
  .cta-note {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
    max-width: 320px;
    text-align: center;
  }

  /* Quick Action Bar Below Visualization (Stitch Screen 1) */
  .quick-action-bar {
    height: 28px;
    min-height: 28px;
    background: #0B0E14;
    border-top: 1px solid rgba(88, 66, 55, 0.25);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
  }
  .scope-tag {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
  }
  .scope-meta {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #14B8A6;
  }
</style>
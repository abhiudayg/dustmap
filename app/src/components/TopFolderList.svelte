<script lang="ts">
  import { appState, navigateTo, setDeleteTarget, getEntrySize, revealPath, fetchCacheEntries } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';
  import type { IpcEntry } from '../lib/types';

  let sortAsc = $state(false);

  // Automatically fetch disk-wide cache entries if user switches to cache filter
  $effect(() => {
    if (appState.activeFilter === 'cache' && appState.scanId && appState.cacheEntries.length === 0) {
      fetchCacheEntries();
    }
  });

  // Compute category icon, color, and badge for any filesystem entry
  function getCategoryMeta(entry: IpcEntry) {
    const name = entry.name.toLowerCase();
    const isDir = entry.kind === 'Directory';

    if (entry.isHidden) {
      return { icon: 'visibility_off', color: '#8B949E', badge: 'HIDDEN' };
    }
    if (name.includes('cache') || name === '.cache' || name.includes('deriveddata')) {
      return { icon: 'local_fire_department', color: '#F43F5E', badge: 'PURGEABLE' };
    }
    if (name === 'system' || name === 'private' || name === 'bin' || name === 'sbin' || name.startsWith('root')) {
      return { icon: 'lock', color: '#8B949E', badge: 'SYSTEM' };
    }
    if (name === '.docker' || name === 'docker' || name.includes('container')) {
      return { icon: 'deployed_code', color: '#14B8A6', badge: 'CONTAINER' };
    }
    if (name.endsWith('.app') || name === 'applications') {
      return { icon: 'apps', color: '#F97316', badge: 'APPLICATION' };
    }
    if (name.includes('coresimulator') || name.includes('developer') || name === 'xcode') {
      return { icon: 'devices', color: '#F59E0B', badge: 'DEVELOPER' };
    }
    if (name === 'downloads') {
      return { icon: 'download', color: '#14B8A6', badge: 'DOWNLOADS' };
    }
    if (name === 'library') {
      return { icon: 'folder_special', color: '#8B5CF6', badge: 'LIBRARY' };
    }
    if (name === 'users') {
      return { icon: 'group', color: '#14B8A6', badge: 'USERS' };
    }
    if (name.includes('log')) {
      return { icon: 'list_alt', color: '#8B949E', badge: 'LOGS' };
    }
    return { icon: isDir ? 'folder' : 'draft', color: '#14B8A6', badge: isDir ? 'FOLDER' : 'FILE' };
  }

  // Filter and sort items
  const sorted = $derived.by(() => {
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

    list.sort((a, b) => {
      const sa = getEntrySize(a);
      const sb = getEntrySize(b);
      return sortAsc ? sa - sb : sb - sa;
    });

    return list.slice(0, 30);
  });

  const maxSize = $derived.by(() => {
    return sorted[0] ? getEntrySize(sorted[0]) : 1;
  });

  function getEntryFullPath(entry: IpcEntry): string {
    if (entry.path) return entry.path;
    const root = appState.summary?.root ?? '/';
    return root === '/' ? `/${entry.name}` : `${root}/${entry.name}`;
  }
</script>

<!-- Stitch Right Panel (40%): Top Space Consumers Inspector -->
<aside class="consumers-panel">
  <!-- Panel Header -->
  <div class="panel-header">
    <div class="header-left">
      <span class="panel-title">Top Space Consumers</span>
      <span class="items-count-badge font-mono">
        {sorted.length} {sorted.length === 1 ? 'item' : 'items'}
      </span>
    </div>

    <!-- Sort Action Dropdown -->
    <div class="header-right">
      <button
        class="sort-dropdown-btn font-mono"
        onclick={() => sortAsc = !sortAsc}
        title="Toggle sort direction"
      >
        <span class="material-symbols-outlined icon-sort">sort</span>
        <span>Size: {sortAsc ? 'Ascending' : 'Descending'}</span>
        <span class="material-symbols-outlined icon-chevron">
          {sortAsc ? 'arrow_drop_up' : 'arrow_drop_down'}
        </span>
      </button>
    </div>
  </div>

  <!-- High-Density High-Precision Table Header (24px) -->
  <div class="table-header font-mono">
    <div class="col-name">PATH / ITEM</div>
    <div class="col-size">SIZE</div>
    <div class="col-actions">ACTION</div>
  </div>

  <!-- Scrollable High-Density Table View (Rows matching Stitch Screen 1) -->
  <div class="table-body">
    {#if sorted.length === 0}
      <div class="empty-list font-mono">
        {#if appState.isScanning}
          <span>Indexing directory entries…</span>
        {:else if !appState.scanId}
          <span>Ready to scan storage volume</span>
        {:else}
          <span>No matching items found</span>
        {/if}
      </div>
    {:else}
      {#each sorted as entry}
        {@const size = getEntrySize(entry)}
        {@const pct = Math.round((size / maxSize) * 100)}
        {@const meta = getCategoryMeta(entry)}
        {@const fullPath = getEntryFullPath(entry)}
        {@const isPurgeable = meta.badge === 'PURGEABLE'}

        <div
          class="table-row"
          class:row-purgeable={isPurgeable}
          role="button"
          tabindex="0"
          onclick={() => navigateTo(entry)}
          onkeydown={(e) => e.key === 'Enter' && navigateTo(entry)}
        >
          <!-- Left: Icon Box + Name + Badge + Truncated Path -->
          <div class="col-name cell-meta">
            <div class="icon-box" style="border-color: {meta.color}40; background: {meta.color}15">
              <span class="material-symbols-outlined row-glyph" style="color: {meta.color}">
                {meta.icon}
              </span>
            </div>

            <div class="name-cluster">
              <div class="name-row">
                <span class="item-name" title={entry.name}>{entry.name}</span>
                <span class="category-pill font-mono" style="color: {meta.color}; border-color: {meta.color}40; background: {meta.color}18">
                  {meta.badge}
                </span>
              </div>
              <span class="item-path font-mono" title={fullPath}>
                {fullPath}
              </span>
            </div>
          </div>

          <!-- Center: Size Value + Proportional Bar Gauge -->
          <div class="col-size cell-size">
            <span class="size-val font-mono" style="color: {size > 30000000000 ? '#F97316' : (size > 10000000000 ? '#F59E0B' : '#14B8A6')}">
              {formatBytes(size, appState.useBinaryUnits)}
            </span>
            <div class="mini-bar-track">
              <div
                class="mini-bar-fill"
                style="width: {pct}%; background: {meta.color}"
              ></div>
            </div>
          </div>

          <!-- Right: Interactive Actions (Reveal in Finder, Move to Trash, Purge) -->
          <div class="col-actions cell-actions">
            <button
              class="btn-action-icon"
              onclick={(e) => {
                e.stopPropagation();
                revealPath(fullPath);
              }}
              title="Reveal in Finder"
            >
              <span class="material-symbols-outlined">visibility</span>
            </button>

            {#if isPurgeable}
              <button
                class="btn-purge-pill font-mono"
                onclick={(e) => {
                  e.stopPropagation();
                  setDeleteTarget(fullPath, entry);
                }}
                title="Purge / Clean this cache"
              >
                Purge
              </button>
            {:else}
              <button
                class="btn-action-icon delete"
                onclick={(e) => {
                  e.stopPropagation();
                  setDeleteTarget(fullPath, entry);
                }}
                title="Move to Trash"
              >
                <span class="material-symbols-outlined">delete</span>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Right Panel Bottom Footer info (Stitch Screen 1) -->
  <div class="panel-footer font-mono">
    <span class="footer-count">
      Showing Top {sorted.length} of {appState.currentChildren.length.toLocaleString()} items
    </span>
    <span
      class="footer-cta"
      role="button"
      tabindex="0"
      onclick={() => {
        appState.activeFilter = 'all';
        appState.searchQuery = '';
      }}
      onkeydown={(e) => e.key === 'Enter' && (appState.activeFilter = 'all')}
    >
      View All Consumers ›
    </span>
  </div>
</aside>

<style>
  .consumers-panel {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #0B0E14;
    overflow: hidden;
    user-select: none;
  }

  /* Header 36px */
  .panel-header {
    height: 36px;
    min-height: 36px;
    background: #101319;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .panel-title {
    font-family: var(--font-sans);
    font-size: 0.8125rem;
    font-weight: 600;
    color: #F0F6FC;
  }
  .items-count-badge {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    padding: 1px 6px;
    border-radius: 4px;
    background: #272A31;
    color: #8B949E;
    border: 1px solid rgba(88, 66, 55, 0.3);
  }

  /* Sort dropdown button */
  .sort-dropdown-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #191C22;
    border: 1px solid rgba(88, 66, 55, 0.35);
    color: #8B949E;
    border-radius: 6px;
    padding: 2px 8px;
    font-family: var(--font-mono);
    font-size: 0.625rem;
    cursor: pointer;
    transition: all 0.12s;
  }
  .sort-dropdown-btn:hover {
    color: #F0F6FC;
    border-color: #F97316;
  }
  .icon-sort, .icon-chevron {
    font-size: 14px;
  }

  /* Table Header (24px) */
  .table-header {
    height: 24px;
    min-height: 24px;
    background: #101319;
    border-bottom: 1px solid rgba(88, 66, 55, 0.2);
    display: flex;
    align-items: center;
    padding: 0 12px;
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    color: #8B949E;
    letter-spacing: 0.05em;
  }
  .col-name {
    flex: 1;
    min-width: 0;
  }
  .col-size {
    width: 100px;
    text-align: right;
    padding-right: 8px;
  }
  .col-actions {
    width: 76px;
    text-align: right;
  }

  /* Table Rows */
  .table-body {
    flex: 1;
    overflow-y: auto;
    background: #0B0E14;
  }
  .table-row {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    border-bottom: 1px solid rgba(88, 66, 55, 0.12);
    cursor: pointer;
    transition: background 0.12s;
    height: 48px;
    box-sizing: border-box;
  }
  .table-row:hover {
    background: #191C22;
  }
  .table-row.row-purgeable {
    background: rgba(244, 63, 94, 0.04);
  }
  .table-row.row-purgeable:hover {
    background: rgba(244, 63, 94, 0.09);
  }

  /* Cell Meta */
  .cell-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .icon-box {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    border: 1px solid rgba(88, 66, 55, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .row-glyph {
    font-size: 15px;
  }
  .name-cluster {
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 1px;
  }
  .name-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .item-name {
    font-family: var(--font-sans);
    font-size: 0.75rem;
    font-weight: 600;
    color: #F0F6FC;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .category-pill {
    font-family: var(--font-mono);
    font-size: 0.5rem;
    font-weight: 700;
    padding: 0 4px;
    border-radius: 3px;
    border: 1px solid transparent;
    flex-shrink: 0;
  }
  .item-path {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Cell Size */
  .cell-size {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }
  .size-val {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 700;
  }
  .mini-bar-track {
    width: 60px;
    height: 3px;
    background: #191C22;
    border-radius: 2px;
    overflow: hidden;
  }
  .mini-bar-fill {
    height: 100%;
    border-radius: 2px;
  }

  /* Cell Actions */
  .cell-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
  }
  .btn-action-icon {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #8B949E;
    border-radius: 4px;
    cursor: pointer;
    opacity: 0;
    transition: all 0.12s;
  }
  .table-row:hover .btn-action-icon {
    opacity: 1;
  }
  .btn-action-icon:hover {
    background: #272A31;
    color: #F0F6FC;
  }
  .btn-action-icon.delete:hover {
    color: #F43F5E;
    background: rgba(244, 63, 94, 0.15);
  }
  .btn-action-icon span {
    font-size: 15px;
  }

  .btn-purge-pill {
    padding: 2px 8px;
    background: rgba(244, 63, 94, 0.15);
    border: 1px solid rgba(244, 63, 94, 0.4);
    color: #ffb4ab;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.625rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s;
  }
  .btn-purge-pill:hover {
    background: #F43F5E;
    color: white;
  }

  /* Empty state */
  .empty-list {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #8B949E;
    font-size: 0.75rem;
    padding: 2rem;
  }

  /* Panel Footer (28px) */
  .panel-footer {
    height: 28px;
    min-height: 28px;
    background: #101319;
    border-top: 1px solid rgba(88, 66, 55, 0.2);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-family: var(--font-mono);
    font-size: 0.625rem;
  }
  .footer-count {
    color: #8B949E;
  }
  .footer-cta {
    color: #14B8A6;
    cursor: pointer;
    transition: color 0.12s;
  }
  .footer-cta:hover {
    text-decoration: underline;
  }
</style>
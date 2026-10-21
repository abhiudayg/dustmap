<script lang="ts">
  import { onMount } from 'svelte';
  import { dataList, dataRemove } from '../lib/invoke';
  import { formatBytes } from '../lib/format';
  import type { DataEntry } from '../lib/types';

  const KEEP_BY_DEFAULT = new Set(['audit-log']);
  const ESSENTIAL = new Set(['settings', 'audit-log']);

  let entries = $state<DataEntry[]>([]);
  let chosen = $state<Set<string>>(new Set());
  let focused = $state<string | null>(null);
  let confirming = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let removed = $state<string[] | null>(null);

  async function load() {
    error = null;
    try {
      entries = await dataList();
      chosen = new Set(entries.filter((entry) => !KEEP_BY_DEFAULT.has(entry.label)).map((entry) => entry.label));
      focused = entries[0]?.label ?? null;
      confirming = false;
    } catch (err) {
      error = String(err);
    }
  }

  onMount(load);

  let totalBytes = $derived(entries.reduce((sum, entry) => sum + entry.bytes, 0));
  let selectedEntries = $derived(entries.filter((entry) => chosen.has(entry.label)));
  let selectedBytes = $derived(selectedEntries.reduce((sum, entry) => sum + entry.bytes, 0));
  let regenerableBytes = $derived(
    entries.filter((entry) => !ESSENTIAL.has(entry.label)).reduce((sum, entry) => sum + entry.bytes, 0),
  );
  let focusedEntry = $derived(entries.find((entry) => entry.label === focused) ?? null);

  function toggle(label: string) {
    const next = new Set(chosen);
    if (next.has(label)) {
      next.delete(label);
    } else {
      next.add(label);
    }
    chosen = next;
    confirming = false;
  }

  async function remove() {
    busy = true;
    error = null;
    try {
      removed = await dataRemove([...chosen]);
      await load();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<section class="data" aria-labelledby="data-title">
  <header class="head">
    <h1 id="data-title">Your DustMap Data</h1>
    <p class="muted">Removing data never touches files you scanned or cleaned.</p>
  </header>

  <div class="tiles">
    <div class="tile">
      <div class="tile-label">Total footprint</div>
      <div class="tile-value mono">{formatBytes(totalBytes)}</div>
      <div class="muted">{entries.length} items</div>
    </div>
    <div class="tile">
      <div class="tile-label">Selected for removal</div>
      <div class="tile-value mono">{formatBytes(selectedBytes)}</div>
      <div class="muted">{selectedEntries.length} selected</div>
    </div>
    <div class="tile">
      <div class="tile-label">Regenerable</div>
      <div class="tile-value mono">{formatBytes(regenerableBytes)}</div>
      <div class="muted">Rebuilt by DustMap when needed</div>
    </div>
  </div>

  {#if error}
    <p class="notice" role="alert">{error}</p>
  {/if}
  {#if removed}
    <p class="muted">Removed: {removed.length > 0 ? removed.join(', ') : 'nothing'}</p>
  {/if}

  <div class="panes">
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th aria-label="Select"></th>
            <th>Item</th>
            <th>Path</th>
            <th>Size</th>
          </tr>
        </thead>
        <tbody>
          {#each entries as entry (entry.label)}
            <tr class:focused={focused === entry.label} onclick={() => (focused = entry.label)}>
              <td>
                <input
                  type="checkbox"
                  checked={chosen.has(entry.label)}
                  onclick={(event) => event.stopPropagation()}
                  onchange={() => toggle(entry.label)}
                  aria-label={`Remove ${entry.label}`}
                />
              </td>
              <td>
                {entry.label}
                {#if entry.label === 'audit-log'}
                  <div class="warning">Deleting the audit log removes the record of past cleanups.</div>
                {/if}
              </td>
              <td class="mono path">{entry.path}</td>
              <td class="mono">{formatBytes(entry.bytes)}</td>
            </tr>
          {:else}
            <tr>
              <td colspan="4" class="muted pad">No DustMap data on this Mac yet.</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <aside class="inspector" aria-label="Selection telemetry">
      <h2>Selection telemetry</h2>
      {#if focusedEntry}
        <div class="mono path">{focusedEntry.path}</div>
        <div class="block">
          <div class="block-title">Size</div>
          <p class="mono">{formatBytes(focusedEntry.bytes)}</p>
        </div>
      {/if}
      <div class="block">
        <div class="block-title">Will be removed</div>
        <p>{selectedEntries.map((entry) => entry.label).join(', ') || 'Nothing selected'}</p>
      </div>
    </aside>
  </div>

  <footer class="action-bar">
    {#if confirming}
      <span>Remove {selectedEntries.length} item(s), {formatBytes(selectedBytes)}? This cannot be undone.</span>
      <button class="primary" disabled={busy || chosen.size === 0} onclick={remove}>
        {busy ? 'Removing…' : 'Confirm remove'}
      </button>
      <button class="ghost" onclick={() => (confirming = false)}>Cancel</button>
    {:else}
      <span class="muted">{selectedEntries.length} selected</span>
      <button class="primary" disabled={chosen.size === 0} onclick={() => (confirming = true)}>Remove selected</button>
    {/if}
  </footer>
</section>

<style>
  .data {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    flex: 1;
    min-width: 0;
    min-height: 0;
    color: var(--color-on-surface);
    background: var(--color-surface);
  }
  h1 {
    font-size: var(--text-xl);
    font-weight: 600;
  }
  h2 {
    font-size: var(--text-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-outline);
    margin-bottom: 8px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .tile {
    padding: 10px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
  }
  .tile-label {
    font-size: var(--text-sm);
    color: var(--color-outline);
  }
  .tile-value {
    font-size: var(--text-xl);
    font-weight: 600;
    margin: 2px 0;
  }
  .panes {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 12px;
    min-height: 0;
    flex: 1;
  }
  .table-wrap {
    overflow: hidden;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-base);
  }
  th,
  td {
    text-align: left;
    padding: 6px 8px;
    border-bottom: 1px solid var(--color-outline-variant);
    vertical-align: top;
  }
  th {
    color: var(--color-outline);
    font-weight: 500;
    background: var(--color-surface-container-lowest);
  }
  tr {
    cursor: pointer;
  }
  tr.focused {
    background: var(--color-surface-container-high);
  }
  .mono {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .path {
    word-break: break-all;
  }
  .warning {
    margin-top: 4px;
    color: var(--color-primary);
    font-size: var(--text-sm);
  }
  .inspector {
    overflow: auto;
    padding: 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
    font-size: var(--text-base);
  }
  .block {
    margin-top: 12px;
  }
  .block-title {
    font-size: var(--text-sm);
    color: var(--color-outline);
    margin-bottom: 4px;
  }
  .action-bar {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: flex-end;
    padding-top: 8px;
    border-top: 1px solid var(--color-outline-variant);
    font-size: var(--text-base);
  }
  .primary,
  .ghost {
    padding: 6px 12px;
    border-radius: 6px;
    font: inherit;
    cursor: pointer;
  }
  .primary {
    border: none;
    font-weight: 600;
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }
  .primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .ghost {
    background: var(--color-surface-container-high);
    border: 1px solid var(--color-outline-variant);
    color: inherit;
  }
  .notice {
    color: var(--color-error);
  }
  .muted {
    color: var(--color-on-surface-variant);
    font-size: var(--text-base);
  }
  .pad {
    padding: 12px;
  }
  .primary, .ghost {
    padding: 5px 10px;
    font-size: var(--text-base);
    font-weight: 500;
  }
  .action-bar,
  .action-bar {
    flex-shrink: 0;
  }
  .panes {
    flex: 1;
    min-height: 0;
    grid-template-rows: minmax(0, 1fr);
  }
  .panes > .table-wrap,
  .panes > .inspector {
    min-height: 0;
    max-height: none;
    overflow: auto;
  }
</style>

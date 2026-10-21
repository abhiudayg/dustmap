<script lang="ts">
  import { remnantRemove, remnantSearch } from '../lib/invoke';
  import type { Remnant, RiskLevel } from '../lib/types';

  let bundleId = $state('');
  let results = $state<Remnant[] | null>(null);
  let chosen = $state<Set<string>>(new Set());
  let focused = $state<string | null>(null);
  let filter = $state<string>('All');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let confirming = $state(false);
  let cautionAck = $state(false);

  const RISK_LABELS: Record<RiskLevel, { text: string; icon: string }> = {
    safe: { text: 'Safe', icon: 'check_circle' },
    review: { text: 'Review', icon: 'info' },
    caution: { text: 'Caution', icon: 'warning' },
    'report-only': { text: 'Report only', icon: 'visibility' },
  };

  let canSearch = $derived(/^[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)+$/.test(bundleId.trim()));
  let locations = $derived(['All', ...new Set((results ?? []).map((item) => item.location))]);
  let visible = $derived((results ?? []).filter((item) => filter === 'All' || item.location === filter));
  let focusedItem = $derived(results?.find((item) => item.path === focused) ?? null);
  let safeCount = $derived((results ?? []).filter((item) => item.risk === 'safe').length);
  let removable = $derived((results ?? []).filter((item) => chosen.has(item.path) && item.risk !== 'report-only'));
  let hasCaution = $derived(removable.some((item) => item.risk === 'caution'));

  async function search() {
    if (!canSearch) return;
    busy = true;
    error = null;
    try {
      results = await remnantSearch(bundleId.trim());
      chosen = new Set();
      confirming = false;
      cautionAck = false;
      filter = 'All';
      focused = results[0]?.path ?? null;
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  function toggle(path: string) {
    const next = new Set(chosen);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    chosen = next;
    confirming = false;
    cautionAck = false;
  }

  async function remove() {
    busy = true;
    let failure: string | null = null;
    try {
      await remnantRemove(bundleId.trim(), removable.map((item) => item.path), cautionAck);
    } catch (err) {
      failure = String(err);
    }
    confirming = false;
    cautionAck = false;
    await search();
    error = failure;
    busy = false;
  }

  function selectSafe() {
    chosen = new Set((results ?? []).filter((item) => item.risk === 'safe').map((item) => item.path));
  }
</script>

<section class="remnants" aria-labelledby="remnants-title">
  <header class="head">
    <div>
      <h1 id="remnants-title">App Cleanups</h1>
      <p class="muted">Find leftovers for an uninstalled app by its bundle identifier.</p>
    </div>
    <span class="dry-run">Moves to Trash only</span>
  </header>

  <form class="search" onsubmit={(event) => { event.preventDefault(); search(); }}>
    <input type="text" bind:value={bundleId} placeholder="com.example.app" aria-label="Bundle identifier" spellcheck="false" />
    <button class="primary" type="submit" disabled={!canSearch || busy}>{busy ? 'Searching…' : 'Search'}</button>
  </form>

  {#if error}
    <p class="notice" role="alert">{error}</p>
  {/if}

  {#if results && results.length > 0}
    <div class="tabs" role="tablist" aria-label="Filter by location">
      {#each locations as name (name)}
        <button role="tab" aria-selected={filter === name} class:active={filter === name} onclick={() => (filter = name)}>{name}</button>
      {/each}
    </div>
  {/if}

  <div class="panes">
    <div class="table-wrap">
      {#if results && results.length > 0}
        <table>
          <thead>
            <tr>
              <th aria-label="Select"></th>
              <th>Location</th>
              <th>Matched path</th>
              <th>Matched by</th>
              <th>Risk</th>
            </tr>
          </thead>
          <tbody>
            {#each visible as item (item.path)}
              {@const label = RISK_LABELS[item.risk]}
              <tr class:focused={focused === item.path} onclick={() => (focused = item.path)}>
                <td>
                  <input
                    type="checkbox"
                    checked={chosen.has(item.path)}
                    onclick={(event) => event.stopPropagation()}
                    onchange={() => toggle(item.path)}
                    aria-label={`Select ${item.path}`}
                  />
                </td>
                <td>{item.location}</td>
                <td class="mono path">{item.path}</td>
                <td>{item.match_kind}</td>
                <td>
                  <span class="risk risk-{item.risk}">
                    <span class="material-symbols-outlined" aria-hidden="true">{label.icon}</span>
                    {label.text}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else if results}
        <p class="muted pad">No leftovers offered. Apps still installed or running are never listed.</p>
      {:else}
        <p class="muted pad">Search for a bundle identifier to see its leftovers.</p>
      {/if}
    </div>

    <aside class="inspector" aria-label="Artifact inspector">
      <h2>Artifact inspector</h2>
      {#if focusedItem}
        <div class="mono path">{focusedItem.path}</div>
        <div class="block">
          <div class="block-title">Location</div>
          <p>{focusedItem.location}</p>
        </div>
        <div class="block">
          <div class="block-title">Matched by</div>
          <p>{focusedItem.match_kind}</p>
        </div>
        <div class="block">
          <div class="block-title">Risk</div>
          <p><span class="risk risk-{focusedItem.risk}">{RISK_LABELS[focusedItem.risk].text}</span></p>
          <p class="muted">{focusedItem.note}</p>
        </div>
      {:else}
        <p class="muted">Select a result to inspect it.</p>
      {/if}
    </aside>
  </div>

  <footer class="action-bar">
    <span class="muted note-left">Apps still installed or currently running are never listed.</span>
    <span class="muted">{chosen.size} selected · {safeCount} safe</span>
    <button class="ghost" disabled={safeCount === 0} onclick={selectSafe}>Select safe items</button>
    {#if confirming}
      {#if hasCaution}
        <label class="caution-ack">
          <input type="checkbox" bind:checked={cautionAck} />
          Caution items may hold activation data. Move them anyway.
        </label>
      {/if}
      <span>Move {removable.length} item(s) to Trash?</span>
      <button class="primary" disabled={busy || removable.length === 0 || (hasCaution && !cautionAck)} onclick={remove}>
        {busy ? 'Moving…' : 'Confirm move to Trash'}
      </button>
    {:else}
      <button class="primary" disabled={removable.length === 0} onclick={() => (confirming = true)}>Move selected to Trash</button>
    {/if}
  </footer>
</section>

<style>
  .remnants {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    color: var(--color-on-surface);
    background: var(--color-surface);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 12px;
  }
  h1 {
    font-size: var(--text-xl);
    font-weight: 600;
  }
  h2 {
    font-size: var(--text-base);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-outline);
    margin-bottom: 8px;
  }
  .dry-run {
    font-size: var(--text-base);
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-primary-container);
    color: var(--color-primary);
  }
  .search {
    display: flex;
    gap: 8px;
  }
  .search input {
    flex: 1;
    padding: 6px 8px;
    border-radius: 6px;
    font-family: var(--font-sans);
    font-size: var(--text-base);
    background: var(--color-surface-container-lowest);
    color: inherit;
    border: 1px solid var(--color-outline-variant);
  }
  .tabs {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .tabs button {
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: none;
    color: var(--color-on-surface-variant);
    font: inherit;
    font-size: var(--text-base);
    cursor: pointer;
  }
  .tabs button.active {
    background: var(--color-surface-container-high);
    border-color: var(--color-outline-variant);
    color: var(--color-on-surface);
  }
  .panes {
    display: grid;
    grid-template-columns: 1fr 300px;
    gap: 12px;
    min-height: 0;
  }
  .table-wrap {
    overflow: auto;
    max-height: 420px;
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
    font-size: var(--text-base);
  }
  .path {
    word-break: break-all;
  }
  .risk {
    display: inline-flex;
    gap: 4px;
    align-items: center;
    padding: 1px 6px;
    border: 1px solid currentColor;
    border-radius: 4px;
  }
  .risk-safe { color: var(--color-secondary); }
  .risk-review { color: var(--color-tertiary); }
  .risk-caution { color: var(--color-primary); }
  .risk-report-only { color: var(--color-outline); }
  .inspector {
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
    font-size: var(--text-base);
    color: var(--color-outline);
    margin-bottom: 4px;
  }
  .action-bar {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: flex-end;
    padding-top: 10px;
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
  .primary:disabled,
  .ghost:disabled {
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
  .foot {
    margin-top: 0;
  }
  .primary, .ghost {
    padding: 5px 10px;
    font-size: var(--text-base);
    font-weight: 500;
  }
  .footer,
  .action-bar,
  .confirm-bar {
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
  .note-left {
    margin-right: auto;
  }
  .block .muted {
    margin-top: 8px;
  }
  .caution-ack {
    display: flex;
    gap: 6px;
    align-items: center;
  }
</style>

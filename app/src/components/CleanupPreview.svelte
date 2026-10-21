<script lang="ts">
  import { onMount } from 'svelte';
  import { cleanupExecute, cleanupPreview } from '../lib/invoke';
  import { appState } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';
  import type { CleanupPreview, CleanupResult, RiskLevel, RuleInfo } from '../lib/types';

  let preview = $state<CleanupPreview | null>(null);
  let selected = $state<Set<string>>(new Set());
  let activeRuleId = $state<string | null>(null);
  let filter = $state('');
  let pending = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let results = $state<CleanupResult[] | null>(null);

  const RISK_LABELS: Record<RiskLevel, string> = {
    safe: 'Safe to Purge',
    review: 'Review Recommended',
    caution: 'Caution',
    'report-only': 'Report only',
  };

  async function load() {
    error = null;
    try {
      const next = await cleanupPreview();
      preview = next;
      selected = new Set(next.rules.filter((rule) => rule.risk === 'safe').map((rule) => rule.id));
      activeRuleId = next.rules[0]?.id ?? null;
      pending = false;
    } catch (err) {
      error = String(err);
    }
  }

  onMount(load);

  function ruleBytes(ruleId: string): number {
    return preview?.targets.filter((t) => t.rule_id === ruleId).reduce((sum, t) => sum + t.logical_bytes, 0) ?? 0;
  }

  function ruleCount(ruleId: string): number {
    return (
      (preview?.targets.filter((t) => t.rule_id === ruleId).length ?? 0) +
      (preview?.tool_targets.filter((t) => t.rule_id === ruleId).length ?? 0)
    );
  }

  function firstPath(ruleId: string): string {
    return preview?.targets.find((t) => t.rule_id === ruleId)?.path ?? '';
  }

  let visibleRules = $derived(
    (preview?.rules ?? []).filter((rule) => {
      const query = filter.trim().toLowerCase();
      return !query || rule.name.toLowerCase().includes(query) || firstPath(rule.id).toLowerCase().includes(query);
    }),
  );

  let groups = $derived.by(() => {
    const byCategory = new Map<string, RuleInfo[]>();
    for (const rule of visibleRules) {
      const list = byCategory.get(rule.category) ?? [];
      list.push(rule);
      byCategory.set(rule.category, list);
    }
    return [...byCategory.entries()];
  });

  let totalRules = $derived(preview?.rules.length ?? 0);
  let selectedRuleCount = $derived(selected.size);
  let selectedItemCount = $derived(
    (preview?.targets.filter((t) => selected.has(t.rule_id)).length ?? 0) +
      (preview?.tool_targets.filter((t) => selected.has(t.rule_id)).length ?? 0),
  );
  let selectedBytes = $derived(
    preview?.targets.filter((t) => selected.has(t.rule_id)).reduce((sum, t) => sum + t.logical_bytes, 0) ?? 0,
  );
  let activeRule = $derived(preview?.rules.find((rule) => rule.id === activeRuleId) ?? null);
  let volume = $derived(appState.currentVolume);
  let usedBytes = $derived(volume ? volume.used_bytes : 0);
  let totalBytes = $derived(volume ? volume.total_bytes : 0);
  let usedPercent = $derived(totalBytes > 0 ? (usedBytes / totalBytes) * 100 : 0);
  let predictedPercent = $derived(totalBytes > 0 ? (Math.max(0, usedBytes - selectedBytes) / totalBytes) * 100 : 0);

  function toggleRule(ruleId: string) {
    pending = false;
    const next = new Set(selected);
    if (next.has(ruleId)) {
      next.delete(ruleId);
    } else {
      next.add(ruleId);
    }
    selected = next;
  }

  async function moveToTrash() {
    if (selected.size === 0) return;
    if (!pending) {
      pending = true;
      return;
    }
    busy = true;
    error = null;
    pending = false;
    try {
      results = await cleanupExecute([...selected], true);
      await load();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<section class="cleanup" aria-labelledby="cleanup-title">
  <div class="filter-bar">
    <input type="search" bind:value={filter} placeholder="Filter paths or rule names (e.g. DerivedData, cache, pods)…" aria-label="Filter candidates" />
    <span class="chip mono">Selected: {selectedItemCount} items ({formatBytes(selectedBytes)})</span>
  </div>

  {#if error}
    <p class="notice" role="alert">{error}</p>
  {/if}

  {#if preview}
    <div class="panes">
      <div class="list">
        <h1 id="cleanup-title" class="sr-only">Cleanup Preview</h1>
        {#each groups as [category, rules] (category)}
          {@const categoryBytes = rules.reduce((sum, rule) => sum + ruleBytes(rule.id), 0)}
          <div class="group-head">
            <span class="group-title">{category}</span>
            <span class="badge mono">{rules.length} Candidates</span>
            <span class="subtotal mono">Subtotal: {formatBytes(categoryBytes)}</span>
          </div>
          {#each rules as rule (rule.id)}
            <div class="row" class:active={rule.id === activeRuleId}>
              <input
                type="checkbox"
                checked={selected.has(rule.id)}
                onchange={() => toggleRule(rule.id)}
                aria-label={`Include ${rule.name}`}
              />
              <button class="row-main" onclick={() => (activeRuleId = rule.id)}>
                <span class="row-top">
                  <span class="row-name">{rule.name}</span>
                  <span class="risk risk-{rule.risk}">{RISK_LABELS[rule.risk]}</span>
                </span>
                <span class="row-path mono">{firstPath(rule.id) || 'No matching items'}{ruleCount(rule.id) ? ` · ${ruleCount(rule.id)} items` : ''}</span>
              </button>
              <span class="size mono" class:size-on={selected.has(rule.id)}>{formatBytes(ruleBytes(rule.id))}</span>
            </div>
          {/each}
        {:else}
          <p class="muted pad">No rules match the filter.</p>
        {/each}

        {#if preview.rejected.length > 0}
          <section class="protected" aria-label="Protected paths bypassed">
            <div class="protected-title">Protected paths bypassed ({preview.rejected.length} automatically skipped)</div>
            <div class="protected-grid">
              {#each preview.rejected as item (item.label)}
                <div class="protected-card">
                  <div class="mono path">{item.label}</div>
                  <div class="muted">{item.reason}</div>
                </div>
              {/each}
            </div>
          </section>
        {/if}
      </div>

      <aside class="inspector" aria-label="Rule impact inspector">
        <div class="inspector-head">
          <span class="inspector-label">Rule impact inspector</span>
          {#if activeRule}
            <span class="status-chip">{selected.has(activeRule.id) ? 'Selected for Purge' : 'Not selected'}</span>
          {/if}
        </div>
        {#if activeRule}
          <h2>{activeRule.name}</h2>
          <p class="muted">{activeRule.summary}</p>
          <div class="path-card mono">{firstPath(activeRule.id) || 'No matching items'}</div>

          <div class="card">
            <div class="card-head">
              <span>Volume capacity impact{volume ? ` (${volume.name})` : ''}</span>
              <span class="reclaim mono">+{formatBytes(ruleBytes(activeRule.id))} reclaimable</span>
            </div>
            <div class="capacity-bar">
              <span class="seg-used" style="width: {usedPercent}%"></span>
              <span class="seg-predicted" style="width: {Math.max(0, usedPercent - predictedPercent)}%"></span>
            </div>
            <div class="capacity-labels mono">
              <span>Used: {formatBytes(usedBytes)}</span>
              <span>Predicted: {formatBytes(Math.max(0, usedBytes - selectedBytes))}</span>
              <span>Total: {formatBytes(totalBytes)}</span>
            </div>
          </div>

          <div class="card">
            <div class="card-head"><span>What it removes</span></div>
            <p>{activeRule.summary}</p>
          </div>
          <div class="card">
            <div class="card-head"><span>Consequences</span></div>
            <p>{activeRule.consequence}</p>
          </div>
          <div class="card">
            <div class="card-head"><span>How to undo (safe quarantine)</span></div>
            <p>{activeRule.undo}</p>
          </div>
        {:else}
          <p class="muted">Select a rule to see what it removes.</p>
        {/if}
      </aside>
    </div>

    <footer class="footer">
      <div class="reclaim-summary">
        <span class="mono muted">Target reclaim</span>
        <span class="big mono">{formatBytes(selectedBytes)}</span>
        <span class="muted">of {formatBytes(preview.estimated_bytes)} selectable · {selectedRuleCount} of {totalRules} rules</span>
      </div>
      <div class="actions">
        <button class="primary" disabled={selected.size === 0 || busy} onclick={moveToTrash}>
          {busy ? "Moving…" : pending ? `Confirm move (${selectedItemCount})` : `Move ${selectedItemCount} items to Trash (${formatBytes(selectedBytes)})`}
        </button>
      </div>
    </footer>
  {/if}

  {#if results}
    <ul class="results">
      {#each results as result (result.item_id)}
        <li class="mono">{result.path ?? result.rule_id}: {result.outcome}{result.detail ? ` (${result.detail})` : ''}</li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .cleanup {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 14px;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    color: var(--color-on-surface);
    background: var(--color-surface);
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
  .filter-bar {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .filter-bar input {
    flex: 1;
    padding: 6px 10px;
    border-radius: 6px;
    font: inherit;
    font-size: var(--text-base);
    background: var(--color-surface-container-lowest);
    color: inherit;
    border: 1px solid var(--color-outline-variant);
  }
  .chip {
    padding: 5px 10px;
    border-radius: 6px;
    border: 1px solid var(--color-secondary);
    color: var(--color-secondary);
    font-size: var(--text-base);
    white-space: nowrap;
  }
  .panes {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 360px;
    gap: 12px;
    min-height: 0;
    flex: 1;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .group-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }
  .group-title {
    font-size: var(--text-lg);
    font-weight: 600;
  }
  .badge {
    padding: 1px 8px;
    border-radius: 4px;
    background: var(--color-surface-container-high);
    font-size: var(--text-sm);
    color: var(--color-on-surface-variant);
  }
  .subtotal {
    margin-left: auto;
    font-size: var(--text-sm);
    color: var(--color-outline);
  }
  .row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    padding: 10px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
  }
  .row.active {
    border-left: 3px solid var(--color-primary-container);
    background: var(--color-surface-container-high);
  }
  .row-main {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    padding: 0;
  }
  .row-top {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .row-name {
    font-size: var(--text-lg);
    font-weight: 500;
  }
  .row-path {
    font-size: var(--text-sm);
    color: var(--color-on-surface-variant);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .risk {
    font-size: var(--text-sm);
    padding: 1px 6px;
    border: 1px solid currentColor;
    border-radius: 4px;
  }
  .risk-safe { color: var(--color-secondary); }
  .risk-review { color: var(--color-tertiary); }
  .risk-caution { color: var(--color-primary); }
  .risk-report-only { color: var(--color-outline); }
  .size {
    padding: 4px 10px;
    border-radius: 4px;
    border: 1px solid var(--color-outline-variant);
    font-size: var(--text-base);
    color: var(--color-outline);
  }
  .size-on {
    background: var(--color-primary-container);
    border-color: var(--color-primary-container);
    color: var(--color-on-primary-container);
    font-weight: 600;
  }
  .protected {
    margin-top: 8px;
  }
  .protected-title {
    font-size: var(--text-sm);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-outline);
    margin-bottom: 6px;
  }
  .protected-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 8px;
    font-size: var(--text-base);
  }
  .protected-card {
    padding: 8px 10px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    background: var(--color-surface-container-low);
  }
  .inspector {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-low);
    font-size: var(--text-base);
    min-height: 0;
    overflow: auto;
  }
  .inspector-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .inspector-label {
    font-size: var(--text-sm);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-secondary);
  }
  .status-chip {
    font-size: var(--text-sm);
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-primary-container);
    color: var(--color-primary);
  }
  .inspector h2 {
    font-size: var(--text-xl);
    font-weight: 600;
  }
  .path-card {
    padding: 8px 10px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 6px;
    background: var(--color-surface-container-lowest);
    font-size: var(--text-base);
    word-break: break-all;
  }
  .card {
    padding: 10px 12px;
    border: 1px solid var(--color-outline-variant);
    border-radius: 8px;
    background: var(--color-surface-container-lowest);
  }
  .card-head {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: var(--text-base);
    color: var(--color-on-surface-variant);
    margin-bottom: 6px;
  }
  .reclaim {
    color: var(--color-secondary);
  }
  .capacity-bar {
    display: flex;
    height: 8px;
    border-radius: 4px;
    overflow: hidden;
    background: var(--color-surface-container-high);
  }
  .seg-used {
    display: block;
    background: var(--color-primary-container);
  }
  .seg-predicted {
    display: block;
    background: var(--color-secondary);
  }
  .capacity-labels {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: var(--text-sm);
    margin-top: 6px;
    color: var(--color-on-surface-variant);
  }
  .footer {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    padding-top: 10px;
    border-top: 1px solid var(--color-outline-variant);
  }
  .reclaim-summary {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: var(--text-base);
  }
  .big {
    font-size: var(--text-2xl);
    color: var(--color-secondary);
    font-weight: 600;
  }
  .confirm {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: var(--text-base);
  }
  .primary,
  .ghost {
    padding: 8px 14px;
    border-radius: 6px;
    font: inherit;
    cursor: pointer;
  }
  .primary {
    margin-left: auto;
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
  .mono {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .path {
    word-break: break-all;
  }
  .results {
    font-size: var(--text-base);
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
  .panes > .list,
  .panes > .inspector {
    min-height: 0;
    max-height: none;
    overflow: auto;
  }
  .footer {
    align-items: flex-end;
    justify-content: space-between;
  }
  .footer .actions {
    display: flex;
    gap: 8px;
    align-items: flex-end;
  }
  .footer .primary {
    margin-left: 0;
  }
  .reclaim-summary {
    display: flex;
    flex-direction: row;
    align-items: baseline;
    gap: 8px;
    white-space: nowrap;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .reclaim-summary .big {
    font-size: var(--text-sm);
    font-weight: 400;
  }
  .footer .primary {
    white-space: nowrap;
  }
  .footer {
    min-height: 36px;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { acknowledgeQuarantine, dismissInterruptedScan, startupReport } from '../lib/invoke';
  import type { StartupReport } from '../lib/types';

  let { onrescan }: { onrescan: (root: string) => void } = $props();

  let report = $state<StartupReport | null>(null);
  let dismissed = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      report = await startupReport();
    } catch (err) {
      error = String(err);
    }
  });

  async function dismiss() {
    dismissed = true;
    try {
      await dismissInterruptedScan();
    } catch (err) {
      error = String(err);
    }
  }

  async function acknowledge() {
    error = null;
    try {
      await acknowledgeQuarantine();
      report = { ...(report as StartupReport), quarantined_journals: [] };
    } catch (err) {
      error = String(err);
    }
  }

  let visible = $derived(
    !dismissed &&
      report !== null &&
      (report.interrupted_scan !== null ||
        report.items_needing_review.length > 0 ||
        report.quarantined_journals.length > 0 ||
        report.errors.length > 0),
  );
</script>

{#if visible && report}
  <section class="startup-banner" aria-live="polite">
    {#if report.interrupted_scan}
      <div class="row">
        <p>A scan of <span class="mono">{report.interrupted_scan}</span> was interrupted. Results may be incomplete.</p>
        <div class="actions">
          <button class="primary" onclick={() => { dismissed = true; onrescan(report?.interrupted_scan ?? ''); }}>Scan again</button>
          <button class="ghost" onclick={dismiss}>Dismiss</button>
        </div>
      </div>
    {/if}

    {#if report.items_needing_review.length > 0}
      <div class="row">
        <p>Cleanup needs review: {report.items_needing_review.length} item(s) from an earlier run have an unknown state.</p>
        <ul class="mono">
          {#each report.items_needing_review as item (item)}
            <li>{item}</li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if report.quarantined_journals.length > 0}
      <div class="row">
        <p>Cleanup is paused until you acknowledge this warning: a cleanup record could not be read and was set aside.</p>
        <button class="primary" onclick={acknowledge}>Acknowledge</button>
      </div>
    {/if}

    {#each report.errors as message (message)}
      <p class="notice">{message}</p>
    {/each}

    {#if error}
      <p class="notice" role="alert">{error}</p>
    {/if}
  </section>
{/if}

<style>
  .startup-banner {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 16px;
    background: var(--color-surface-container-low);
    border-bottom: 1px solid var(--color-outline-variant);
    color: var(--color-on-surface);
    font-size: var(--text-base);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: space-between;
    align-items: center;
  }
  .row p {
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .mono {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .primary,
  .ghost {
    padding: 4px 10px;
    border-radius: 6px;
    font: inherit;
    cursor: pointer;
    border: 1px solid var(--color-outline-variant);
  }
  .primary {
    background: var(--color-primary-container);
    color: var(--color-on-primary-container);
    border-color: transparent;
  }
  .ghost {
    background: var(--color-surface-container-high);
    color: inherit;
  }
  .notice {
    color: var(--color-error);
  }
</style>

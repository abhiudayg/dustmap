<script lang="ts">
  import { appState, clearDeleteTarget, performDelete, getEntrySize } from '../lib/store.svelte';
  import { formatBytes } from '../lib/format';

  let understood = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  async function doDelete(permanent: boolean) {
    if (!appState.deleteTarget) return;
    if (permanent && !understood) return;
    deleting = true;
    error = null;
    try {
      await performDelete(appState.deleteTarget.path, permanent, appState.deleteTarget.entry.id);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
      // Keep modal open so error is fully visible to the user!
    } finally {
      deleting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !deleting) {
      clearDeleteTarget();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if appState.deleteTarget}
  {@const target = appState.deleteTarget}
  {@const size = getEntrySize(target.entry)}
  <div class="modal-backdrop" onclick={clearDeleteTarget} role="presentation">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal-card"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Confirm Deletion"
      tabindex="-1"
    >
      <!-- Technical Corner Metadata Labels -->
      <div class="corner-tag left">+ SEC_OPS_L3 +</div>
      <div class="corner-tag right">INODE #{target.entry.id} // APFS</div>
      <div class="ambient-glow"></div>

      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-left">
          <div class="warning-badge">
            <span class="warning-icon">⚠️</span>
            <span class="ping-dot"></span>
          </div>
          <div class="header-text">
            <h2 class="header-title">Confirm Deletion</h2>
            <p class="header-subtitle">Surgical disk reclaim protocol triggered</p>
          </div>
        </div>
        <button class="esc-close-btn" onclick={clearDeleteTarget} disabled={deleting} title="Close (Esc)">
          <span>esc</span>
          <span class="close-x">✕</span>
        </button>
      </div>

      <!-- Target File/Folder Telemetry Box -->
      <div class="target-box">
        <div class="target-header">
          <span class="target-label">Target Path</span>
          <span class="resolved-tag">
            <span class="resolved-dot"></span>
            {target.entry.kind === 'Directory' ? 'RESOLVED DIRECTORY' : 'RESOLVED FILE'}
          </span>
        </div>
        <div class="path-display">
          <span class="path-icon">{target.entry.kind === 'Directory' ? '📁' : '📄'}</span>
          <span class="path-text">{target.path}</span>
        </div>
        <div class="target-meta">
          <span>APFS Native • Read/Write</span>
          <span>{target.entry.coverage} Coverage</span>
        </div>
      </div>

      <!-- Space To Reclaim Bento Grid -->
      <div class="bento-grid">
        <!-- Main Big Metric Display -->
        <div class="metric-card">
          <span class="metric-label">Space to Reclaim</span>
          <div class="metric-value">
            {formatBytes(size, appState.useBinaryUnits)}
          </div>
          <span class="reclaim-tag">Physical Disk Allocation</span>
        </div>

        <!-- Telemetry Chips Grid -->
        <div class="chips-stack">
          <div class="telemetry-chip">
            <span class="chip-label">Type</span>
            <span class="chip-value">{target.entry.kind}</span>
          </div>
          {#if target.entry.child_count !== undefined}
            <div class="telemetry-chip">
              <span class="chip-label">Child Items</span>
              <span class="chip-value highlight">{target.entry.child_count.toLocaleString()} items</span>
            </div>
          {/if}
          <div class="telemetry-chip">
            <span class="chip-label">Hidden Item</span>
            <span class="chip-value">{target.entry.is_hidden ? 'Yes (Hidden)' : 'No'}</span>
          </div>
        </div>
      </div>

      <!-- Critical Warning / Failure Callout Box -->
      {#if error}
        <div class="error-callout">
          <span class="callout-icon">🛑</span>
          <div class="callout-text">
            <span class="callout-title">macOS Operation Rejected</span>
            <p class="callout-desc">{error}</p>
          </div>
        </div>
      {:else}
        <div class="warning-callout">
          <span class="callout-icon">⚠️</span>
          <div class="callout-text">
            <span class="callout-title">Irreversible System Action</span>
            <p class="callout-desc">
              DustMap adheres to a conservative Trash-first policy. Items moved to Trash can be restored until emptied. Permanent deletion removes files immediately and cannot be undone.
            </p>
          </div>
        </div>
      {/if}

      <!-- Safety Checkbox for Permanent Deletion -->
      <label class="safety-check">
        <input type="checkbox" bind:checked={understood} />
        <span>I understand that permanent deletion cannot be recovered and bypasses macOS Trash</span>
      </label>

      <!-- Action Buttons Container (Side-by-Side matching Stitch) -->
      <div class="actions-grid">
        <!-- Move to Trash (Safe Option) -->
        <button
          class="btn-trash"
          onclick={() => doDelete(false)}
          disabled={deleting}
          title="Safely move to macOS Trash"
        >
          <span class="btn-icon">🗑</span>
          <span>{deleting && !understood ? 'Moving to Trash…' : 'Move to Trash'}</span>
          <span class="safer-tag">Safer</span>
        </button>

        <!-- Delete Forever (Primary Destructive) -->
        <button
          class="btn-delete"
          onclick={() => doDelete(true)}
          disabled={deleting || !understood}
          title={understood ? 'Permanently delete this path' : 'Check confirmation box to enable'}
        >
          <span class="btn-icon">☠</span>
          <span>{deleting && understood ? 'Deleting…' : 'Delete Forever'}</span>
        </button>
      </div>

      <!-- Secondary Footer Cancel Button -->
      <div class="cancel-footer">
        <button class="btn-cancel-keep" onclick={clearDeleteTarget} disabled={deleting}>
          <span>Cancel (Keep Files)</span>
          <kbd class="kbd-esc">Esc</kbd>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: color-mix(in srgb, var(--color-surface-container-lowest) calc(0.82 * 100%), transparent);
    backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    user-select: none;
  }
  .modal-card {
    position: relative;
    width: 100%;
    max-width: 580px;
    border-radius: 16px;
    background: var(--color-surface-container-low);
    border: 1px solid color-mix(in srgb, var(--color-primary-container) calc(0.45 * 100%), transparent);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.8), 0 0 30px color-mix(in srgb, var(--color-primary-container) calc(0.15 * 100%), transparent);
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow: hidden;
  }
  .corner-tag {
    position: absolute;
    top: 8px;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    letter-spacing: 0.08em;
    pointer-events: none;
  }
  .corner-tag.left {
    left: 12px;
    color: color-mix(in srgb, var(--color-primary-container) calc(0.4 * 100%), transparent);
  }
  .corner-tag.right {
    right: 48px;
    color: var(--color-outline-variant);
  }
  .ambient-glow {
    position: absolute;
    top: -40px;
    right: -40px;
    width: 140px;
    height: 140px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--color-primary-container) calc(0.15 * 100%), transparent);
    filter: blur(30px);
    pointer-events: none;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.25 * 100%), transparent);
    padding-bottom: 0.75rem;
    position: relative;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .warning-badge {
    position: relative;
    width: 38px;
    height: 38px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--color-primary-container) calc(0.15 * 100%), transparent);
    border: 1px solid color-mix(in srgb, var(--color-primary-container) calc(0.4 * 100%), transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-2xl);
  }
  .ping-dot {
    position: absolute;
    top: -2px;
    right: -2px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-primary-container);
    animation: ping 1.5s cubic-bezier(0, 0, 0.2, 1) infinite;
  }
  @keyframes ping {
    75%, 100% { transform: scale(2); opacity: 0; }
  }
  .header-title {
    font-family: var(--font-sans);
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
  }
  .header-subtitle {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-outline);
    margin-top: 2px;
  }
  .esc-close-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--color-surface-container-high);
    border: 1px solid var(--color-outline-variant);
    color: var(--color-outline);
    border-radius: 6px;
    padding: 3px 8px;
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    transition: all 0.15s;
  }
  .esc-close-btn:hover {
    color: var(--color-on-surface);
    border-color: var(--color-outline);
  }

  /* Target Box */
  .target-box {
    background: var(--color-surface-container-lowest);
    border: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.3 * 100%), transparent);
    border-radius: 10px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .target-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .target-label {
    color: var(--color-outline);
    text-transform: uppercase;
    font-weight: 600;
  }
  .resolved-tag {
    color: var(--color-secondary);
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .resolved-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-secondary);
  }
  .path-display {
    display: flex;
    align-items: center;
    gap: 8px;
    background: color-mix(in srgb, var(--color-surface-container-high) calc(0.5 * 100%), transparent);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.2 * 100%), transparent);
    overflow-x: auto;
  }
  .path-icon {
    font-size: var(--text-lg);
    flex-shrink: 0;
  }
  .path-text {
    font-family: var(--font-sans);
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-primary);
    word-break: break-all;
  }
  .target-meta {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-outline);
    padding-top: 2px;
  }

  /* Bento Grid */
  .bento-grid {
    display: grid;
    grid-template-columns: 5fr 7fr;
    gap: 0.75rem;
  }
  .metric-card {
    background: var(--color-surface);
    border: 1px solid color-mix(in srgb, var(--color-primary-container) calc(0.35 * 100%), transparent);
    border-radius: 10px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .metric-label {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    text-transform: uppercase;
    font-weight: 600;
    color: var(--color-primary);
  }
  .metric-value {
    font-family: var(--font-sans);
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-primary-container);
    margin-top: 2px;
  }
  .reclaim-tag {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-secondary);
    margin-top: 4px;
  }
  .chips-stack {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .telemetry-chip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--color-surface);
    border: 1px solid color-mix(in srgb, var(--color-outline-variant) calc(0.25 * 100%), transparent);
    padding: 0.35rem 0.625rem;
    border-radius: 6px;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }
  .chip-label {
    color: var(--color-outline);
  }
  .chip-value {
    color: var(--color-on-surface);
    font-weight: 600;
  }
  .chip-value.highlight {
    color: var(--color-primary-container);
  }

  /* Callout Boxes */
  .warning-callout {
    background: color-mix(in srgb, var(--color-error) calc(0.08 * 100%), transparent);
    border: 1px solid color-mix(in srgb, var(--color-error) calc(0.3 * 100%), transparent);
    border-radius: 8px;
    padding: 0.75rem;
    display: flex;
    gap: 0.625rem;
  }
  .error-callout {
    background: color-mix(in srgb, var(--color-error) calc(0.2 * 100%), transparent);
    border: 1px solid var(--color-error);
    border-radius: 8px;
    padding: 0.75rem;
    display: flex;
    gap: 0.625rem;
    animation: shake 0.3s ease;
  }
  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    25% { transform: translateX(-4px); }
    75% { transform: translateX(4px); }
  }
  .callout-icon {
    font-size: var(--text-xl);
    flex-shrink: 0;
  }
  .callout-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .callout-title {
    font-family: var(--font-sans);
    font-weight: 700;
    font-size: var(--text-base);
    color: var(--color-error);
  }
  .callout-desc {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--color-error);
    line-height: 1.4;
    word-break: break-all;
  }

  /* Safety Check */
  .safety-check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-sans);
    font-size: var(--text-base);
    color: var(--color-on-surface);
    cursor: pointer;
  }
  .safety-check input {
    cursor: pointer;
    accent-color: var(--color-primary-container);
  }

  /* Action Buttons */
  .actions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    padding-top: 0.25rem;
  }
  .btn-trash {
    background: color-mix(in srgb, var(--color-secondary) calc(0.1 * 100%), transparent);
    border: 1px solid color-mix(in srgb, var(--color-secondary) calc(0.6 * 100%), transparent);
    color: var(--color-secondary);
    padding: 0.625rem 1rem;
    border-radius: 10px;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: var(--text-base);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-trash:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-secondary) calc(0.2 * 100%), transparent);
    border-color: var(--color-secondary);
    transform: translateY(-0.5px);
  }
  .btn-trash:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .safer-tag {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    background: color-mix(in srgb, var(--color-secondary) calc(0.25 * 100%), transparent);
    color: var(--color-secondary);
    padding: 1px 4px;
    border-radius: 3px;
    margin-left: 2px;
  }
  .btn-delete {
    background: var(--color-error);
    border: 1px solid color-mix(in srgb, var(--color-error) calc(0.8 * 100%), transparent);
    color: white;
    padding: 0.625rem 1rem;
    border-radius: 10px;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: var(--text-base);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    box-shadow: 0 4px 12px color-mix(in srgb, var(--color-error) calc(0.35 * 100%), transparent);
    transition: all 0.15s;
  }
  .btn-delete:hover:not(:disabled) {
    background: var(--color-error);
    transform: translateY(-0.5px);
  }
  .btn-delete:disabled {
    opacity: 0.4;
    cursor: default;
  }

  /* Cancel Footer */
  .cancel-footer {
    display: flex;
    justify-content: center;
    padding-top: 0.25rem;
  }
  .btn-cancel-keep {
    background: none;
    border: none;
    color: var(--color-outline);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: color 0.15s;
  }
  .btn-cancel-keep:hover {
    color: var(--color-on-surface);
  }
  .kbd-esc {
    background: var(--color-surface-container-high);
    border: 1px solid var(--color-outline-variant);
    color: var(--color-outline);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: var(--text-sm);
  }
</style>

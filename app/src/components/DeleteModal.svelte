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
    background: rgba(11, 14, 20, 0.82);
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
    background: #161B22;
    border: 1px solid rgba(249, 115, 22, 0.45);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.8), 0 0 30px rgba(249, 115, 22, 0.15);
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow: hidden;
  }
  .corner-tag {
    position: absolute;
    top: 8px;
    font-family: var(--font-mono);
    font-size: 0.5rem;
    letter-spacing: 0.08em;
    pointer-events: none;
  }
  .corner-tag.left {
    left: 12px;
    color: rgba(249, 115, 22, 0.4);
  }
  .corner-tag.right {
    right: 48px;
    color: #484F58;
  }
  .ambient-glow {
    position: absolute;
    top: -40px;
    right: -40px;
    width: 140px;
    height: 140px;
    border-radius: 50%;
    background: rgba(249, 115, 22, 0.15);
    filter: blur(30px);
    pointer-events: none;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(88, 66, 55, 0.25);
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
    background: rgba(249, 115, 22, 0.15);
    border: 1px solid rgba(249, 115, 22, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
  }
  .ping-dot {
    position: absolute;
    top: -2px;
    right: -2px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #F97316;
    animation: ping 1.5s cubic-bezier(0, 0, 0.2, 1) infinite;
  }
  @keyframes ping {
    75%, 100% { transform: scale(2); opacity: 0; }
  }
  .header-title {
    font-family: var(--font-sans);
    font-size: 1.125rem;
    font-weight: 700;
    color: #F0F6FC;
    margin: 0;
  }
  .header-subtitle {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: #8B949E;
    margin-top: 2px;
  }
  .esc-close-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #21262D;
    border: 1px solid #30363D;
    color: #8B949E;
    border-radius: 6px;
    padding: 3px 8px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    transition: all 0.15s;
  }
  .esc-close-btn:hover {
    color: #F0F6FC;
    border-color: #8B949E;
  }

  /* Target Box */
  .target-box {
    background: #0B0E14;
    border: 1px solid rgba(88, 66, 55, 0.3);
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
    font-family: var(--font-mono);
    font-size: 0.625rem;
  }
  .target-label {
    color: #8B949E;
    text-transform: uppercase;
    font-weight: 600;
  }
  .resolved-tag {
    color: #4FDCB8;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .resolved-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #4FDCB8;
  }
  .path-display {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(33, 38, 45, 0.5);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid rgba(88, 66, 55, 0.2);
    overflow-x: auto;
  }
  .path-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }
  .path-text {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: #FFB690;
    word-break: break-all;
  }
  .target-meta {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: #8B949E;
    padding-top: 2px;
  }

  /* Bento Grid */
  .bento-grid {
    display: grid;
    grid-template-columns: 5fr 7fr;
    gap: 0.75rem;
  }
  .metric-card {
    background: #101319;
    border: 1px solid rgba(249, 115, 22, 0.35);
    border-radius: 10px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .metric-label {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    font-weight: 600;
    color: #FFB690;
  }
  .metric-value {
    font-family: var(--font-mono);
    font-size: 1.5rem;
    font-weight: 700;
    color: #F97316;
    margin-top: 2px;
  }
  .reclaim-tag {
    font-family: var(--font-sans);
    font-size: 0.6875rem;
    color: #14B8A6;
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
    background: #101319;
    border: 1px solid rgba(88, 66, 55, 0.25);
    padding: 0.35rem 0.625rem;
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
  }
  .chip-label {
    color: #8B949E;
  }
  .chip-value {
    color: #F0F6FC;
    font-weight: 600;
  }
  .chip-value.highlight {
    color: #F97316;
  }

  /* Callout Boxes */
  .warning-callout {
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    padding: 0.75rem;
    display: flex;
    gap: 0.625rem;
  }
  .error-callout {
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid #EF4444;
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
    font-size: 1.125rem;
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
    font-size: 0.75rem;
    color: #EF4444;
  }
  .callout-desc {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: #F87171;
    line-height: 1.4;
    word-break: break-all;
  }

  /* Safety Check */
  .safety-check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-sans);
    font-size: 0.75rem;
    color: #F0F6FC;
    cursor: pointer;
  }
  .safety-check input {
    cursor: pointer;
    accent-color: #F97316;
  }

  /* Action Buttons */
  .actions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    padding-top: 0.25rem;
  }
  .btn-trash {
    background: rgba(20, 184, 166, 0.1);
    border: 1px solid rgba(20, 184, 166, 0.6);
    color: #4FDCB8;
    padding: 0.625rem 1rem;
    border-radius: 10px;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 0.8125rem;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-trash:hover:not(:disabled) {
    background: rgba(20, 184, 166, 0.2);
    border-color: #4FDCB8;
    transform: translateY(-0.5px);
  }
  .btn-trash:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .safer-tag {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    background: rgba(20, 184, 166, 0.25);
    color: #71F8E4;
    padding: 1px 4px;
    border-radius: 3px;
    margin-left: 2px;
  }
  .btn-delete {
    background: #EF4444;
    border: 1px solid rgba(239, 68, 68, 0.8);
    color: white;
    padding: 0.625rem 1rem;
    border-radius: 10px;
    font-family: var(--font-sans);
    font-weight: 600;
    font-size: 0.8125rem;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.35);
    transition: all 0.15s;
  }
  .btn-delete:hover:not(:disabled) {
    background: #DC2626;
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
    color: #8B949E;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: color 0.15s;
  }
  .btn-cancel-keep:hover {
    color: #F0F6FC;
  }
  .kbd-esc {
    background: #21262D;
    border: 1px solid #30363D;
    color: #8B949E;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 0.5625rem;
  }
</style>

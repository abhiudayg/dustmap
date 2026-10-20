<script lang="ts">
  import { appState } from '../lib/store.svelte';

  function close() {
    appState.showPermissionDialog = false;
  }
</script>

{#if appState.showPermissionDialog}
  <div class="backdrop" onclick={close} role="presentation">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Permission Warnings Explained"
      tabindex="-1"
    >
      <div class="modal-header">
        <span class="shield-icon">🛡️</span>
        <h2>macOS Full Disk Access & Privacy Warnings</h2>
      </div>

      <div class="modal-body">
        <div class="stat-highlight">
          <span class="warn-number">{appState.summary?.error_count ?? 0}</span>
          <span class="warn-label">paths skipped due to macOS permissions</span>
        </div>

        <p class="desc">
          macOS enforces <strong>Transparency, Consent, and Control (TCC)</strong>. By default, applications cannot read user-sensitive or system-protected directories, including:
        </p>

        <ul class="protected-list">
          <li><code>~/Library/Messages</code> (iMessage database)</li>
          <li><code>~/Library/Mail</code> (Email archives)</li>
          <li><code>~/Library/Application Support/AddressBook</code> (Contacts)</li>
          <li><code>/Library/Application Support/Apple/AssetCache</code> (System Caches)</li>
          <li>System snapshots and root-only daemons</li>
        </ul>

        <div class="tip-box">
          <p>
            <strong>How to resolve:</strong> DustMap safely skips these folders so your scans never crash. If you want to include these protected folders in your disk analysis, grant <strong>Full Disk Access</strong>:
          </p>
          <ol class="steps-list">
            <li>Open <strong>System Settings → Privacy & Security</strong></li>
            <li>Select <strong>Full Disk Access</strong></li>
            <li>Enable <strong>DustMap</strong> (or click <code>+</code> to add it)</li>
            <li>Click <strong>Rescan</strong> in DustMap</li>
          </ol>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn-done" onclick={close}>Understood</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(14, 17, 23, 0.85);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #161B22;
    border: 1px solid #30363D;
    border-radius: 12px;
    width: 540px;
    max-width: 92vw;
    padding: 1.5rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
  }
  .modal-header {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    margin-bottom: 1rem;
  }
  .shield-icon {
    font-size: 1.5rem;
  }
  h2 {
    font-size: 1.125rem;
    font-weight: 700;
    color: #F0F6FC;
  }
  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
  }
  .stat-highlight {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid rgba(245, 158, 11, 0.3);
    padding: 0.625rem 0.875rem;
    border-radius: 6px;
  }
  .warn-number {
    font-size: 1.25rem;
    font-weight: 700;
    color: #F59E0B;
    font-family: 'JetBrains Mono', monospace;
  }
  .warn-label {
    font-size: 0.8125rem;
    color: #F0F6FC;
  }
  .desc {
    font-size: 0.8125rem;
    color: #8B949E;
    line-height: 1.5;
  }
  .desc strong {
    color: #F0F6FC;
  }
  .protected-list {
    margin-left: 1.25rem;
    font-size: 0.75rem;
    color: #8B949E;
    line-height: 1.6;
  }
  .protected-list code {
    font-family: 'JetBrains Mono', monospace;
    background: #0E1117;
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    border: 1px solid #21262D;
    color: #F0F6FC;
  }
  .tip-box {
    background: #0E1117;
    border: 1px solid #21262D;
    border-radius: 6px;
    padding: 0.75rem 0.875rem;
    font-size: 0.78125rem;
    color: #8B949E;
    line-height: 1.5;
  }
  .tip-box strong {
    color: #14B8A6;
  }
  .steps-list {
    margin-left: 1.25rem;
    margin-top: 0.375rem;
    color: #F0F6FC;
  }
  .steps-list code {
    background: #21262D;
    padding: 0.1rem 0.25rem;
    border-radius: 3px;
  }
  .modal-actions {
    margin-top: 1.25rem;
    display: flex;
    justify-content: flex-end;
  }
  .btn-done {
    background: #14B8A6;
    color: #0E1117;
    border: none;
    font-weight: 600;
    font-size: 0.8125rem;
    padding: 0.45rem 1.125rem;
    border-radius: 6px;
    cursor: pointer;
    transition: opacity 0.15s;
  }
  .btn-done:hover {
    opacity: 0.9;
  }
</style>

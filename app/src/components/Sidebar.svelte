<script lang="ts">
  import type { View } from '../lib/types';

  let { view, onselect }: { view: View | null; onselect: (view: View) => void } = $props();

  const ITEMS: { id: View; label: string; icon: string }[] = [
    { id: 'explorer', label: 'Overview', icon: 'dashboard' },
    { id: 'cleanup', label: 'Cleanup', icon: 'cleaning_services' },
    { id: 'remnants', label: 'App Cleanups', icon: 'apps' },
    { id: 'data', label: 'Your Data', icon: 'database' },
  ];
</script>

<nav class="topnav" aria-label="Main navigation">
  {#each ITEMS as item (item.id)}
    <button class:active={view === item.id} aria-current={view === item.id ? 'page' : undefined} onclick={() => onselect(item.id)}>
      <span class="material-symbols-outlined" aria-hidden="true">{item.icon}</span>
      {item.label}
    </button>
  {/each}
</nav>

<style>
  .topnav {
    display: flex;
    gap: 2px;
    align-items: center;
    flex-shrink: 0;
  }
  .topnav button {
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: none;
    color: var(--color-on-surface-variant);
    font: inherit;
    font-size: var(--text-base);
    white-space: nowrap;
    cursor: pointer;
  }
  .topnav button.active {
    background: var(--color-surface-container-high);
    border-color: var(--color-outline-variant);
    color: var(--color-on-surface);
  }
</style>

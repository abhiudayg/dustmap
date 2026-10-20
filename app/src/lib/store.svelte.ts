import type { VolumeInfo, IpcEntry, IpcScanSummary } from './types';
import { listVolumes, startScan, scanSummary, listChildren, listCacheEntries, revealInFinder, saveReportFile, deletePath as invokeDelete } from './invoke';

export interface BreadcrumbItem {
  id: number;
  name: string;
}

export interface CachedVolumeScan {
  scanId: string;
  summary: IpcScanSummary;
  currentChildren: IpcEntry[];
  breadcrumb: BreadcrumbItem[];
  currentEntryId: number;
  cacheEntries?: IpcEntry[];
}

export const appState = $state({
  volumes: [] as VolumeInfo[],
  currentVolume: null as VolumeInfo | null,
  scanId: null as string | null,
  summary: null as IpcScanSummary | null,
  currentEntryId: 0,
  currentChildren: [] as IpcEntry[],
  breadcrumb: [{ id: 0, name: '/' }] as BreadcrumbItem[],
  isScanning: false,
  scanError: null as string | null,
  deleteTarget: null as { path: string; entry: IpcEntry } | null,
  sortKey: 'size' as 'name' | 'size' | 'modified',
  sortDir: 'desc' as 'asc' | 'desc',
  metric: 'allocated' as 'allocated' | 'logical',
  useBinaryUnits: false,
  showPermissionDialog: false,
  showSplash: true,
  volumeCache: {} as Record<string, CachedVolumeScan>,
  searchQuery: '',
  activeFilter: 'all' as 'all' | 'cache' | 'apps',
  viewMode: 'donut' as 'donut' | 'treemap',
  cacheEntries: [] as IpcEntry[],
  history: [] as { entryId: number; breadcrumb: BreadcrumbItem[] }[],
  historyIdx: 0,
  toastMessage: null as string | null,
});

export function getEntrySize(entry: IpcEntry): number {
  if (appState.metric === 'allocated') {
    return entry.total_allocated ?? entry.allocated_bytes ?? entry.total_logical ?? entry.logical_bytes ?? 0;
  }
  return entry.total_logical ?? entry.logical_bytes ?? 0;
}

export function toggleMetric() {
  appState.metric = appState.metric === 'allocated' ? 'logical' : 'allocated';
}

export function toggleUnits() {
  appState.useBinaryUnits = !appState.useBinaryUnits;
}

export async function loadVolumes() {
  try {
    appState.volumes = await listVolumes();
    if (appState.volumes.length > 0 && !appState.currentVolume) {
      appState.currentVolume = appState.volumes[0];
      // Do NOT trigger an auto-scan! Show the clean dashboard overview first.
      appState.isScanning = false;
      appState.breadcrumb = [{ id: 0, name: appState.volumes[0].name || '/' }];
    }
  } catch (e) {
    console.error('Failed to load volumes:', e);
  }
}

export async function selectVolume(vol: VolumeInfo, force = false) {
  // If already active and cached, do nothing
  if (!force && appState.currentVolume?.mount_point === vol.mount_point && appState.scanId) {
    return;
  }

  appState.currentVolume = vol;

  // If already in cache, switch immediately without scanning!
  if (!force && appState.volumeCache[vol.mount_point]) {
    const cached = appState.volumeCache[vol.mount_point];
    appState.scanId = cached.scanId;
    appState.summary = cached.summary;
    appState.currentChildren = [...cached.currentChildren];
    appState.breadcrumb = [...cached.breadcrumb];
    appState.currentEntryId = cached.currentEntryId;
    appState.cacheEntries = cached.cacheEntries ? [...cached.cacheEntries] : [];
    appState.history = [{ entryId: cached.currentEntryId, breadcrumb: [...cached.breadcrumb] }];
    appState.historyIdx = 0;
    return;
  }

  // Not in cache yet: reset scan state so the new disk's clean dashboard appears
  // and the previous disk's children/consumers are not shown!
  appState.scanId = null;
  appState.summary = null;
  appState.currentChildren = [];
  appState.cacheEntries = [];
  appState.breadcrumb = [{ id: 0, name: vol.name || vol.mount_point }];
  appState.currentEntryId = 0;
  appState.history = [{ entryId: 0, breadcrumb: [{ id: 0, name: vol.name || vol.mount_point }] }];
  appState.historyIdx = 0;

  if (force) {
    await doScan(vol.mount_point);
  }
}

export async function startCurrentVolumeScan() {
  if (appState.currentVolume) {
    await doScan(appState.currentVolume.mount_point);
  }
}

export async function doScan(path: string) {
  appState.isScanning = true;
  appState.scanError = null;
  appState.summary = null;
  appState.scanId = null;
  appState.currentChildren = [];
  appState.breadcrumb = [{ id: 0, name: path.split('/').filter(Boolean).pop() || '/' }];
  appState.currentEntryId = 0;
  try {
    const id = await startScan(path);
    appState.scanId = id;
    const summary = await scanSummary(id);
    appState.summary = summary;
    await loadChildren(id, 0);

    // Save scan to cache so clicking the volume again is instant
    appState.volumeCache[path] = {
      scanId: id,
      summary,
      currentChildren: [...appState.currentChildren],
      breadcrumb: [...appState.breadcrumb],
      currentEntryId: 0,
    };
  } catch (e: unknown) {
    appState.scanError = e instanceof Error ? e.message : String(e);
  } finally {
    appState.isScanning = false;
  }
}

export async function rescanCurrentVolume() {
  if (appState.currentVolume) {
    delete appState.volumeCache[appState.currentVolume.mount_point];
    await doScan(appState.currentVolume.mount_point);
  }
}

export async function loadChildren(scanId: string, entryId: number) {
  try {
    const resp = await listChildren(scanId, entryId);
    appState.currentChildren = resp.entries;
  } catch (e) {
    console.error('Failed to list children:', e);
  }
}

export async function fetchCacheEntries() {
  if (!appState.scanId) return;
  try {
    const entries = await listCacheEntries(appState.scanId);
    appState.cacheEntries = entries;
    if (appState.currentVolume && appState.volumeCache[appState.currentVolume.mount_point]) {
      appState.volumeCache[appState.currentVolume.mount_point].cacheEntries = [...entries];
    }
  } catch (e) {
    console.error('Failed to list cache entries:', e);
  }
}

export async function exportReport() {
  const vol = appState.currentVolume || appState.volumes[0];
  const summary = appState.summary || (vol ? appState.volumeCache[vol.mount_point]?.summary : null);

  const report = {
    app: 'DustMap',
    version: '0.1.0',
    exported_at: new Date().toISOString(),
    active_volume: {
      name: vol?.name ?? 'Macintosh HD',
      mount_point: vol?.mount_point ?? '/',
      fs_type: vol?.fs_type ?? 'APFS',
      total_bytes: vol?.total_bytes ?? 0,
      used_bytes: vol?.used_bytes ?? 0,
      available_bytes: vol?.available_bytes ?? 0,
    },
    scan_summary: summary
      ? {
          root: summary.root,
          total_items: summary.total_items,
          total_logical_bytes: summary.total_logical,
          total_allocated_bytes: summary.total_allocated,
          duration_seconds: summary.duration_secs,
          error_count: summary.error_count,
        }
      : 'Initial volume overview (full scan not yet completed)',
    top_consumers: appState.currentChildren.slice(0, 50).map(e => ({
      name: e.name,
      path: e.path || e.name,
      kind: e.kind,
      size_bytes: e.total_allocated ?? e.allocated_bytes ?? e.total_logical ?? e.logical_bytes ?? 0,
      is_hidden: e.isHidden,
    })),
    connected_volumes: appState.volumes.map(v => ({
      name: v.name,
      mount_point: v.mount_point,
      total_bytes: v.total_bytes,
      used_bytes: v.used_bytes,
    })),
  };

  const jsonStr = JSON.stringify(report, null, 2);
  const dateStr = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const volName = (vol?.name || 'Volume').replace(/[^a-zA-Z0-9]/g, '_');
  const filename = `DustMap-Report-${volName}-${dateStr}.json`;

  let exportedPath = '';
  try {
    exportedPath = await saveReportFile(jsonStr, filename);
  } catch (err) {
    console.warn('Native save failed, falling back to browser blob download:', err);
    try {
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (_) {}
  }

  // Also copy to clipboard as convenient fallback
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(jsonStr);
    }
  } catch (_) {}

  // Trigger feedback toast notification
  appState.toastMessage = exportedPath
    ? `Saved to ~/Downloads & revealed in Finder: ${filename}`
    : `Report generated: ${filename} (copied to clipboard)`;

  setTimeout(() => {
    appState.toastMessage = null;
  }, 4500);
}

export async function navigateTo(entry: IpcEntry) {
  if (entry.kind !== 'Directory' || !appState.scanId) return;
  appState.currentEntryId = entry.id;
  const newBreadcrumb = [...appState.breadcrumb, { id: entry.id, name: entry.name }];
  appState.breadcrumb = newBreadcrumb;
  
  // Push to history
  appState.history = appState.history.slice(0, appState.historyIdx + 1);
  appState.history.push({ entryId: entry.id, breadcrumb: [...newBreadcrumb] });
  appState.historyIdx = appState.history.length - 1;

  await loadChildren(appState.scanId, entry.id);
}

export async function navigateToBreadcrumb(idx: number) {
  if (!appState.scanId) return;
  const crumb = appState.breadcrumb[idx];
  const newBreadcrumb = appState.breadcrumb.slice(0, idx + 1);
  appState.breadcrumb = newBreadcrumb;
  appState.currentEntryId = crumb.id;

  appState.history = appState.history.slice(0, appState.historyIdx + 1);
  appState.history.push({ entryId: crumb.id, breadcrumb: [...newBreadcrumb] });
  appState.historyIdx = appState.history.length - 1;

  await loadChildren(appState.scanId, crumb.id);
}

export async function goBack() {
  if (appState.historyIdx > 0 && appState.scanId) {
    appState.historyIdx -= 1;
    const item = appState.history[appState.historyIdx];
    appState.currentEntryId = item.entryId;
    appState.breadcrumb = [...item.breadcrumb];
    await loadChildren(appState.scanId, item.entryId);
  } else if (appState.breadcrumb.length > 1 && appState.scanId) {
    await navigateToBreadcrumb(appState.breadcrumb.length - 2);
  }
}

export async function goForward() {
  if (appState.historyIdx < appState.history.length - 1 && appState.scanId) {
    appState.historyIdx += 1;
    const item = appState.history[appState.historyIdx];
    appState.currentEntryId = item.entryId;
    appState.breadcrumb = [...item.breadcrumb];
    await loadChildren(appState.scanId, item.entryId);
  }
}

export async function revealPath(path: string) {
  try {
    await revealInFinder(path);
  } catch (e) {
    console.error('Failed to reveal in Finder:', e);
  }
}

export function setDeleteTarget(path: string, entry: IpcEntry) {
  appState.deleteTarget = { path, entry };
}

export function clearDeleteTarget() {
  appState.deleteTarget = null;
}

export async function performDelete(path: string, permanent: boolean, entryId?: number) {
  const result = await invokeDelete(path, permanent, appState.scanId, entryId, appState.currentEntryId);
  if (!result.success) {
    throw new Error(result.error ?? 'Delete operation was rejected by macOS (permission denied or protected path)');
  }
  clearDeleteTarget();
  if (appState.scanId) {
    await loadChildren(appState.scanId, appState.currentEntryId);
  }
  return result;
}

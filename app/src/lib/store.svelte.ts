import type { VolumeInfo, IpcEntry, IpcScanSummary } from './types';
import { listVolumes, startScan, scanSummary, listChildren, listCacheEntries, revealInFinder, deletePath as invokeDelete } from './invoke';

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
  if (!appState.summary) {
    alert('No scan summary available to export. Please scan a volume first.');
    return;
  }
  const report = {
    volume: appState.currentVolume?.name ?? 'Volume',
    mount_point: appState.summary.root,
    total_bytes: appState.summary.total_allocated,
    items_count: appState.summary.total_items,
    scan_duration_secs: appState.summary.duration_secs,
    errors_encountered: appState.summary.error_count,
    exported_at: new Date().toISOString(),
    top_entries: appState.currentChildren.slice(0, 25).map(e => ({
      name: e.name,
      bytes: e.total_allocated ?? e.allocated_bytes ?? e.logical_bytes,
      kind: e.kind,
      path: e.path,
    })),
  };
  try {
    await navigator.clipboard.writeText(JSON.stringify(report, null, 2));
    alert('Telemetry & Disk Space Report copied to clipboard!');
  } catch (e) {
    console.error('Failed to copy report to clipboard:', e);
  }
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

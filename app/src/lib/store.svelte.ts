import type { VolumeInfo, IpcEntry, IpcScanSummary } from './types';
import { listVolumes, startScan, scanSummary, listChildren, deletePath as invokeDelete } from './invoke';

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
    appState.currentChildren = cached.currentChildren;
    appState.breadcrumb = cached.breadcrumb;
    appState.currentEntryId = cached.currentEntryId;
    return;
  }

  // If not in cache and user clicked, start the scan
  if (force || !appState.scanId) {
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

export async function navigateTo(entry: IpcEntry) {
  if (entry.kind !== 'Directory' || !appState.scanId) return;
  appState.currentEntryId = entry.id;
  appState.breadcrumb = [...appState.breadcrumb, { id: entry.id, name: entry.name }];
  await loadChildren(appState.scanId, entry.id);
}

export async function navigateToBreadcrumb(idx: number) {
  if (!appState.scanId) return;
  const crumb = appState.breadcrumb[idx];
  appState.breadcrumb = appState.breadcrumb.slice(0, idx + 1);
  appState.currentEntryId = crumb.id;
  await loadChildren(appState.scanId, crumb.id);
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

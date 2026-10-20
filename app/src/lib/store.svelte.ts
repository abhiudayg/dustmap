import type { VolumeInfo, IpcEntry, IpcScanSummary } from './types';
import { listVolumes, startScan, scanSummary, listChildren, deletePath as invokeDelete } from './invoke';

export interface BreadcrumbItem {
  id: number;
  name: string;
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
});

export async function loadVolumes() {
  try {
    appState.volumes = await listVolumes();
    if (appState.volumes.length > 0 && !appState.currentVolume) {
      appState.currentVolume = appState.volumes[0];
    }
  } catch (e) {
    console.error('Failed to load volumes:', e);
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
  } catch (e: unknown) {
    appState.scanError = e instanceof Error ? e.message : String(e);
  } finally {
    appState.isScanning = false;
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

export async function performDelete(path: string, permanent: boolean) {
  const result = await invokeDelete(path, permanent);
  clearDeleteTarget();
  if (!result.success) throw new Error(result.error ?? 'Delete failed');
  if (appState.scanId) {
    await loadChildren(appState.scanId, appState.currentEntryId);
  }
  return result;
}

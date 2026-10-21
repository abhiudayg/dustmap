import { invoke } from '@tauri-apps/api/core';
import type { VolumeInfo, IpcScanSummary, IpcChildrenResponse, IpcDeleteResult, IpcClearResult, IpcEntry, StartupReport, CleanupPreview, CleanupResult, Remnant, DataEntry, CachedScanInfo } from './types';

export async function listVolumes(): Promise<VolumeInfo[]> {
  return invoke<VolumeInfo[]>('list_volumes_cmd');
}

export async function startScan(path: string): Promise<string> {
  return invoke<string>('start_scan', { path });
}

export async function scanSummary(scanId: string): Promise<IpcScanSummary> {
  return invoke<IpcScanSummary>('scan_summary', { scanId });
}

export async function listChildren(scanId: string, entryId: number): Promise<IpcChildrenResponse> {
  return invoke<IpcChildrenResponse>('list_children', { scanId, entryId });
}

export async function listCacheEntries(scanId: string): Promise<IpcEntry[]> {
  return invoke<IpcEntry[]>('list_cache_entries', { scanId });
}

export async function revealInFinder(path: string): Promise<void> {
  return invoke<void>('reveal_in_finder', { path });
}

export async function saveReportFile(content: string, filename: string): Promise<string> {
  return invoke<string>('save_report_file', { content, filename });
}

export async function deletePath(
  path: string,
  permanent: boolean,
  scanId?: string | null,
  entryId?: number | null,
): Promise<IpcDeleteResult> {
  return invoke<IpcDeleteResult>('delete_path', {
    path,
    permanent,
    scanId: scanId ?? null,
    entryId: entryId ?? null,
  });
}

export async function clearFolderContents(
  path: string,
  permanent: boolean,
  scanId?: string | null,
  entryId?: number | null,
): Promise<IpcClearResult> {
  return invoke<IpcClearResult>('clear_folder_contents', {
    path,
    permanent,
    scanId: scanId ?? null,
    entryId: entryId ?? null,
  });
}

export async function startupReport(): Promise<StartupReport> {
  return invoke<StartupReport>('startup_report');
}

export async function cleanupPreview(): Promise<CleanupPreview> {
  return invoke<CleanupPreview>('cleanup_preview');
}

export async function cleanupExecute(ruleIds: string[], confirmed: boolean): Promise<CleanupResult[]> {
  return invoke<CleanupResult[]>('cleanup_execute', { ruleIds, confirmed });
}

export async function remnantSearch(bundleId: string): Promise<Remnant[]> {
  return invoke<Remnant[]>('remnant_search', { bundleId });
}

export async function remnantRemove(bundleId: string, paths: string[], confirmCaution: boolean): Promise<string[]> {
  return invoke<string[]>('remnant_remove', { bundleId, paths, confirmCaution });
}

export async function dataList(): Promise<DataEntry[]> {
  return invoke<DataEntry[]>('data_list');
}

export async function dataRemove(labels: string[]): Promise<string[]> {
  return invoke<string[]>('data_remove', { labels });
}

export async function acknowledgeQuarantine(): Promise<void> {
  return invoke<void>('acknowledge_quarantine');
}

export async function cancelScan(): Promise<void> {
  return invoke<void>('cancel_scan');
}

export async function dismissInterruptedScan(): Promise<void> {
  return invoke<void>('dismiss_interrupted_scan');
}

export async function loadCachedScan(path: string): Promise<CachedScanInfo | null> {
  return invoke<CachedScanInfo | null>('load_cached_scan', { path });
}

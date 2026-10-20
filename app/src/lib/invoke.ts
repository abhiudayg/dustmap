import { invoke } from '@tauri-apps/api/core';
import type { VolumeInfo, IpcScanSummary, IpcChildrenResponse, IpcDeleteResult, IpcEntry } from './types';

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
  parentId?: number | null,
): Promise<IpcDeleteResult> {
  return invoke<IpcDeleteResult>('delete_path', {
    path,
    permanent,
    scanId: scanId ?? null,
    entryId: entryId ?? null,
    parentId: parentId ?? null,
  });
}

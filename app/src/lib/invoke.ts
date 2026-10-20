import { invoke } from '@tauri-apps/api/core';
import type { VolumeInfo, IpcScanSummary, IpcChildrenResponse, IpcDeleteResult } from './types';

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

export async function deletePath(path: string, permanent: boolean): Promise<IpcDeleteResult> {
  return invoke<IpcDeleteResult>('delete_path', { path, permanent });
}

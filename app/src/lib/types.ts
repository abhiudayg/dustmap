export interface VolumeInfo {
  name: string;
  mount_point: string;
  fs_type: string;
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  is_removable: boolean;
  is_read_only: boolean;
}

export interface IpcEntry {
  id: number;
  name: string;
  kind: 'File' | 'Directory' | 'Symlink' | 'Other';
  logical_bytes: number;
  allocated_bytes: number | null;
  is_hidden: boolean;
  modified_secs: number;
  coverage: 'Complete' | 'Partial' | 'NotScanned';
  child_count?: number;
  total_logical?: number;
  total_allocated?: number;
}

export interface IpcScanSummary {
  scan_id: string;
  root: string;
  total_logical: number;
  total_allocated: number;
  total_items: number;
  error_count: number;
  duration_secs: number;
  is_partial: boolean;
}

export interface IpcChildrenResponse {
  parent_id: number;
  entries: IpcEntry[];
  total_count: number;
}

export interface IpcDeleteResult {
  path: string;
  success: boolean;
  error?: string;
}

export type SortKey = 'name' | 'size' | 'modified';
export type SortDir = 'asc' | 'desc';

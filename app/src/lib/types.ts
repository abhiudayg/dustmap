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
  path: string;
  name: string;
  kind: 'File' | 'Directory' | 'Symlink' | 'Other';
  logical_bytes: number;
  allocated_bytes: number | null;
  is_hidden: boolean;
  is_sparse: boolean;
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

export type RiskLevel = 'safe' | 'review' | 'caution' | 'report-only';

export interface RuleInfo {
  id: string;
  name: string;
  category: string;
  risk: RiskLevel;
  summary: string;
  consequence: string;
  undo: string;
}

export interface CleanupTarget {
  rule_id: string;
  risk: RiskLevel;
  path: string;
  logical_bytes: number;
}

export interface ToolTarget {
  rule_id: string;
  risk: RiskLevel;
  description: string;
}

export interface RejectedTarget {
  label: string;
  reason: string;
}

export interface CleanupPreview {
  rules: RuleInfo[];
  targets: CleanupTarget[];
  tool_targets: ToolTarget[];
  rejected: RejectedTarget[];
  estimated_bytes: number;
}

export interface CleanupResult {
  item_id: string;
  rule_id: string;
  path: string | null;
  outcome: string;
  detail: string | null;
}

export interface Remnant {
  path: string;
  location: string;
  risk: RiskLevel;
  note: string;
  match_kind: string;
}

export interface DataEntry {
  label: string;
  path: string;
  bytes: number;
}

export interface StartupReport {
  interrupted_scan: string | null;
  items_needing_review: string[];
  quarantined_journals: string[];
  errors: string[];
}

export type View = 'explorer' | 'cleanup' | 'remnants' | 'data';

export interface ScanProgressInfo {
  entries_seen: number;
  bytes_accounted: number;
  errors: number;
  elapsed_secs: number;
  current_path: string;
}

export interface CachedScanInfo {
  scan_id: string;
  scanned_at_secs: number;
}

export function formatBytes(bytes?: number | null): string {
  if (bytes === undefined || bytes === null || isNaN(bytes) || bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export function formatDate(secs: number): string {
  return new Date(secs * 1000).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function usagePercent(used: number, total: number): number {
  return total > 0 ? Math.min(100, Math.round((used / total) * 100)) : 0;
}

export const CATEGORY_COLORS: Record<string, string> = {
  Applications: '#F97316',
  Users: '#14B8A6',
  Library: '#8B5CF6',
  Developer: '#F59E0B',
  System: '#64748B',
  Downloads: '#3B82F6',
  '.Trash': '#EF4444',
  default: '#475569',
};

export function colorForEntry(name: string, isHidden: boolean): string {
  if (isHidden) return '#991B1B';
  return CATEGORY_COLORS[name] ?? CATEGORY_COLORS.default;
}

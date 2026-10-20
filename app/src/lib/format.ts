export function formatBytes(bytes?: number | null, useBinary = false): string {
  if (bytes === undefined || bytes === null || isNaN(bytes) || bytes <= 0) return '0 B';
  const base = useBinary ? 1024 : 1000;
  const units = useBinary
    ? ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB']
    : ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(base)));
  const val = bytes / Math.pow(base, i);
  return `${val >= 100 || i === 0 ? val.toFixed(0) : val.toFixed(1)} ${units[i]}`;
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
  Applications: '#F97316', // Amber-Orange
  Users: '#04B4A2',        // APFS Teal
  Library: '#A985FF',      // Telemetry Violet
  Developer: '#FFB690',    // Peach Glow
  System: '#334155',       // Deep Slate
  Downloads: '#38BDF8',    // Sky Cyan
  '.Trash': '#EF4444',     // Crimson
  default: '#272A31',
};

export function colorForEntry(name: string, isHidden: boolean): string {
  if (isHidden) return '#991B1B';
  return CATEGORY_COLORS[name] ?? CATEGORY_COLORS.default;
}

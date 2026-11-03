const LOCALE = 'pt-BR';

/** `1536` → `1,5 KB`. */
export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = unit === 0 ? 0 : 1;
  const number = new Intl.NumberFormat(LOCALE, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(value);
  return `${number} ${units[unit] ?? 'B'}`;
}

/** `2026-11-02_15-30-00` (nome do backup) → `02/11/2026 15:30:00`; texto fora do padrão volta igual. */
export function formatStamp(stamp: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})/.exec(stamp);
  if (!match) return stamp;
  const [, year, month, day, hour, minute, second] = match;
  return `${day ?? ''}/${month ?? ''}/${year ?? ''} ${hour ?? ''}:${minute ?? ''}:${second ?? ''}`;
}

/** Segundos desde 1970 → data e hora locais (`timeZone` existe para os testes). */
export function formatEpoch(seconds: number, timeZone?: string): string {
  return new Intl.DateTimeFormat(LOCALE, {
    dateStyle: 'short',
    timeStyle: 'short',
    ...(timeZone ? { timeZone } : {}),
  }).format(new Date(seconds * 1000));
}

/** `5374` → `5.374`. */
export function formatCount(count: number): string {
  return new Intl.NumberFormat(LOCALE).format(count);
}

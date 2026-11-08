import { getLocale } from './i18n';

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
  const number = new Intl.NumberFormat(getLocale(), {
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
  return new Intl.DateTimeFormat(getLocale(), {
    dateStyle: 'short',
    timeStyle: 'short',
    ...(timeZone ? { timeZone } : {}),
  }).format(new Date(seconds * 1000));
}

/** Segundos desde 1970 → só a data local (`02/11/2026`). */
export function formatDate(seconds: number, timeZone?: string): string {
  return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'short', ...(timeZone ? { timeZone } : {}) }).format(
    new Date(seconds * 1000),
  );
}

/** Valor em reais (`15` → `R$ 15,00`). */
export function formatMoney(value: number): string {
  return new Intl.NumberFormat(getLocale(), { style: 'currency', currency: 'BRL', currencyDisplay: 'narrowSymbol' }).format(value);
}

/** `5374` → `5.374`. */
export function formatCount(count: number): string {
  return new Intl.NumberFormat(getLocale()).format(count);
}

/** Número com casas decimais no idioma do app (não no do Windows): `5.6` → `5,6` em português, `5.6` em inglês. */
export function formatDecimal(value: number, digits = 2): string {
  return new Intl.NumberFormat(getLocale(), { maximumFractionDigits: digits }).format(value);
}

import type { EquipmentItem, EquipmentKind } from './types';

/** Categorias na ordem em que a página as mostra. */
export const KINDS: EquipmentKind[] = ['weapon', 'ammo', 'sight', 'optic', 'lure_caller', 'deployable', 'consumable', 'vehicle'];

export type OwnFilter = 'all' | 'owned' | 'missing';
export type EquipTab = 'arsenal' | 'ranking' | 'loadouts' | 'advisor';
export type RankKey = 'kills' | 'accuracy' | 'record' | 'distance' | 'shots';

export const RANK_KEYS: RankKey[] = ['kills', 'accuracy', 'record', 'distance', 'shots'];

/** Maior dinheiro que o app grava (igual ao limite do núcleo). */
export const MAX_CASH = 10_000_000;

/** Maior quantidade que o app grava de um item da categoria (1: item que não se empilha). */
export function maxAmount(kind: EquipmentKind): number {
  switch (kind) {
    case 'ammo':
      return 999;
    case 'consumable':
    case 'lure_caller':
    case 'deployable':
      return 99;
    default:
      return 1;
  }
}

export interface Progress {
  owned: number;
  total: number;
}

/** Itens de uma categoria, filtrados por posse. */
export function visibleItems(items: EquipmentItem[], kind: EquipmentKind, own: OwnFilter): EquipmentItem[] {
  return items.filter((item) => {
    if (item.kind !== kind) return false;
    if (own === 'owned') return item.owned;
    if (own === 'missing') return !item.owned;
    return true;
  });
}

/** Quantos itens de uma categoria cabem em cada filtro de posse (mostrados ao lado dos botões). */
export function ownCounts(items: EquipmentItem[], kind: EquipmentKind): Record<OwnFilter, number> {
  const list = items.filter((item) => item.kind === kind);
  const owned = list.filter((item) => item.owned).length;
  return { all: list.length, owned, missing: list.length - owned };
}

export interface ItemGroup {
  /** Classe de arma que o grupo representa; `null` quando a categoria não se divide. */
  key: EquipmentItem['class'];
  items: EquipmentItem[];
}

const CLASS_ORDER = ['rifle', 'handgun', 'shotgun', 'archery'];

/**
 * Divide os itens por classe de arma (rifles, pistolas, espingardas e arcos) nas categorias em que isso existe.
 * Dentro de cada grupo, o que o jogador tem vem primeiro.
 */
export function groupItems(items: EquipmentItem[]): ItemGroup[] {
  const byOwned = (a: EquipmentItem, b: EquipmentItem): number => Number(b.owned) - Number(a.owned) || a.name.localeCompare(b.name);
  if (!items.some((item) => item.class)) return items.length === 0 ? [] : [{ key: null, items: [...items].sort(byOwned) }];
  const groups = new Map<EquipmentItem['class'], EquipmentItem[]>();
  for (const item of items) groups.set(item.class, [...(groups.get(item.class) ?? []), item]);
  return [...groups.entries()]
    .sort(([a], [b]) => CLASS_ORDER.indexOf(a ?? '') - CLASS_ORDER.indexOf(b ?? ''))
    .map(([key, list]) => ({ key, items: list.sort(byOwned) }));
}

/** Quantos itens o jogador tem em cada categoria. */
export function collection(items: EquipmentItem[]): Record<EquipmentKind, Progress> {
  const progress = Object.fromEntries(KINDS.map((kind) => [kind, { owned: 0, total: 0 }])) as Record<EquipmentKind, Progress>;
  for (const item of items) {
    progress[item.kind].total += 1;
    if (item.owned) progress[item.kind].owned += 1;
  }
  return progress;
}

/** Valor usado para ordenar uma arma; `null` quando a arma não tem o dado. */
export function rankValue(item: EquipmentItem, key: RankKey): number | null {
  const stats = item.stats;
  if (!stats || stats.shotsFired === 0) return null;
  switch (key) {
    case 'kills':
      return stats.kills;
    case 'accuracy':
      return stats.accuracy;
    case 'record':
      return stats.recordScore;
    case 'distance':
      return stats.furthestShot;
    case 'shots':
      return stats.shotsFired;
  }
}

/** Armas já usadas, da melhor para a pior no critério escolhido. */
export function rankWeapons(items: EquipmentItem[], key: RankKey): EquipmentItem[] {
  return items
    .filter((item) => item.kind === 'weapon' && rankValue(item, key) !== null)
    .sort((a, b) => (rankValue(b, key) ?? 0) - (rankValue(a, key) ?? 0) || a.name.localeCompare(b.name));
}

/** Itens de uma lista de ids, na ordem dada, ignorando o que o catálogo não tem. */
export function itemsById(items: EquipmentItem[], ids: string[]): EquipmentItem[] {
  const byId = new Map(items.map((item) => [item.id, item]));
  return ids.flatMap((id) => {
    const item = byId.get(id);
    return item ? [item] : [];
  });
}

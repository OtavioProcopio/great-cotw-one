// Cálculos puros do mapa: horários, filtros, projeção e zoom. Sem DOM, para testar fácil.
import type { MapGroup, MapZone, ReserveMap } from './types';

/** 1 comer, 2 beber, 3 descansar. */
export type Need = 1 | 2 | 3;

export const NEEDS: readonly Need[] = [1, 2, 3];

export interface Filters {
  needs: ReadonlySet<number>;
  species: ReadonlySet<number>;
  showPending: boolean;
  /** `null` = qualquer horário. */
  hour: number | null;
}

/** `3.5` → `03:30`. */
export function formatHour(hour: number): string {
  const total = Math.round(hour * 60) % (24 * 60);
  const h = Math.floor(total / 60);
  const m = total % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
}

/** `4, 8` → `04:00 – 08:00`. */
export function slotText(start: number, end: number): string {
  return `${formatHour(start)} – ${formatHour(end)}`;
}

/** O horário `hour` cai na janela `start`–`end`? A janela pode passar da meia-noite (20 → 0). */
export function inSlot(start: number, end: number, hour: number): boolean {
  if (start === end) return true;
  return start < end ? hour >= start && hour < end : hour >= start || hour < end;
}

/** Zonas que passam nos filtros. */
export function visibleZones(zones: readonly MapZone[], filters: Filters): MapZone[] {
  return zones.filter(
    (zone) =>
      filters.needs.has(zone.kind) &&
      (zone.species < 0 || filters.species.has(zone.species)) &&
      (!zone.pending || filters.showPending) &&
      (filters.hour === null || inSlot(zone.start, zone.end, filters.hour)),
  );
}

export interface View {
  /** Pixels por metro. */
  scale: number;
  /** Deslocamento em pixels: `tela = mundo * scale + offset`. */
  x: number;
  y: number;
}

export const MIN_SCALE = 0.01;
export const MAX_SCALE = 3;

export function worldToScreen(view: View, x: number, z: number): [number, number] {
  return [x * view.scale + view.x, z * view.scale + view.y];
}

export function screenToWorld(view: View, px: number, py: number): [number, number] {
  return [(px - view.x) / view.scale, (py - view.y) / view.scale];
}

export interface Bounds {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
}

/** Caixa que envolve as zonas; sem zonas, o quadrado da imagem do mapa. */
export function boundsOf(map: ReserveMap): Bounds {
  if (map.zones.length === 0) {
    return { minX: map.x0, maxX: map.x0 + map.span, minZ: map.z0, maxZ: map.z0 + map.span };
  }
  let [minX, maxX, minZ, maxZ] = [Infinity, -Infinity, Infinity, -Infinity];
  for (const zone of map.zones) {
    minX = Math.min(minX, zone.x);
    maxX = Math.max(maxX, zone.x);
    minZ = Math.min(minZ, zone.z);
    maxZ = Math.max(maxZ, zone.z);
  }
  return { minX, maxX, minZ, maxZ };
}

/** Enquadra `bounds` na área de `width` x `height` pixels, com folga. */
export function fitView(bounds: Bounds, width: number, height: number, margin = 400): View {
  const w = bounds.maxX - bounds.minX + margin * 2;
  const h = bounds.maxZ - bounds.minZ + margin * 2;
  const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, Math.min(width / w, height / h)));
  const cx = (bounds.minX + bounds.maxX) / 2;
  const cz = (bounds.minZ + bounds.maxZ) / 2;
  return { scale, x: width / 2 - cx * scale, y: height / 2 - cz * scale };
}

/** Zoom mantendo o ponto (`px`, `py`) da tela parado. */
export function zoomAt(view: View, factor: number, px: number, py: number): View {
  const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, view.scale * factor));
  const ratio = scale / view.scale;
  return { scale, x: px - (px - view.x) * ratio, y: py - (py - view.y) * ratio };
}

/** Centraliza o ponto do mundo (`x`, `z`) na tela, com pelo menos `minScale` de zoom. */
export function centerOn(view: View, x: number, z: number, width: number, height: number, minScale = 0.35): View {
  const scale = Math.max(view.scale, minScale);
  return { scale, x: width / 2 - x * scale, y: height / 2 - z * scale };
}

export interface Stop {
  /** Posição na rota, a partir de 1. */
  n: number;
  zone: MapZone;
  zoneIndex: number;
}

/** Paradas do grupo na ordem da rota. */
export function stopsOf(group: MapGroup, zones: readonly MapZone[]): Stop[] {
  return group.stops.flatMap((zoneIndex, i) => {
    const zone = zones[zoneIndex];
    return zone ? [{ n: i + 1, zone, zoneIndex }] : [];
  });
}

/** Paradas em que o grupo deve estar no horário `hour`. */
export function stopsNow(stops: readonly Stop[], hour: number): Set<Stop> {
  return new Set(stops.filter((s) => inSlot(s.zone.start, s.zone.end, hour)));
}

/** Grupos de uma espécie: os que têm Great One primeiro, depois os de maior pontuação. */
export function groupsOfSpecies(map: ReserveMap, species: number): MapGroup[] {
  const best = (g: MapGroup): number => Math.max(0, ...g.animals.map((a) => a.score));
  return map.groups
    .filter((g) => g.species === species)
    .sort(
      (a, b) =>
        Number(b.animals.some((x) => x.greatOne)) - Number(a.animals.some((x) => x.greatOne)) ||
        best(b) - best(a) ||
        a.number - b.number,
    );
}

/** Quantas zonas visíveis (nos outros filtros) cada espécie tem. */
export function countBySpecies(map: ReserveMap, filters: Filters): Map<number, number> {
  const counts = new Map<number, number>();
  const all = new Set(map.species.map((_, i) => i));
  for (const zone of visibleZones(map.zones, { ...filters, species: all })) {
    counts.set(zone.species, (counts.get(zone.species) ?? 0) + 1);
  }
  return counts;
}

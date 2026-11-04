import type { ReserveSummary, UnlockRequest, ZonesPreview } from './types';

/** O que liberar: tudo, só alguns animais ou só algumas necessidades (comer, beber, descansar). */
export type UnlockMode = 'all' | 'animal' | 'need';

export interface UnlockState {
  /** Reservas marcadas; `null` enquanto o usuário não mexeu (todas as que têm algo a liberar). */
  reserves: Set<number> | null;
  mode: UnlockMode;
  species: Set<string>;
  needs: Set<number>;
}

export function initialUnlockState(): UnlockState {
  return { reserves: null, mode: 'all', species: new Set(), needs: new Set() };
}

/** Reservas com zonas a liberar. */
export function readyReserves(preview: ZonesPreview): ReserveSummary[] {
  return preview.reserves.filter((r) => r.state === 'ready');
}

/** Reservas marcadas que têm zonas a liberar. */
export function chosenReserves(preview: ZonesPreview, state: UnlockState): ReserveSummary[] {
  const { reserves } = state;
  return readyReserves(preview).filter((r) => reserves === null || reserves.has(r.id));
}

export interface SpeciesTotal {
  id: string;
  name: string;
  count: number;
}

export interface Totals {
  /** Zonas que seriam liberadas com o filtro atual. */
  total: number;
  /** Reservas que ganhariam alguma zona. */
  reserves: number;
  /** Zonas por espécie nas reservas marcadas (sem o filtro de espécie), para a lista de animais. */
  species: SpeciesTotal[];
  /** Zonas por necessidade (comer, beber, descansar) nas reservas marcadas, para os botões. */
  needs: [number, number, number];
}

const sum = (values: readonly number[]): number => values.reduce((a, b) => a + b, 0);

/** Quantas zonas cada escolha libera, calculado das contagens que o app já trouxe. */
export function summarize(preview: ZonesPreview, state: UnlockState): Totals {
  const chosen = chosenReserves(preview, state);
  const bySpecies = new Map<string, SpeciesTotal>();
  const needs: [number, number, number] = [0, 0, 0];
  let total = 0;
  let reserves = 0;
  for (const reserve of chosen) {
    let reserveTotal = 0;
    for (const row of reserve.species) {
      const speciesOk = state.mode !== 'animal' || state.species.has(row.id);
      row.needs.forEach((count, index) => {
        needs[index] = (needs[index] ?? 0) + count;
        if (speciesOk && (state.mode !== 'need' || state.needs.has(index + 1))) reserveTotal += count;
      });
      if (row.id !== '') {
        const known = bySpecies.get(row.id) ?? { id: row.id, name: row.name, count: 0 };
        known.count += sum(row.needs);
        bySpecies.set(row.id, known);
      }
    }
    total += reserveTotal;
    if (reserveTotal > 0) reserves += 1;
  }
  const species = [...bySpecies.values()].sort((a, b) => a.name.localeCompare(b.name, 'pt-BR'));
  return { total, reserves, species, needs };
}

/** O pedido que a interface envia ao app. */
export function toRequest(preview: ZonesPreview, state: UnlockState): UnlockRequest {
  return {
    reserves: chosenReserves(preview, state).map((r) => r.id),
    species: state.mode === 'animal' ? [...state.species] : null,
    needs: state.mode === 'need' ? [...state.needs] : null,
  };
}

/** Marca ou desmarca `value` num conjunto, sem mudar o original. */
export function toggled<T>(set: ReadonlySet<T>, value: T): Set<T> {
  const next = new Set(set);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

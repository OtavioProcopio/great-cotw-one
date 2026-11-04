import { describe, expect, it } from 'vitest';

import { zonesPreview } from './test-data';
import {
  chosenReserves,
  initialUnlockState,
  readyReserves,
  summarize,
  toRequest,
  toggled,
  type UnlockState,
} from './unlocklogic';

describe('unlock logic', () => {
  const start = initialUnlockState();

  it('starts with every reserve that has something to unlock', () => {
    expect(readyReserves(zonesPreview).map((r) => r.id)).toEqual([2, 13]);
    expect(chosenReserves(zonesPreview, start).map((r) => r.id)).toEqual([2, 13]);
    expect(chosenReserves(zonesPreview, { ...start, reserves: new Set([13, 0]) }).map((r) => r.id)).toEqual([13]);
  });

  it('adds everything up when nothing is filtered', () => {
    const totals = summarize(zonesPreview, start);
    expect(totals.total).toBe(7_330);
    expect(totals.reserves).toBe(2);
    expect(totals.needs).toEqual([2_940, 2_540, 1_850]);
    expect(totals.species.map((s) => [s.name, s.count])).toEqual([
      ['Alce', 7_219],
      ['Urso-pardo', 111],
    ]);
  });

  it('counts only the chosen animals', () => {
    const state: UnlockState = { ...start, mode: 'animal', species: new Set(['moose']) };
    expect(summarize(zonesPreview, state).total).toBe(7_219);
    expect(summarize(zonesPreview, { ...state, species: new Set() }).total).toBe(0);
  });

  it('counts only the chosen needs and the chosen reserves', () => {
    const drink: UnlockState = { ...start, mode: 'need', needs: new Set([2]) };
    expect(summarize(zonesPreview, drink).total).toBe(2_540);
    const oneReserve: UnlockState = { ...drink, reserves: new Set([13]) };
    expect(summarize(zonesPreview, oneReserve)).toMatchObject({ total: 1_000, reserves: 1 });
  });

  it('builds the request the app expects', () => {
    expect(toRequest(zonesPreview, start)).toEqual({ reserves: [2, 13], species: null, needs: null });
    expect(toRequest(zonesPreview, { ...start, mode: 'animal', species: new Set(['moose']) })).toEqual({
      reserves: [2, 13],
      species: ['moose'],
      needs: null,
    });
    expect(toRequest(zonesPreview, { ...start, mode: 'need', needs: new Set([1, 3]) })).toEqual({
      reserves: [2, 13],
      species: null,
      needs: [1, 3],
    });
  });

  it('toggles a value without touching the original set', () => {
    const original = new Set([1]);
    expect([...toggled(original, 2)]).toEqual([1, 2]);
    expect([...toggled(original, 1)]).toEqual([]);
    expect([...original]).toEqual([1]);
  });
});

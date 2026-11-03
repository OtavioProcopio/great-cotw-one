import { describe, expect, it } from 'vitest';

import {
  type Filters,
  boundsOf,
  centerOn,
  countBySpecies,
  fitView,
  formatHour,
  groupsOfSpecies,
  inSlot,
  screenToWorld,
  slotText,
  stopsNow,
  stopsOf,
  visibleZones,
  worldToScreen,
  zoomAt,
} from './maplogic';
import { sampleMap } from './test-data';

const everything: Filters = {
  needs: new Set([1, 2, 3]),
  species: new Set([0, 1]),
  showPending: true,
  hour: null,
};

describe('hours', () => {
  it('formats hours and slots', () => {
    expect(formatHour(4)).toBe('04:00');
    expect(formatHour(6.5)).toBe('06:30');
    expect(formatHour(24)).toBe('00:00');
    expect(slotText(4, 8)).toBe('04:00 – 08:00');
  });

  it('checks slots, including the ones that cross midnight', () => {
    expect(inSlot(4, 8, 4)).toBe(true);
    expect(inSlot(4, 8, 8)).toBe(false);
    expect(inSlot(20, 0, 23)).toBe(true);
    expect(inSlot(20, 0, 1)).toBe(false);
    expect(inSlot(22, 4, 2)).toBe(true);
    expect(inSlot(0, 0, 13)).toBe(true);
  });
});

describe('visibleZones', () => {
  it('shows everything by default', () => {
    expect(visibleZones(sampleMap.zones, everything)).toHaveLength(6);
  });

  it('filters by need, species, pending and hour', () => {
    expect(visibleZones(sampleMap.zones, { ...everything, needs: new Set([2]) })).toHaveLength(2);
    expect(visibleZones(sampleMap.zones, { ...everything, species: new Set([1]) })).toHaveLength(3);
    expect(visibleZones(sampleMap.zones, { ...everything, showPending: false })).toHaveLength(4);
    const at6 = visibleZones(sampleMap.zones, { ...everything, hour: 6 });
    expect(at6.map((z) => z.kind)).toEqual([1, 3]);
    const atMidnight = visibleZones(sampleMap.zones, { ...everything, hour: 23 });
    expect(atMidnight).toHaveLength(1);
  });

  it('keeps zones of unknown species whenever the other filters allow them', () => {
    const none = visibleZones(sampleMap.zones, { ...everything, species: new Set() });
    expect(none).toHaveLength(1);
    expect(none[0]?.species).toBe(-1);
  });
});

describe('view', () => {
  it('converts between world and screen and back', () => {
    const view = { scale: 0.05, x: 100, y: -40 };
    const [sx, sy] = worldToScreen(view, -8000, 2000);
    const [wx, wz] = screenToWorld(view, sx, sy);
    expect(wx).toBeCloseTo(-8000);
    expect(wz).toBeCloseTo(2000);
  });

  it('fits the zones inside the area', () => {
    const view = fitView(boundsOf(sampleMap), 800, 600);
    const [x0, y0] = worldToScreen(view, -12000, -12000);
    const [x1, y1] = worldToScreen(view, -7000, -9500);
    for (const v of [x0, x1]) expect(v).toBeGreaterThan(0);
    for (const v of [x0, x1]) expect(v).toBeLessThan(800);
    for (const v of [y0, y1]) expect(v).toBeGreaterThan(0);
    for (const v of [y0, y1]) expect(v).toBeLessThan(600);
  });

  it('falls back to the map image square when there are no zones', () => {
    const bounds = boundsOf({ ...sampleMap, zones: [] });
    expect(bounds).toEqual({ minX: -16384, maxX: 0, minZ: -16384, maxZ: 0 });
  });

  it('zooms around a fixed point and respects the limits', () => {
    const view = { scale: 0.1, x: 50, y: 20 };
    const zoomed = zoomAt(view, 2, 300, 200);
    expect(screenToWorld(zoomed, 300, 200)[0]).toBeCloseTo(screenToWorld(view, 300, 200)[0]);
    expect(zoomAt(view, 1e9, 0, 0).scale).toBe(3);
    expect(zoomAt(view, 1e-9, 0, 0).scale).toBe(0.01);
  });

  it('centers on a point with at least a minimum zoom', () => {
    const view = centerOn({ scale: 0.05, x: 0, y: 0 }, -9000, -10500, 800, 600);
    expect(view.scale).toBe(0.35);
    const [sx, sy] = worldToScreen(view, -9000, -10500);
    expect([sx, sy]).toEqual([400, 300]);
  });
});

describe('groups', () => {
  it('lists the stops in route order and skips missing zones', () => {
    const stops = stopsOf(sampleMap.groups[2] ?? { number: 0, species: 0, stops: [], animals: [] }, sampleMap.zones);
    expect(stops.map((s) => s.n)).toEqual([1, 2]);
  });

  it('finds the stops where the group should be at a given hour', () => {
    const group = sampleMap.groups[0];
    if (!group) throw new Error('grupo ausente');
    const stops = stopsOf(group, sampleMap.zones);
    expect([...stopsNow(stops, 9)].map((s) => s.n)).toEqual([2]);
    expect(stopsNow(stops, 23).size).toBe(0);
  });

  it('puts groups with a Great One first, then by score', () => {
    expect(groupsOfSpecies(sampleMap, 0).map((g) => g.number)).toEqual([1, 2]);
    expect(groupsOfSpecies(sampleMap, 1).map((g) => g.number)).toEqual([3]);
  });

  it('counts zones per species ignoring the species filter', () => {
    const counts = countBySpecies(sampleMap, { ...everything, species: new Set() });
    expect(counts.get(0)).toBe(3);
    expect(counts.get(1)).toBe(2);
    expect(counts.get(-1)).toBe(1);
  });
});

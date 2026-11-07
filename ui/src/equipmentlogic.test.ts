import { describe, expect, it } from 'vitest';

import { KINDS, MAX_CASH, collection, itemsById, maxAmount, rankValue, rankWeapons, visibleItems } from './equipmentlogic';
import { mockInventory } from './mock-data';
import type { EquipmentItem, WeaponStats } from './types';

const items = mockInventory().items;
const first = items[0];
if (!first?.stats) throw new Error('exemplo sem estatísticas');
const rifle: EquipmentItem = first;
const base: WeaponStats = first.stats;

describe('visibleItems', () => {
  it('keeps one category and filters by ownership', () => {
    expect(visibleItems(items, 'weapon', 'all').map((i) => i.name)).toEqual(['.270 Bolt-Action Rifle', 'Longbow']);
    expect(visibleItems(items, 'weapon', 'owned')).toHaveLength(1);
    expect(visibleItems(items, 'ammo', 'missing').map((i) => i.id)).toEqual(['equipment_ammo_270_pt_01']);
  });
});

describe('collection', () => {
  it('counts what the player owns in every category', () => {
    const progress = collection(items);
    expect(progress.weapon).toEqual({ owned: 1, total: 2 });
    expect(progress.ammo).toEqual({ owned: 1, total: 2 });
    expect(progress.deployable).toEqual({ owned: 0, total: 1 });
    expect(Object.keys(progress)).toEqual(KINDS);
  });
});

describe('ranking', () => {
  it('uses only weapons that have fired and orders them best first', () => {
    const second: EquipmentItem = { ...rifle, id: 'b', name: 'B', stats: { ...base, kills: 90, accuracy: 50 } };
    const never: EquipmentItem = { ...rifle, id: 'c', name: 'C', stats: { ...base, shotsFired: 0 } };
    const list = [rifle, second, never, ...items.slice(1)];
    expect(rankWeapons(list, 'kills').map((i) => i.id)).toEqual(['b', rifle.id]);
    expect(rankWeapons(list, 'accuracy').map((i) => i.id)).toEqual([rifle.id, 'b']);
    expect(rankValue(never, 'kills')).toBeNull();
    expect(rankValue({ ...rifle, stats: null }, 'kills')).toBeNull();
  });

  it.each([
    ['record', 812.5],
    ['distance', 250.5],
    ['shots', 100],
  ] as const)('reads %s from the weapon', (key, expected) => {
    expect(rankValue(rifle, key)).toBe(expected);
  });
});

describe('itemsById', () => {
  it('keeps the order and skips unknown ids', () => {
    expect(itemsById(items, ['equipment_ammo_270_sp_01', 'nao_existe', 'equipment_atv_01']).map((i) => i.name)).toEqual([
      '.270 Soft-Point Bullet',
      'ATV',
    ]);
  });
});

describe('maxAmount', () => {
  it('lets ammo and stackable items hold many units and keeps the rest at one', () => {
    expect(maxAmount('ammo')).toBe(999);
    expect(maxAmount('consumable')).toBe(99);
    expect(maxAmount('lure_caller')).toBe(99);
    expect(maxAmount('deployable')).toBe(99);
    expect(maxAmount('weapon')).toBe(1);
    expect(maxAmount('sight')).toBe(1);
    expect(MAX_CASH).toBe(10_000_000);
  });
});

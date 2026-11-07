import { describe, expect, it } from 'vitest';

import { mockInventory } from '../mock-data';
import { bindEquipment, initialEquipState, renderEquipment } from './equipment';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

const inv = mockInventory();

describe('renderEquipment', () => {
  it('shows the cash, the weapon levels, the tabs and the weapons of the arsenal', () => {
    const root = html(renderEquipment(inv, initialEquipState(), {}));
    const text = root.textContent;
    expect(text).toContain('300.618');
    expect(text).toContain('Nível dos rifles');
    expect(text).toContain('Ranking das armas');
    expect(text).toContain('.270 Bolt-Action Rifle');
    expect(text).toContain('Liberado para comprar');
    expect(root.querySelectorAll('.eq-card')).toHaveLength(2);
  });

  it('counts the collection on each category button', () => {
    const root = html(renderEquipment(inv, initialEquipState(), {}));
    expect(root.querySelector('[data-kind="weapon"]')?.textContent).toContain('1 de 2');
    expect(root.querySelector('[data-kind="ammo"]')?.textContent).toContain('1 de 2');
  });

  it('shows amounts for ammo and the box size', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), kind: 'ammo' }, {}));
    expect(root.textContent).toContain('203 unidades');
    expect(root.textContent).toContain('10 por caixa');
  });

  it('uses the game icon when it is already loaded and asks for it otherwise', () => {
    const none = html(renderEquipment(inv, initialEquipState(), {}));
    expect(none.querySelectorAll('img[data-equip-icon]')).toHaveLength(2);
    const loaded = html(renderEquipment(inv, initialEquipState(), { equipment_weapon_ba_rifle_270_01: 'data:image/png;base64,AAAA' }));
    expect(loaded.querySelectorAll('img[data-equip-icon]')).toHaveLength(1);
    expect(loaded.querySelector('img[src^="data:image/png"]')).not.toBeNull();
  });

  it('opens the weapon sheet with stats, ammo and sights', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), open: 'equipment_weapon_ba_rifle_270_01' }, {}));
    const text = root.textContent;
    expect(text).toContain('75%');
    expect(text).toContain('813 pontos (Alce)');
    expect(text).toContain('Munições que servem');
    expect(text).toContain('.270 Polymer-Tip Bullet');
    expect(text).toContain('Miras que servem');
    expect(root.querySelector('[data-action="equip-close"]')).not.toBeNull();
  });

  it('says so when a weapon has no shots yet', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), open: 'equipment_weapon_longbow_01' }, {}));
    expect(root.textContent).toContain('ainda não tem tiros');
  });

  it('shows nothing for a weapon id that does not exist', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), open: 'nao_existe' }, {}));
    expect(root.querySelector('.eq-sheet-head')).toBeNull();
  });

  it('ranks the used weapons and says when there are none', () => {
    const state = { ...initialEquipState(), tab: 'ranking' as const };
    const ranked = html(renderEquipment(inv, state, {}));
    expect(ranked.querySelectorAll('.eq-rank li')).toHaveLength(1);
    expect(ranked.querySelector('.eq-value')?.textContent).toBe('40');
    const empty = html(renderEquipment({ ...inv, items: inv.items.map((i) => ({ ...i, stats: null })) }, state, {}));
    expect(empty.textContent).toContain('Nenhuma arma foi usada ainda.');
  });

  it('formats the accuracy and the distance on the ranking', () => {
    const accuracy = html(renderEquipment(inv, { ...initialEquipState(), tab: 'ranking', rank: 'accuracy' }, {}));
    expect(accuracy.querySelector('.eq-value')?.textContent).toBe('75%');
    const distance = html(renderEquipment(inv, { ...initialEquipState(), tab: 'ranking', rank: 'distance' }, {}));
    expect(distance.querySelector('.eq-value')?.textContent).toContain('251');
  });

  it('lists the loadouts with what is missing', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), tab: 'loadouts' }, {}));
    const text = root.textContent;
    expect(text).toContain('Alces');
    expect(text).toContain('Em uso');
    expect(text).toContain('Faltam: Longbow');
    expect(text).toContain('Conjunto 2');
    expect(text).toContain('Conjunto vazio.');
    const none = html(renderEquipment({ ...inv, loadouts: [] }, { ...initialEquipState(), tab: 'loadouts' }, {}));
    expect(none.textContent).toContain('Nenhum conjunto salvo.');
  });

  it('says so when a filter leaves nothing', () => {
    const root = html(renderEquipment(inv, { ...initialEquipState(), kind: 'deployable', own: 'owned' }, {}));
    expect(root.textContent).toContain('Nenhum item aqui.');
  });
});

describe('bindEquipment', () => {
  it('filters the cards by text without redrawing', () => {
    const root = html(renderEquipment(inv, initialEquipState(), {}));
    bindEquipment(root);
    const input = root.querySelector<HTMLInputElement>('#eq-search');
    if (!input) throw new Error('sem campo de busca');
    input.value = 'longbow';
    input.dispatchEvent(new Event('input'));
    const shown = [...root.querySelectorAll<HTMLElement>('#eq-grid li')].filter((li) => !li.hidden);
    expect(shown).toHaveLength(1);
    input.value = '';
    input.dispatchEvent(new Event('input'));
    expect([...root.querySelectorAll<HTMLElement>('#eq-grid li')].every((li) => !li.hidden)).toBe(true);
  });

  it('does nothing on other screens', () => {
    expect(() => {
      bindEquipment(html('<p>nada</p>'));
    }).not.toThrow();
  });
});

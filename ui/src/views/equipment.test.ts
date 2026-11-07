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

describe('Premium actions', () => {
  it('shows locked buttons that lead to the Premium page for free users', () => {
    const root = html(renderEquipment(inv, initialEquipState(), {}));
    const locked = [...root.querySelectorAll<HTMLElement>('.eq-act.locked')];
    expect(locked.length).toBeGreaterThan(0);
    expect(locked.every((b) => b.dataset['action'] === 'show-premium')).toBe(true);
    expect(root.querySelector('#eq-cash')).toBeNull();
    expect(root.querySelector('[data-action="equip-give"]')).toBeNull();
  });

  it('offers to give what is missing and to unlock what is not on the shop list', () => {
    const root = html(renderEquipment(inv, initialEquipState(), {}, true));
    // Longbow: não tem, mas já está liberado: só "Entregar".
    const give = root.querySelector('[data-action="equip-give"][data-id="equipment_weapon_longbow_01"]');
    expect(give?.textContent).toBe('Entregar');
    expect(root.querySelector('[data-action="equip-unlock"][data-id="equipment_weapon_longbow_01"]')).toBeNull();
    // A arma que o jogador tem não ganha botão de entregar nem de quantidade.
    expect(root.querySelector('[data-action="equip-give"][data-id="equipment_weapon_ba_rifle_270_01"]')).toBeNull();
    const ammo = html(renderEquipment(inv, { ...initialEquipState(), kind: 'ammo' }, {}, true));
    expect(ammo.querySelector('[data-action="equip-give"][data-id="equipment_ammo_270_pt_01"]')).not.toBeNull();
    expect(ammo.querySelector('[data-action="equip-unlock"][data-id="equipment_ammo_270_pt_01"]')).not.toBeNull();
  });

  it('lets the player set the amount of ammo they own and their cash', () => {
    const ammo = html(renderEquipment(inv, { ...initialEquipState(), kind: 'ammo' }, {}, true));
    const field = ammo.querySelector<HTMLInputElement>('[data-eq-amount="equipment_ammo_270_sp_01"]');
    expect(field?.value).toBe('203');
    expect(field?.max).toBe('999');
    expect(ammo.querySelector('[data-action="equip-set-amount"]')).not.toBeNull();
    expect(ammo.querySelector<HTMLInputElement>('#eq-cash')?.value).toBe('300618');
    expect(ammo.textContent).toContain('Até 10.000.000.');
  });

  it('does not offer an amount for items that cannot stack', () => {
    const sights = html(renderEquipment(inv, { ...initialEquipState(), kind: 'sight' }, {}, true));
    expect(sights.querySelector('[data-eq-amount]')).toBeNull();
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

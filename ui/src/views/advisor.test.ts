import { describe, expect, it } from 'vitest';

import { mockAdvice, mockInventory } from '../mock-data';
import { sampleAnimals } from '../test-data';
import { initialAdvisorState, renderAdvisor } from './advisor';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

const inv = mockInventory();
const reserveId = sampleAnimals[0]?.id ?? 0;
const speciesId = sampleAnimals[0]?.species[0]?.id ?? '';

describe('renderAdvisor', () => {
  it('asks for a reserve first and then for the animal', () => {
    const first = html(renderAdvisor(inv, sampleAnimals, initialAdvisorState(), null, false));
    expect(first.textContent).toContain('Escolha onde vai caçar');
    expect(first.querySelectorAll('[data-action="adv-reserve"]').length).toBeGreaterThan(0);
    expect(first.querySelector('[data-action="adv-species"]')).toBeNull();
    const second = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: null }, null, false));
    expect(second.querySelector(`[data-action="adv-species"][data-species="${speciesId}"]`)).not.toBeNull();
    expect(second.querySelector('.eq-pick')).toBeNull();
  });

  it('says so when the game gives no hunting class for the animal', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: speciesId }, null, false));
    expect(root.textContent).toContain('não informa a classe de caça');
  });

  it('shows the class, what the player can use and what is worth buying', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: speciesId }, mockAdvice(), true));
    const text = root.textContent;
    expect(text).toContain('Alce');
    expect(text).toContain('classe de caça 8');
    expect(text).toContain('Você já pode usar');
    expect(text).toContain('Vale comprar');
    expect(text).toContain('.270 Bolt-Action Rifle');
    expect(text).toContain('Munição para classes 4 a 8');
    expect(text).toContain('Precisão 55');
    expect(text).toContain('Alcance 150 m');
    expect(text).toContain('3-9x 44mm Rifle Scope');
    expect(root.querySelectorAll('.eq-part').length).toBeGreaterThanOrEqual(4);
    expect(text).toContain('Você tem arma e munição');
    expect(text).toContain('Falta gastar 12.560');
    expect(root.querySelectorAll('.eq-pick')).toHaveLength(2);
  });

  it('offers to give the missing set only to Premium users and locks it for the others', () => {
    const state = { reserve: reserveId, species: speciesId };
    const premium = html(renderAdvisor(inv, sampleAnimals, state, mockAdvice(), true));
    const give = premium.querySelector<HTMLElement>('[data-action="adv-give"]');
    expect(give?.dataset['weapon']).toBe('equipment_weapon_longbow_01');
    expect(give?.dataset['ammo']).toBe('equipment_ammo_270_pt_01');
    const free = html(renderAdvisor(inv, sampleAnimals, state, mockAdvice(), false));
    expect(free.querySelector('[data-action="adv-give"]')).toBeNull();
    expect(free.querySelector('.eq-act.locked')?.getAttribute('data-action')).toBe('show-premium');
  });

  it('shows the level that unlocks what is blocked and an empty message when there is nothing', () => {
    const advice = mockAdvice();
    const blocked = { ...advice, ready: [], toBuy: advice.toBuy.map((p) => ({ ...p, available: false })) };
    const root = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: speciesId }, blocked, true));
    expect(root.textContent).toContain('Libera no nível 7 (rifles)');
    expect(root.textContent).toContain('Nada do seu arsenal serve bem');
    const none = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: speciesId }, { ...advice, toBuy: [] }, true));
    expect(none.textContent).toContain('Nada a sugerir comprar.');
  });

  it('falls back to the item id when the catalog item is unknown', () => {
    const advice = mockAdvice();
    const first = advice.ready[0];
    if (!first) throw new Error('exemplo sem sugestão');
    const odd = { ...advice, ready: [{ ...first, weapon: 'equipment_weapon_desconhecida_01' }], toBuy: [] };
    const root = html(renderAdvisor(inv, sampleAnimals, { reserve: reserveId, species: speciesId }, odd, true));
    expect(root.textContent).toContain('weapon desconhecida 01');
  });
});

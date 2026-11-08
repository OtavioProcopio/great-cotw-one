import { describe, expect, it } from 'vitest';

import { mockAdvice, mockInventory, mockKit } from '../mock-data';
import { sampleAnimals } from '../test-data';
import { initialAdvisorState, limitNote, renderAdvisor } from './advisor';

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

describe('whole-reserve kit', () => {
  const state = { reserve: reserveId, species: null, mode: 'reserve' as const };

  it('offers a switch between one animal and the whole reserve', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, initialAdvisorState(), null, false));
    expect(root.querySelectorAll('[data-action="adv-mode"]')).toHaveLength(2);
    expect(root.querySelector('[data-action="adv-mode"][aria-pressed="true"]')?.getAttribute('data-mode')).toBe('animal');
  });

  it('asks only for the reserve in reserve mode and shows no kit before choosing one', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, { reserve: null, species: null, mode: 'reserve' }, null, false));
    expect(root.querySelectorAll('[data-action="adv-reserve"]').length).toBeGreaterThan(0);
    expect(root.querySelector('[data-action="adv-species"]')).toBeNull();
    expect(root.textContent).not.toContain('Armas para levar');
  });

  it('shows weapons, lures, extras and the total weight', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, state, null, false, {}, mockKit(), null));
    const text = root.textContent;
    expect(text).toContain('3 animais considerados');
    expect(text).toContain(`Peso do kit: ${(5.6).toLocaleString()} kg`);
    expect(text).toContain('Armas para levar');
    expect(text).toContain('Atende: Alce, Cervo-almiscarado');
    expect(text).toContain('Para cobrir o que falta');
    expect(text).toContain('Iscas e chamados');
    expect(text).toContain('Atrai: Alce');
    expect(text).toContain('Atrai: Lobo-cinzento');
    expect(text).toContain('Extras úteis');
    expect(root.querySelector('[data-weight="5.6"]')).not.toBeNull();
  });

  it('says so when the reserve could not be built and when nothing covers an animal', () => {
    const none = html(renderAdvisor(inv, sampleAnimals, state, null, false, {}, null, null));
    expect(none.textContent).toContain('Não consegui montar o kit');
    const kit = { ...mockKit(), weapons: [], toBuy: [], uncovered: ['Alce'], lures: [], luresToBuy: [], extras: [] };
    const root = html(renderAdvisor(inv, sampleAnimals, state, null, false, {}, kit, null));
    expect(root.textContent).toContain('não cobre nenhum animal');
    expect(root.textContent).toContain('Nem o seu arsenal nem a loja cobrem: Alce.');
    expect(root.textContent).toContain('não tem iscas');
  });

  it('compares the kit weight with the limit the player typed', () => {
    expect(limitNote(5.6, null)).toContain('Digite o máximo');
    expect(limitNote(5.6, 0)).toContain('Digite o máximo');
    expect(limitNote(5.6, 30)).toBe(`Cabe no seu limite: sobram ${(24.4).toLocaleString()} kg.`);
    expect(limitNote(5.6, 5)).toContain(`Passa ${(0.6).toLocaleString()} kg do seu limite`);
    const root = html(renderAdvisor(inv, sampleAnimals, state, null, false, {}, mockKit(), 30));
    expect(root.querySelector<HTMLInputElement>('#kit-limit')?.value).toBe('30');
    expect(root.textContent).toContain(`sobram ${(24.4).toLocaleString()} kg`);
  });

  it('offers Premium users a button to give the missing set', () => {
    const root = html(renderAdvisor(inv, sampleAnimals, state, null, true, {}, mockKit(), null));
    expect(root.querySelector('[data-action="adv-give"]')?.getAttribute('data-weapon')).toBe('equipment_weapon_longbow_01');
  });
});


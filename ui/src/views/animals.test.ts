import { describe, expect, it } from 'vitest';

import { sampleAnimals } from '../test-data';
import { bindAnimals, hueOf, renderAnimals, reservesOf } from './animals';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

describe('animals helpers', () => {
  it('gives a species the same hue every time, between 0 and 359', () => {
    expect(hueOf('moose')).toBe(hueOf('moose'));
    expect(hueOf('moose')).not.toBe(hueOf('brown_bear'));
    expect(hueOf('')).toBe(0);
    expect(hueOf('red_deer')).toBeLessThan(360);
  });

  it('finds the reserves where a species lives', () => {
    expect(reservesOf(sampleAnimals, 'moose').map((r) => r.id)).toEqual([2, 8]);
    expect(reservesOf(sampleAnimals, 'brown_bear').map((r) => r.id)).toEqual([2]);
    expect(reservesOf(sampleAnimals, 'dragon')).toEqual([]);
  });
});

describe('renderAnimals', () => {
  it('shows one card per reserve with the species count and whether the player has been there', () => {
    const root = html(renderAnimals(sampleAnimals, 2, null));
    expect(root.querySelectorAll('.reserve-card')).toHaveLength(2);
    expect(root.querySelectorAll('.reserve-card img')).toHaveLength(1);
    expect(root.querySelector('.reserve-card[aria-pressed="true"]')?.textContent).toContain('Medved-Taiga');
    expect(root.textContent).toContain('2 espécies');
    expect(root.querySelectorAll('.played-dot')).toHaveLength(1);
    expect(root.querySelector('.reserve-card[data-reserve="8"]')?.getAttribute('title')).toBe('Você ainda não jogou aqui');
  });

  it('lists the species of the chosen reserve with what the save knows', () => {
    const root = html(renderAnimals(sampleAnimals, 2, null));
    expect(root.querySelectorAll('.species-card')).toHaveLength(2);
    expect(root.textContent).toContain('9 animais em 3 grupos');
    expect(root.textContent).toContain('1 Great One');
    expect(root.textContent).toContain('2 diamante(s)');
    expect(root.textContent).toContain('Sem animais no seu progresso');
    expect(root.querySelector('.species-detail')).toBeNull();
  });

  it('opens the species sheet with where it lives, the live animals and the best harvests', () => {
    const root = html(renderAnimals(sampleAnimals, 2, 'moose'));
    const detail = root.querySelector('.species-detail');
    expect(detail?.textContent).toContain('Alce');
    expect([...root.querySelectorAll('.chip-link')].map((c) => c.textContent)).toEqual(['Medved-Taiga', 'Cuatro Colinas']);
    expect(root.querySelector('.chip-link[aria-pressed="true"]')?.textContent).toBe('Medved-Taiga');
    expect(detail?.textContent).toContain('♂ 4 · ♀ 5');
    expect(detail?.textContent).toContain('640 kg');
    expect(detail?.textContent).toContain('Diamante a partir de (pontos)');
    expect(detail?.textContent).toContain('255');
    expect(root.querySelectorAll('.harvests li')).toHaveLength(2);
    expect(root.querySelector('.medal')?.textContent).toBe('★★★');
    expect(root.querySelectorAll('.medal')[1]?.textContent).toBe('sem medalha');
    expect(root.querySelector('.species-card[aria-pressed="true"]')?.textContent).toContain('Alce');
  });

  it('says so when the species has no live animals or harvests', () => {
    const root = html(renderAnimals(sampleAnimals, 2, 'brown_bear'));
    const text = root.querySelector('.species-detail')?.textContent ?? '';
    expect(text).toContain('Sem animais no seu progresso');
    expect(text).toContain('Você ainda não abateu esta espécie.');
  });

  it('escapes names and falls back to the first reserve', () => {
    const root = html(renderAnimals(sampleAnimals, 99, null));
    expect(root.querySelector('i')).toBeNull();
    expect(root.textContent).toContain('<i>Urso-pardo</i>');
    expect(root.querySelector('.reserve-card[aria-pressed="true"]')?.textContent).toContain('Medved-Taiga');
    expect(html(renderAnimals(sampleAnimals, 8, 'inexistente')).querySelector('.species-detail')).toBeNull();
  });

  it('copes with a game without any reserve', () => {
    expect(html(renderAnimals([], null, null)).textContent).toContain('Não encontramos animais no jogo');
  });

  it('filters the species by name as the player types', () => {
    const root = html(renderAnimals(sampleAnimals, 2, null));
    bindAnimals(root);
    const search = root.querySelector<HTMLInputElement>('#animal-search');
    if (!search) throw new Error('busca ausente');
    const shown = (): number => [...root.querySelectorAll<HTMLElement>('#species-grid li')].filter((li) => !li.hidden).length;
    search.value = 'ALC';
    search.dispatchEvent(new Event('input'));
    expect(shown()).toBe(1);
    expect(root.querySelector<HTMLElement>('#animal-empty')?.hidden).toBe(true);
    search.value = 'zzz';
    search.dispatchEvent(new Event('input'));
    expect(shown()).toBe(0);
    expect(root.querySelector<HTMLElement>('#animal-empty')?.hidden).toBe(false);
    search.value = '';
    search.dispatchEvent(new Event('input'));
    expect(shown()).toBe(2);
  });

  it('does nothing without a species grid', () => {
    expect(() => {
      bindAnimals(html(renderAnimals([], null, null)));
    }).not.toThrow();
  });
});

import { describe, expect, it } from 'vitest';

import { sampleMissions, sampleProfile } from '../test-data';
import type { Profile } from '../types';
import { filterMissions, renderMissions } from './missions';
import { accuracy, bindProfile, renderProfile, ring } from './profile';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

describe('ring', () => {
  it('fills a share of the circle and clamps the percentage', () => {
    const full = html(ring(100, '100%', 'tudo'));
    expect(full.querySelector('.ring-fill')?.getAttribute('stroke-dashoffset')).toBe('0.0');
    const over = html(ring(250, '?', ''));
    expect(over.querySelector('.ring-fill')?.getAttribute('stroke-dashoffset')).toBe('0.0');
    const none = html(ring(-5, '0%', ''));
    expect(Number(none.querySelector('.ring-fill')?.getAttribute('stroke-dashoffset'))).toBeGreaterThan(300);
  });
});

describe('renderProfile', () => {
  it('shows the level, the marks and the achievements', () => {
    const root = html(renderProfile(sampleProfile));
    const text = root.textContent;
    expect(text).toContain('Nível 60');
    expect(text).toContain('Animais abatidos');
    expect(text).toContain('124');
    expect(text).toContain('75%');
    expect(text).toContain('1 de 3 conquistas');
    expect(root.querySelectorAll('.metric')).toHaveLength(8);
    // As obtidas vêm primeiro.
    expect(root.querySelector('.ach')?.classList.contains('ach-done')).toBe(true);
  });

  it('hides the name and the description of secret achievements and escapes text', () => {
    const root = html(renderProfile(sampleProfile));
    expect(root.textContent).toContain('Conquista secreta');
    expect(root.textContent).not.toContain('Escondida.');
    expect(root.querySelector('b')).toBeNull();
  });

  it('explains how to get the achievements when the Steam files are missing', () => {
    const none: Profile = { ...sampleProfile, hasAchievements: false, achievements: [] };
    const root = html(renderProfile(none));
    expect(root.textContent).toContain('Abra a Steam');
    expect(root.querySelector('#ach-grid')).toBeNull();
  });

  it('computes the accuracy and copes with no shots', () => {
    expect(accuracy(sampleProfile)).toBe(75);
    expect(accuracy({ ...sampleProfile, shotsFired: 0, shotsMissed: 0 })).toBe(0);
  });

  it('filters the achievements by state and by search text', () => {
    const root = html(renderProfile(sampleProfile));
    bindProfile(root);
    const visible = (): number => [...root.querySelectorAll<HTMLElement>('.ach')].filter((c) => !c.hidden).length;
    const filter = (name: string): void => {
      root.querySelector<HTMLElement>(`[data-ach-filter="${name}"]`)?.click();
    };
    expect(visible()).toBe(3);
    filter('done');
    expect(visible()).toBe(1);
    expect(root.querySelector('[data-ach-filter="done"]')?.getAttribute('aria-pressed')).toBe('true');
    expect(root.querySelector('[data-ach-filter="all"]')?.getAttribute('aria-pressed')).toBe('false');
    filter('todo');
    expect(visible()).toBe(2);
    filter('all');
    const search = root.querySelector<HTMLInputElement>('#ach-search');
    if (!search) throw new Error('busca ausente');
    search.value = 'MARATONA';
    search.dispatchEvent(new Event('input'));
    expect(visible()).toBe(1);
    expect(root.querySelector<HTMLElement>('#ach-empty')?.hidden).toBe(true);
    search.value = 'nada disso existe';
    search.dispatchEvent(new Event('input'));
    expect(visible()).toBe(0);
    expect(root.querySelector<HTMLElement>('#ach-empty')?.hidden).toBe(false);
  });

  it('does nothing when there is no achievements grid', () => {
    const root = html(renderProfile({ ...sampleProfile, hasAchievements: false, achievements: [] }));
    expect(() => {
      bindProfile(root);
    }).not.toThrow();
  });
});

describe('renderMissions', () => {
  const covers = new Map<number, string | null>([[2, 'data:image/jpeg;base64,AAAA']]);

  it('shows the overall progress, one card per reserve and the active missions first', () => {
    const root = html(renderMissions(sampleMissions, covers, 2, 'active'));
    expect(root.textContent).toContain('1 de 4 missões concluídas em 2 reservas');
    expect(root.querySelectorAll('.reserve-card')).toHaveLength(2);
    expect(root.querySelectorAll('.reserve-card img')).toHaveLength(1);
    expect(root.querySelector('.reserve-card[aria-pressed="true"]')?.textContent).toContain('Medved-Taiga');
    expect(root.textContent).toContain('História principal: 1 de 3');
    expect(root.textContent).toContain('2 em andamento');
    expect(root.querySelectorAll('.mission')).toHaveLength(2);
    expect(root.querySelector('[data-filter="active"]')?.getAttribute('aria-pressed')).toBe('true');
  });

  it('falls back to a numbered title for a mission the game has no text for', () => {
    const root = html(renderMissions(sampleMissions, covers, 2, 'active'));
    expect(root.textContent).toContain('Missão 3');
    expect(root.textContent).toContain('Fotografe um alce.');
  });

  it('marks the story missions and shows how many objectives are done', () => {
    const root = html(renderMissions(sampleMissions, covers, 2, 'all'));
    expect(root.querySelectorAll('.mission')).toHaveLength(4);
    expect(root.querySelectorAll('.mission .badge')).toHaveLength(3);
    const objectives = [...root.querySelectorAll('.objectives .muted')].map((n) => n.textContent);
    expect(objectives).toEqual(['1/1', '0/2', '1/1']);
    expect(root.querySelectorAll('.mission-locked')).toHaveLength(1);
  });

  it('filters by state and says so when nothing matches', () => {
    expect(filterMissions(sampleMissions[0]?.missions ?? [], 'done')).toHaveLength(1);
    expect(filterMissions(sampleMissions[0]?.missions ?? [], 'all')).toHaveLength(4);
    const empty = html(renderMissions(sampleMissions, covers, 0, 'done'));
    expect(empty.textContent).toContain('Nenhuma missão com esse filtro');
  });

  it('opens the first reserve when the chosen one is unknown and copes with no missions at all', () => {
    const root = html(renderMissions(sampleMissions, covers, 99, 'all'));
    expect(root.querySelector('.reserve-summary')?.textContent).toContain('Medved-Taiga');
    const none = html(renderMissions([], covers, null, 'all'));
    expect(none.textContent).toContain('Ainda não há missões');
  });

  it('has a progress bar per objective with the counts as tooltip', () => {
    const root = html(renderMissions(sampleMissions, covers, 2, 'active'));
    const titles = [...root.querySelectorAll('.bar')].map((b) => b.getAttribute('title'));
    expect(titles).toEqual(['1/2', '0/1']);
  });
});

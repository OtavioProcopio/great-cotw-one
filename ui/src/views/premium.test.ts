import { describe, expect, it } from 'vitest';

import { PRICES, yearlyDiscountPercent, yearlyPerMonth, yearlySaving } from '../config';
import { formatMoney } from '../format';
import { ACTIVE_LICENSE, FREE_LICENSE } from '../test-data';
import { renderPremium } from './premium';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

describe('prices', () => {
  it('are 15 reais a month or 75 reais a year, and the yearly plan saves 105 reais', () => {
    expect(PRICES).toEqual({ monthly: 15, yearly: 75 });
    expect(yearlySaving()).toBe(105);
    expect(yearlyDiscountPercent()).toBe(58);
    expect(yearlyPerMonth()).toBe(6.25);
  });

  it('are shown in reais', () => {
    expect(formatMoney(15)).toMatch(/^R\$\s15,00$/);
    expect(formatMoney(6.25)).toMatch(/^R\$\s6,25$/);
  });
});

describe('renderPremium', () => {
  it('sells the advantages over the free version', () => {
    const root = html(renderPremium(FREE_LICENSE));
    expect(root.querySelector('h1')?.textContent).toBe('Great COTW One Premium');
    expect(root.querySelectorAll('.benefit')).toHaveLength(4);
    expect(root.textContent).toContain('O que você ganha sendo Premium');
  });

  it('compares what each version does, with the editing features only in Premium', () => {
    const root = html(renderPremium(FREE_LICENSE));
    const rows = [...root.querySelectorAll('table.compare tbody tr')];
    expect(rows).toHaveLength(8);
    const marks = (row: Element | undefined): string[] =>
      [...(row?.querySelectorAll('.mark-yes, .mark-no, .badge') ?? [])].map((m) => m.className.split(' ')[1] ?? '');
    // Mapa: os dois têm; liberar zonas: só o Premium.
    expect(marks(rows[0])).toEqual(['mark-yes', 'mark-yes']);
    expect(marks(rows[4])).toEqual(['mark-no', 'mark-yes']);
    expect(rows[6]?.textContent).toContain('Editar peso, pontuação, sexo e Great One');
    expect(marks(rows[6])).toEqual(['mark-no', 'mark-yes']);
  });

  it('marks the fur choice as coming soon instead of promising it', () => {
    const rows = [...html(renderPremium(FREE_LICENSE)).querySelectorAll('table.compare tbody tr')];
    expect(rows[7]?.textContent).toContain('pelagem');
    expect(rows[7]?.querySelector('.badge')?.textContent).toBe('Em breve');
  });

  it('offers the monthly and the yearly plans with the saving of the yearly one', () => {
    const root = html(renderPremium(FREE_LICENSE));
    const plans = [...root.querySelectorAll('.plan')];
    expect(plans).toHaveLength(2);
    expect(plans[0]?.textContent).toMatch(/R\$\s15,00/);
    expect(plans[0]?.textContent).toContain('por mês');
    expect(plans[1]?.textContent).toMatch(/R\$\s75,00/);
    expect(plans[1]?.textContent).toContain('por ano');
    expect(plans[1]?.querySelector('.ribbon')?.textContent).toContain('58% menos');
    expect(plans[1]?.textContent).toMatch(/economiza R\$\s105,00/);
    expect(plans[1]?.textContent).toMatch(/R\$\s6,25 por mês/);
    expect(root.querySelector('[data-action="buy"][data-plan="monthly"]')).not.toBeNull();
    expect(root.querySelector('[data-action="buy"][data-plan="yearly"]')).not.toBeNull();
  });

  it('marks the plan the player already has', () => {
    const monthly = html(renderPremium({ ...ACTIVE_LICENSE, plan: 'monthly' }));
    expect(monthly.querySelectorAll('.plan .badge-ok')).toHaveLength(1);
    expect(monthly.querySelector('.plan:not(.plan-best) .badge-ok')).not.toBeNull();
    expect(html(renderPremium(FREE_LICENSE)).querySelector('.plan .badge-ok')).toBeNull();
  });

  it('ends with the place to paste the key', () => {
    const root = html(renderPremium(FREE_LICENSE));
    expect(root.querySelector('#license-key')).not.toBeNull();
    expect(root.textContent).toContain('funciona sem internet');
  });
});

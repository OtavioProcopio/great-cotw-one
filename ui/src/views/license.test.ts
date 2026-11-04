import { describe, expect, it } from 'vitest';

import { ACTIVE_LICENSE, FREE_LICENSE, foundDiagnostics } from '../test-data';
import type { License } from '../types';
import { renderHome } from './home';
import { renderLicense } from './license';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

describe('renderLicense', () => {
  it('invites a free user to paste a key and shows the yearly price', () => {
    const root = html(renderLicense(FREE_LICENSE));
    expect(root.textContent).toContain('modo gratuito');
    expect(root.textContent).toContain('não envia nenhum dado seu');
    expect(root.querySelector('#license-key')).not.toBeNull();
    expect(root.querySelector('[data-action="activate-license"]')).not.toBeNull();
    expect(root.querySelector('[data-action="remove-license"]')).toBeNull();
    expect(root.querySelector('.badge-muted')?.textContent).toBe('Gratuito');
  });

  it('shows who the license is for and until when, and lets the player remove it', () => {
    const root = html(renderLicense(ACTIVE_LICENSE));
    expect(root.textContent).toContain('Premium ativo até');
    expect(root.textContent).toContain('ana@exemplo.com');
    expect(root.querySelector('[data-action="remove-license"]')).not.toBeNull();
    expect(root.querySelector('.badge-ok')?.textContent).toBe('Premium');
  });

  it('warns when the license is about to expire or has expired', () => {
    const soon: License = { ...ACTIVE_LICENSE, state: 'expiring', daysLeft: 12 };
    const soonRoot = html(renderLicense(soon));
    expect(soonRoot.textContent).toContain('vence em 12 dias');
    expect(soonRoot.querySelector('.badge-warn')).not.toBeNull();
    const expired: License = { ...ACTIVE_LICENSE, premium: false, state: 'expired', daysLeft: 0 };
    const expiredRoot = html(renderLicense(expired));
    expect(expiredRoot.textContent).toContain('nada foi apagado');
    expect(expiredRoot.querySelector('.badge-warn')?.textContent).toBe('Gratuito');
  });

  it('shows the plan next to the e-mail', () => {
    expect(html(renderLicense(ACTIVE_LICENSE)).textContent).toContain('Plano anual');
    expect(html(renderLicense({ ...ACTIVE_LICENSE, plan: 'monthly' })).textContent).toContain('Plano mensal');
    expect(html(renderLicense({ ...ACTIVE_LICENSE, plan: null })).textContent).not.toContain('Plano');
  });

  it('escapes the e-mail', () => {
    const root = html(renderLicense({ ...ACTIVE_LICENSE, email: '<b>x</b>@a.com' }));
    expect(root.querySelector('b')).toBeNull();
  });
});

describe('license in other screens', () => {
  it('reminds the player to renew on the home screen', () => {
    const soon = html(renderHome(foundDiagnostics, [], { ...ACTIVE_LICENSE, state: 'expiring', daysLeft: 9 }));
    expect(soon.querySelector('.notice-warn')?.textContent).toContain('vence em 9 dias');
    const expired = html(renderHome(foundDiagnostics, [], { ...FREE_LICENSE, state: 'expired' }));
    expect(expired.querySelector('.notice-warn')?.textContent).toContain('venceu');
    expect(html(renderHome(foundDiagnostics, [], ACTIVE_LICENSE)).querySelector('.notice-warn')).toBeNull();
    expect(html(renderHome(foundDiagnostics)).querySelector('.notice-warn')).toBeNull();
  });
});

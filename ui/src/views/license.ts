import { escapeHtml as e } from '../escape';
import { formatDate } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { License } from '../types';

function planName(license: License): string {
  if (license.plan === 'monthly') return t('license.plan.monthly');
  return license.plan === 'yearly' ? t('license.plan.yearly') : '';
}

function statusText(license: License): string {
  const date = license.expires === null ? '' : formatDate(license.expires);
  switch (license.state) {
    case 'active':
      return t('license.active', { date });
    case 'expiring':
      return t('license.expiring', { date, days: license.daysLeft ?? 0 });
    case 'expired':
      return t('license.expired', { date });
    case 'free':
      return t('license.free');
  }
}

/** Cartão de Ajustes onde o jogador vê a licença Premium e cola a chave para ativar ou renovar. */
export function renderLicense(license: License): string {
  const plan = planName(license);
  const email = license.email
    ? `<p class="muted">${e(t('license.email', { email: license.email }))}${plan === '' ? '' : ` · ${e(plan)}`}</p>`
    : '';
  const tone = license.state === 'expired' || license.state === 'expiring' ? 'warn' : license.premium ? 'ok' : 'muted';
  const remove = license.state === 'free'
    ? ''
    : `<button data-action="remove-license">${e(t('license.remove'))}</button>`;
  return `<section class="card" aria-labelledby="license-title">
    <h2 id="license-title" class="card-title"><span class="card-icon" aria-hidden="true">${icon('star', 18)}</span>${e(t('license.title'))}
      <span class="badge badge-${tone}">${e(license.premium ? t('premium.badge') : t('license.badgeFree'))}</span></h2>
    <p>${e(statusText(license))}</p>${email}
    <label class="lbl" for="license-key">${e(t('license.keyLabel'))}</label>
    <textarea id="license-key" rows="3" spellcheck="false" placeholder="${e(t('license.keyPlaceholder'))}"></textarea>
    <div class="actions"><button class="primary" data-action="activate-license">${e(t('license.activate'))}</button>${remove}</div>
    <p class="muted fine">${e(t('license.buy'))}</p>
    <p class="muted fine">${e(t('license.privacy'))}</p>
  </section>`;
}

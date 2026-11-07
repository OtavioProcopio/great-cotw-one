import { PRICES, yearlyDiscountPercent, yearlyPerMonth, yearlySaving } from '../config';
import { escapeHtml as e } from '../escape';
import { formatMoney } from '../format';
import { t, type MessageKey } from '../i18n';
import { icon, type IconName } from '../icons';
import type { License } from '../types';
import { renderLicense } from './license';

interface Feature {
  label: MessageKey;
  free: boolean;
  /** `true` se já funciona; `soon` quando ainda está sendo feito. */
  premium: true | 'soon';
}

const FEATURES: Feature[] = [
  { label: 'premium.f.map', free: true, premium: true },
  { label: 'premium.f.missing', free: true, premium: true },
  { label: 'premium.f.wiki', free: true, premium: true },
  { label: 'premium.f.backups', free: true, premium: true },
  { label: 'premium.f.unlock', free: false, premium: true },
  { label: 'premium.f.unlockBy', free: false, premium: true },
  { label: 'premium.f.edit', free: false, premium: true },
  { label: 'premium.f.gear', free: true, premium: true },
  { label: 'premium.f.gearEdit', free: false, premium: true },
  { label: 'premium.f.fur', free: false, premium: 'soon' },
];

const BENEFITS: { symbol: IconName; title: MessageKey; text: MessageKey }[] = [
  { symbol: 'unlock', title: 'premium.b1.title', text: 'premium.b1.text' },
  { symbol: 'target', title: 'premium.b2.title', text: 'premium.b2.text' },
  { symbol: 'star', title: 'premium.b3.title', text: 'premium.b3.text' },
  { symbol: 'shield', title: 'premium.b4.title', text: 'premium.b4.text' },
];

function mark(on: boolean): string {
  return on
    ? `<span class="mark mark-yes" aria-label="${e(t('premium.yes'))}">${icon('check', 16)}</span>`
    : `<span class="mark mark-no" aria-label="${e(t('premium.no'))}">${icon('minus', 16)}</span>`;
}

function featureRow(feature: Feature): string {
  const premium =
    feature.premium === 'soon'
      ? `<span class="badge badge-muted">${e(t('home.soon'))}</span>`
      : mark(true);
  return `<tr><th scope="row">${e(t(feature.label))}</th><td>${mark(feature.free)}</td><td class="col-premium">${premium}</td></tr>`;
}

function planCard(plan: 'monthly' | 'yearly', current: License['plan'], waiting: boolean): string {
  const yearly = plan === 'yearly';
  const price = formatMoney(yearly ? PRICES.yearly : PRICES.monthly);
  const ribbon = yearly
    ? `<span class="ribbon">${e(t('premium.bestValue', { percent: yearlyDiscountPercent() }))}</span>`
    : '';
  const extra = yearly
    ? `<p class="plan-extra">${e(t('premium.saving', { money: formatMoney(yearlySaving()), permonth: formatMoney(yearlyPerMonth()) }))}</p>`
    : `<p class="plan-extra">${e(t('premium.monthlyNote'))}</p>`;
  const yours = current === plan ? `<span class="badge badge-ok">${e(t('premium.yourPlan'))}</span>` : '';
  return `<li class="plan${yearly ? ' plan-best' : ''}">${ribbon}
    <h3>${e(t(yearly ? 'premium.yearly' : 'premium.monthly'))} ${yours}</h3>
    <p class="price"><strong>${e(price)}</strong><span>${e(t(yearly ? 'premium.perYear' : 'premium.perMonth'))}</span></p>
    ${extra}
    <button class="primary${yearly ? ' gold' : ''}" data-action="buy" data-plan="${plan}"${waiting ? ' disabled' : ''}>${e(t(yearly ? 'premium.buyYearly' : 'premium.buyMonthly'))}</button>
  </li>`;
}

function waitingBanner(plan: 'monthly' | 'yearly' | null): string {
  if (plan === null) return '';
  return `<div class="card purchase-waiting" role="status">
    <p><strong>${e(t('purchase.waiting'))}</strong></p>
    <p class="muted">${e(t('purchase.waitingHint'))}</p>
    <div class="actions"><button type="button" data-action="cancel-purchase">${e(t('purchase.cancel'))}</button></div>
  </div>`;
}

/** Tela do Premium: o que muda em relação ao gratuito, os planos e onde colar a chave. */
export function renderPremium(license: License, waitingPlan: 'monthly' | 'yearly' | null = null): string {
  const hero = `<header class="premium-hero">
      <span class="premium-icon big" aria-hidden="true">${icon('star', 30)}</span>
      <div><h1>${e(t('premium.title'))}</h1><p class="lead">${e(t('premium.tagline'))}</p></div>
    </header>`;
  const benefits = BENEFITS.map(
    (b) => `<li class="benefit"><span class="benefit-icon" aria-hidden="true">${icon(b.symbol, 22)}</span>
      <div><h3>${e(t(b.title))}</h3><p class="muted">${e(t(b.text))}</p></div></li>`,
  ).join('');
  const table = `<table class="compare">
      <thead><tr><th scope="col">${e(t('premium.compare'))}</th><th scope="col">${e(t('license.badgeFree'))}</th><th scope="col" class="col-premium">${e(t('premium.badge'))}</th></tr></thead>
      <tbody>${FEATURES.map(featureRow).join('')}</tbody></table>`;
  return `${hero}
    <section aria-labelledby="benefits-title"><h2 id="benefits-title">${e(t('premium.whatYouGet'))}</h2>
      <ul class="benefits">${benefits}</ul></section>
    <section aria-labelledby="compare-title"><h2 id="compare-title">${e(t('premium.versus'))}</h2>
      <div class="card">${table}</div></section>
    <section aria-labelledby="plans-title"><h2 id="plans-title">${e(t('premium.choose'))}</h2>
      ${waitingBanner(waitingPlan)}
      <ul class="plans">${planCard('monthly', license.plan, waitingPlan !== null)}${planCard('yearly', license.plan, waitingPlan !== null)}</ul>
      <p class="muted fine">${e(t('premium.payNote'))}</p></section>
    ${renderLicense(license)}`;
}

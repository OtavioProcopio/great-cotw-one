import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import type { ReserveSummary, ZonesPreview } from '../types';
import {
  type UnlockMode,
  type UnlockState,
  chosenReserves,
  readyReserves,
  summarize,
} from '../unlocklogic';

const NEEDS: { kind: number; symbol: IconName; label: 'map.need.1' | 'map.need.2' | 'map.need.3' }[] = [
  { kind: 1, symbol: 'leaf', label: 'map.need.1' },
  { kind: 2, symbol: 'drop', label: 'map.need.2' },
  { kind: 3, symbol: 'moon', label: 'map.need.3' },
];

function detail(reserve: ReserveSummary): string {
  switch (reserve.state) {
    case 'ready': {
      const approx =
        reserve.estimated > 0 ? ` · ${t('unlock.estimated', { count: formatCount(reserve.estimated) })}` : '';
      return t('unlock.ready', { count: formatCount(reserve.newZones) }) + approx;
    }
    case 'nothingToDo':
      return t('unlock.nothing');
    case 'needsDiscovery':
      return t('unlock.needsDiscovery');
    case 'noData':
      return t('unlock.noData');
  }
}

function renderRow(reserve: ReserveSummary, state: UnlockState): string {
  const checked = state.reserves === null || state.reserves.has(reserve.id);
  const pick =
    reserve.state === 'ready'
      ? `<input type="checkbox" class="reserve-pick" data-action="toggle-reserve" data-reserve="${String(reserve.id)}" value="${String(reserve.id)}"${checked ? ' checked' : ''} />`
      : '<span class="pick-spacer" aria-hidden="true"></span>';
  return `<li class="reserve-row">
    <label>${pick}<strong>${e(reserve.name)}</strong></label>
    <span class="muted">${e(detail(reserve))}</span>
  </li>`;
}

function modeButton(mode: UnlockMode, current: UnlockMode, label: string): string {
  return `<button class="seg" data-action="unlock-mode" data-mode="${mode}" aria-pressed="${String(mode === current)}">${e(label)}</button>`;
}

function renderChoices(preview: ZonesPreview, state: UnlockState): string {
  const totals = summarize(preview, state);
  if (state.mode === 'animal') {
    const chips = totals.species
      .map(
        (s) =>
          `<button class="chip-link" data-action="toggle-species" data-species="${e(s.id)}" aria-pressed="${String(state.species.has(s.id))}">${e(s.name)} <span class="count">${e(formatCount(s.count))}</span></button>`,
      )
      .join('');
    return `<p class="muted">${e(t('unlock.pickAnimals'))}</p><div class="chip-links">${chips}</div>`;
  }
  if (state.mode === 'need') {
    const chips = NEEDS.map(
      (n) =>
        `<button class="chip-link" data-action="toggle-need" data-need="${String(n.kind)}" aria-pressed="${String(state.needs.has(n.kind))}">${icon(n.symbol, 14)} ${e(t(n.label))} <span class="count">${e(formatCount(totals.needs[n.kind - 1] ?? 0))}</span></button>`,
    ).join('');
    return `<p class="muted">${e(t('unlock.pickNeeds'))}</p><div class="chip-links">${chips}</div>`;
  }
  return '';
}

function renderAction(total: number, reserves: number, premium: boolean): string {
  if (total === 0) return `<p class="muted">${e(t('unlock.nothingChosen'))}</p>`;
  const summary = `<p>${e(t('unlock.summary', { count: formatCount(total), reserves }))}</p>
    <p class="muted">${e(t('unlock.approx'))}</p>`;
  if (premium) {
    return `${summary}<div class="actions"><button class="primary" data-action="unlock-run">${e(t('unlock.run'))}</button></div>`;
  }
  return `${summary}<div class="actions"><button class="primary locked" data-action="premium-info">${icon('lock', 16)}<span>${e(t('unlock.runPremium'))}</span></button></div>`;
}

function premiumBanner(): string {
  return `<section class="card premium-banner" aria-label="${e(t('premium.badge'))}">
    <span class="premium-icon" aria-hidden="true">${icon('star', 22)}</span>
    <div><h2>${e(t('unlock.premiumTitle'))}</h2><p class="muted">${e(t('unlock.premiumText'))}</p></div>
    <button class="primary locked" data-action="show-premium">${e(t('unlock.knowPremium'))}</button>
  </section>`;
}

/** Tela de liberar o mapa; `preview` é `null` enquanto o app analisa o progresso. */
export function renderUnlock(preview: ZonesPreview | null, state: UnlockState, premium: boolean): string {
  const header = `<header class="view-header">
      <div><h1>${e(t('unlock.title'))}</h1><p class="lead">${e(t('unlock.intro'))}</p></div>
      <button data-action="show-home">${icon('back', 16)}<span>${e(t('unlock.back'))}</span></button>
    </header>`;
  if (!preview) return `${header}<p class="muted">${e(t('unlock.analysing'))}</p>`;
  const banner = premium ? '' : premiumBanner();
  const list = `<section class="card"><ul class="reserve-list">${preview.reserves.map((r) => renderRow(r, state)).join('')}</ul></section>`;
  if (readyReserves(preview).length === 0) {
    return `${header}${banner}${list}<p class="muted">${e(t('unlock.none'))}</p>`;
  }
  if (chosenReserves(preview, state).length === 0) {
    return `${header}${banner}${list}<p class="muted">${e(t('unlock.noReserve'))}</p>`;
  }
  const totals = summarize(preview, state);
  const modes = `<div class="segmented">${modeButton('all', state.mode, t('unlock.modeAll'))}${modeButton('animal', state.mode, t('unlock.modeAnimal'))}${modeButton('need', state.mode, t('unlock.modeNeed'))}</div>`;
  return `${header}${banner}${list}
    <section class="card"><h2>${e(t('unlock.what'))}</h2>${modes}${renderChoices(preview, state)}</section>
    <section class="card">${renderAction(totals.total, totals.reserves, premium)}</section>`;
}

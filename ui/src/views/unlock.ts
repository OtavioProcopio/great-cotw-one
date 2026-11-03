import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { ReserveSummary, ZonesPreview } from '../types';

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

function renderRow(reserve: ReserveSummary): string {
  const pick =
    reserve.state === 'ready'
      ? `<input type="checkbox" class="reserve-pick" value="${String(reserve.id)}" checked />`
      : '<span class="pick-spacer" aria-hidden="true"></span>';
  return `<li class="reserve-row">
    <label>${pick}<strong>${e(reserve.name)}</strong></label>
    <span class="muted">${e(detail(reserve))}</span>
  </li>`;
}

/** Tela de liberar o mapa; `preview` é `null` enquanto o app analisa o progresso. */
export function renderUnlock(preview: ZonesPreview | null): string {
  const header = `<header class="view-header">
      <div><h1>${e(t('unlock.title'))}</h1><p class="muted">${e(t('unlock.intro'))}</p></div>
      <button data-action="show-home">${icon('back', 16)}<span>${e(t('unlock.back'))}</span></button>
    </header>`;
  if (!preview) return `${header}<p class="muted">${e(t('unlock.analysing'))}</p>`;
  const ready = preview.reserves.filter((r) => r.state === 'ready');
  const footer =
    ready.length === 0
      ? `<p class="muted">${e(t('unlock.none'))}</p>`
      : `<p>${e(t('unlock.summary', { count: formatCount(preview.total), reserves: ready.length }))}</p>
         <p class="muted">${e(t('unlock.approx'))}</p>
         <div class="actions"><button class="primary" data-action="unlock-run">${e(t('unlock.run'))}</button></div>`;
  return `${header}<section class="card"><ul class="reserve-list">${preview.reserves.map(renderRow).join('')}</ul>${footer}</section>`;
}

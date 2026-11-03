import { escapeHtml as e } from '../escape';
import { formatStamp } from '../format';
import { t } from '../i18n';
import type { HistoryItem } from '../types';

function renderItem(item: HistoryItem): string {
  const files = item.files.length === 0 ? t('history.noFiles') : item.files.join(', ');
  const restore = item.backupMissing
    ? `<p class="notice notice-warn">${e(t('history.backupMissing'))}</p>`
    : `<div class="actions"><button class="primary" data-action="restore" data-backup="${e(item.backup)}" data-at="${e(item.at)}">${e(t('history.restore'))}</button></div>`;
  return `<li class="card history-item">
    <div class="history-head">
      <span class="badge badge-${item.kind === 'restore' ? 'muted' : 'ok'}">${e(t(`history.kind.${item.kind}`))}</span>
      <strong>${e(item.function)}</strong>
      <time class="muted">${e(formatStamp(item.at))}</time>
    </div>
    <p>${e(item.summary)}</p>
    ${restore}
    <details class="details"><summary>${e(t('history.details'))}</summary>
      <p class="muted">${e(t('history.files', { files }))}</p>
      <p class="muted path">${e(t('history.backup', { path: item.backup }))}</p>
    </details>
  </li>`;
}

/** Tela de histórico, com a restauração de cada backup. */
export function renderHistory(items: HistoryItem[]): string {
  const list =
    items.length === 0
      ? `<p class="muted">${e(t('history.empty'))}</p>`
      : `<ul class="history-list">${items.map(renderItem).join('')}</ul>`;
  return `<header class="view-header">
      <div><h1>${e(t('history.title'))}</h1><p class="muted">${e(t('history.intro'))}</p></div>
    </header>${list}`;
}

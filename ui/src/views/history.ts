import { escapeHtml as e } from '../escape';
import { formatStamp } from '../format';
import { type MessageKey, t } from '../i18n';
import type { HistoryItem } from '../types';

/** Nomes de funções que o app grava no histórico, traduzidos para o idioma atual. */
const FUNCTIONS: Record<string, MessageKey> = {
  'Liberar o mapa': 'history.fn.unlock',
  'Editar animais': 'history.fn.edit',
  'Editar equipamentos': 'history.fn.equipment',
  'Teste de gravação': 'history.fn.test',
  'Restaurar backup': 'history.fn.restore',
};

/**
 * O histórico guarda o resumo em português (é assim desde as primeiras versões, e o arquivo continua
 * válido). Na hora de mostrar, os resumos conhecidos são traduzidos para o idioma atual.
 */
const SUMMARIES: [RegExp, (m: RegExpMatchArray) => string][] = [
  [/^(\d+) zonas liberadas em (\d+) reservas$/, (m) => t('history.sum.zones', { zones: m[1] ?? '', reserves: m[2] ?? '' })],
  [/^(\d+) mudanças nos equipamentos$/, (m) => t('history.sum.equipment', { count: m[1] ?? '' })],
  [/^(\d+) animais editados \((.+)\)$/, (m) => t('history.sum.animals', { count: m[1] ?? '', species: m[2] ?? '' })],
  [/^(.+) regravado sem mudanças$/, (m) => t('history.sum.rewrite', { file: m[1] ?? '' })],
  [/^Pasta do save restaurada a partir de (.+)$/, (m) => t('history.sum.restore', { path: m[1] ?? '' })],
];

export function summaryText(summary: string): string {
  for (const [pattern, text] of SUMMARIES) {
    const match = pattern.exec(summary);
    if (match) return text(match);
  }
  return summary;
}

function functionName(name: string): string {
  const key = FUNCTIONS[name];
  return key === undefined ? name : t(key);
}

function renderItem(item: HistoryItem): string {
  const files = item.files.length === 0 ? t('history.noFiles') : item.files.join(', ');
  const restore = item.backupMissing
    ? `<p class="notice notice-warn">${e(t('history.backupMissing'))}</p>`
    : `<div class="actions"><button class="primary" data-action="restore" data-backup="${e(item.backup)}" data-at="${e(item.at)}">${e(t('history.restore'))}</button></div>`;
  return `<li class="card history-item">
    <div class="history-head">
      <span class="badge badge-${item.kind === 'restore' ? 'muted' : 'ok'}">${e(t(`history.kind.${item.kind}`))}</span>
      <strong>${e(functionName(item.function))}</strong>
      <time class="muted">${e(formatStamp(item.at))}</time>
    </div>
    <p>${e(summaryText(item.summary))}</p>
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

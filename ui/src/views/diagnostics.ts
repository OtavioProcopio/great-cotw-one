import { escapeHtml as e } from '../escape';
import { formatBytes, formatEpoch } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import { renderLanguage } from './language';
import type { AccountDiagnostic, Diagnostics, FileDiagnostic, GameDiagnostic, SaveDiagnostic } from '../types';

function badge(kind: 'ok' | 'warn' | 'muted', text: string): string {
  return `<span class="badge badge-${kind}">${e(text)}</span>`;
}

function heading(id: string, symbol: IconName, text: string): string {
  return `<h2 id="${id}" class="card-title"><span class="card-icon" aria-hidden="true">${icon(symbol, 18)}</span>${e(text)}</h2>`;
}

function renderGame(game: GameDiagnostic, limited: boolean): string {
  const body = game.found
    ? `<dl class="facts">
        <dt>${e(t('game.path'))}</dt><dd class="path">${e(game.root ?? '')}</dd>
        <dt>${e(t('game.archives'))}</dt><dd>${e(t('game.tabFiles', { count: game.tabFiles }))}</dd>
        <dt>${e(t('game.buildLabel'))}</dt><dd>${e(game.buildId ?? t('game.buildUnknown'))}</dd>
      </dl>`
    : '';
  const manual = game.found && game.manual ? badge('muted', t('game.manual')) : '';
  const warning = limited ? `<p class="notice notice-warn">${e(t('game.limited'))}</p>` : '';
  return `<section class="card" aria-labelledby="game-title">
    ${heading('game-title', 'game', t('game.title'))}
    <p>${badge(game.found ? 'ok' : 'warn', t(game.found ? 'game.found' : 'game.notFound'))} ${manual}</p>
    ${body}${warning}
    <div class="actions"><button data-action="choose-game-folder">${icon('folder', 16)}<span>${e(t('game.choose'))}</span></button></div>
  </section>`;
}

function accountLine(account: AccountDiagnostic): string {
  const modified =
    account.lastModified === null
      ? ''
      : ` · ${e(t('save.modified', { date: formatEpoch(account.lastModified) }))}`;
  return `<strong>${e(t('save.account', { id: account.steamId }))}</strong>${modified}
    <div class="path">${e(account.path)}</div>`;
}

function renderSave(save: SaveDiagnostic): string {
  let body: string;
  if (save.status === 'found' && save.account) {
    const manual = save.manual ? badge('muted', t('save.manual')) : '';
    body = `<p>${badge('ok', t('save.found'))} ${manual}</p><p>${accountLine(save.account)}</p>`;
  } else if (save.status === 'choose') {
    const items = save.accounts
      .map(
        (account) => `<li class="choice">
          <div>${accountLine(account)}</div>
          <button data-action="use-account" data-path="${e(account.path)}">${e(t('save.use'))}</button>
        </li>`,
      )
      .join('');
    body = `<p>${e(t('save.chooseIntro', { count: save.accounts.length }))}</p><ul class="choices">${items}</ul>`;
  } else {
    const searched = save.searched.map((path) => `<li class="path">${e(path)}</li>`).join('');
    body = `<p>${badge('warn', t('save.notFound'))}</p>
      <p>${e(t('save.notFoundHelp'))}</p><ul>${searched}</ul>`;
  }
  const reset = save.manual
    ? `<button data-action="reset-choices">${e(t('save.reset'))}</button>`
    : '';
  return `<section class="card" aria-labelledby="save-title">
    ${heading('save-title', 'save', t('save.title'))}
    ${body}
    <div class="actions"><button data-action="choose-save-folder">${icon('folder', 16)}<span>${e(t('save.choose'))}</span></button>${reset}</div>
  </section>`;
}

function renderReserves(d: Diagnostics): string {
  const items = d.reserves
    .map((r) => `<li><span class="chip">${r.id}</span> ${e(r.name)}</li>`)
    .join('');
  return `<section class="card" aria-labelledby="reserves-title">
    ${heading('reserves-title', 'map', t('reserves.title'))}
    ${d.reserves.length === 0 ? `<p class="muted">${e(t('reserves.empty'))}</p>` : `<ul class="chips">${items}</ul>`}
  </section>`;
}

function kindText(file: FileDiagnostic): string {
  const reserve = file.reserve ?? '';
  switch (file.kind) {
    case 'population':
      return t('files.kind.population', { reserve });
    case 'foundIcons':
      return t('files.kind.foundIcons', { reserve });
    case 'fogOfWar':
      return t('files.kind.fogOfWar', { reserve });
    case 'known':
      return t('files.kind.known');
    case 'unrecognized':
      return t('files.kind.unrecognized');
  }
}

function stateBadge(file: FileDiagnostic): string {
  switch (file.state) {
    case 'ok':
      return badge('ok', t('files.state.ok'));
    case 'unreadable':
      return badge('warn', t('files.state.unreadable'));
    case 'unrecognized':
      return badge('muted', t('files.state.unrecognized'));
  }
}

function renderFiles(files: FileDiagnostic[]): string {
  const ok = files.filter((f) => f.state === 'ok').length;
  const rows = files
    .map(
      (f) => `<tr>
        <td class="path">${e(f.name)}</td>
        <td>${e(kindText(f))}</td>
        <td class="num">${e(formatBytes(f.size))}</td>
        <td>${stateBadge(f)}</td>
      </tr>`,
    )
    .join('');
  const table =
    files.length === 0
      ? `<p class="muted">${e(t('files.empty'))}</p>`
      : `<div class="table-wrap"><table>
          <thead><tr><th>${e(t('files.colName'))}</th><th>${e(t('files.colKind'))}</th>
          <th class="num">${e(t('files.colSize'))}</th><th>${e(t('files.colState'))}</th></tr></thead>
          <tbody>${rows}</tbody></table></div>`;
  const summary =
    files.length === 0 ? '' : `<p class="muted">${e(t('files.summary', { ok, other: files.length - ok }))}</p>`;
  return `<details class="card files-card">
    <summary>${icon('list', 18)}<span>${e(t('files.title'))}</span></summary>${summary}${table}</details>`;
}

/** Detalhes técnicos: jogo, save, reservas e arquivos reconhecidos. */
export function renderDiagnostics(d: Diagnostics): string {
  return `<header class="view-header">
      <div><h1>${e(t('diag.title'))}</h1><p class="muted">${e(t('diag.intro'))}</p></div>
      <button data-action="refresh">${icon('refresh', 16)}<span>${e(t('common.refresh'))}</span></button>
    </header>
    ${renderLanguage()}
    ${renderGame(d.game, d.limitedMode)}
    ${renderSave(d.save)}
    ${d.save.status === 'found' ? renderReserves(d) + renderFiles(d.files) : ''}`;
}

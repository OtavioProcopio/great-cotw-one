import { escapeHtml as e } from '../escape';
import { formatEpoch } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import type { AccountDiagnostic, Diagnostics, FileDiagnostic, License, MapReserve } from '../types';

/** Arquivo usado no teste de gravação: a população da reserva 2 se existir, senão qualquer população. */
export function defaultRewriteFile(files: FileDiagnostic[]): string | undefined {
  const readable = files.filter((f) => f.state === 'ok');
  return (
    readable.find((f) => f.name === 'animal_population_2') ??
    readable.find((f) => f.kind === 'population') ??
    readable[0]
  )?.name;
}

function statusCard(ok: boolean, symbol: IconName, body: string): string {
  return `<li class="status-card">
    <span class="status-icon status-${ok ? 'ok' : 'warn'}" aria-hidden="true">${icon(symbol, 22)}</span>
    <div class="status-body">${body}</div>
    <span class="status-mark status-${ok ? 'ok' : 'warn'}" aria-hidden="true">${icon(ok ? 'check' : 'alert', 16)}</span>
  </li>`;
}

function renderGameRow(d: Diagnostics): string {
  if (d.game.found) return statusCard(true, 'game', `<p>${e(t('home.game.ok'))}</p>`);
  return statusCard(
    false,
    'game',
    `<p>${e(t('home.game.missing'))}</p>
     <div class="actions"><button data-action="choose-game-folder">${e(t('home.game.choose'))}</button></div>`,
  );
}

function profileLine(account: AccountDiagnostic): string {
  const last =
    account.lastModified === null ? '' : ` · ${e(t('home.save.okLast', { date: formatEpoch(account.lastModified) }))}`;
  return `<strong>${e(t('home.save.profile', { id: account.steamId }))}</strong>${last}`;
}

function renderSaveRow(d: Diagnostics): string {
  const { save } = d;
  if (save.status === 'found' && save.account) {
    const last =
      save.account.lastModified === null
        ? ''
        : `<p class="muted">${e(t('home.save.okLast', { date: formatEpoch(save.account.lastModified) }))}</p>`;
    return statusCard(true, 'save', `<p>${e(t('home.save.ok', { count: d.reserves.length }))}</p>${last}`);
  }
  if (save.status === 'choose') {
    const items = save.accounts
      .map(
        (account) => `<li class="choice">
          <div>${profileLine(account)}</div>
          <button class="primary" data-action="use-account" data-path="${e(account.path)}">${e(t('home.save.use'))}</button>
        </li>`,
      )
      .join('');
    return statusCard(false, 'save', `<p>${e(t('home.save.choose'))}</p><ul class="choices">${items}</ul>`);
  }
  return statusCard(
    false,
    'save',
    `<p>${e(t('home.save.missing'))}</p>
     <div class="actions"><button data-action="choose-save-folder">${e(t('home.save.choose2'))}</button></div>`,
  );
}

function tile(symbol: IconName, tone: string, title: string, text: string, button: string, premium = false): string {
  const badge = premium ? ` <span class="badge badge-premium">${e(t('premium.badge'))}</span>` : '';
  return `<li class="tile tile-${tone}"><span class="tile-icon" aria-hidden="true">${icon(symbol, 26)}</span>
    <h3>${e(title)}${badge}</h3><p class="muted">${e(text)}</p>${button}</li>`;
}

function soonTile(kind: 'guide' | 'animals'): string {
  const badge = `<span class="badge badge-muted">${e(t('home.soon'))}</span>`;
  const symbol: IconName = kind === 'guide' ? 'list' : 'paw';
  return tile(symbol, 'soon', t(`home.soon.${kind}.title`), t(`home.soon.${kind}.text`), badge);
}

function button(action: string, label: string, primary: boolean): string {
  return `<div class="actions"><button${primary ? ' class="primary"' : ''} data-action="${action}">${e(label)}</button></div>`;
}

function renderActions(d: Diagnostics): string {
  if (d.save.status !== 'found') return '';
  const test = defaultRewriteFile(d.files)
    ? tile('shield', 'blue', t('home.test.title'), t('home.test.text'), button('rewrite-unchanged', t('home.test.button'), true))
    : '';
  const map = d.game.found
    ? tile('map', 'green', t('home.map.title'), t('home.map.text'), button('show-map', t('home.map.button'), true))
    : '';
  const unlock = d.game.found
    ? tile('unlock', 'orange', t('home.unlock.title'), t('home.unlock.text'), button('show-unlock', t('home.unlock.button'), true), true)
    : '';
  const backups = tile('backup', 'violet', t('home.backups.title'), t('home.backups.text'), button('show-history', t('home.backups.button'), false));
  return `<section aria-labelledby="actions-title">
    <h2 id="actions-title">${e(t('home.actions.title'))}</h2>
    <ul class="tiles">${map}${unlock}${test}${backups}${soonTile('guide')}${soonTile('animals')}</ul>
  </section>`;
}

function renderReserves(reserves: MapReserve[]): string {
  if (reserves.length === 0) return '';
  const cards = reserves
    .map((r) => {
      const cover = r.cover
        ? `<img src="${e(r.cover)}" alt="" loading="lazy" />`
        : `<span class="cover-empty" aria-hidden="true">${icon('map', 28)}</span>`;
      return `<li><button class="reserve-card" data-action="open-reserve" data-reserve="${String(r.id)}">
        <span class="cover">${cover}</span><span class="reserve-name">${e(r.name)}</span></button></li>`;
    })
    .join('');
  return `<section aria-labelledby="reserves-home-title">
    <h2 id="reserves-home-title">${e(t('home.reserves.title'))}</h2>
    <p class="muted">${e(t('home.reserves.text'))}</p>
    <ul class="reserve-gallery">${cards}</ul>
  </section>`;
}

function renewalNotice(license: License | null): string {
  if (license?.state === 'expiring') {
    return `<p class="notice notice-warn">${e(t('home.renew.soon', { days: license.daysLeft ?? 0 }))}</p>`;
  }
  return license?.state === 'expired' ? `<p class="notice notice-warn">${e(t('home.renew.expired'))}</p>` : '';
}

/** Tela inicial: o que está pronto e o que o jogador pode fazer, sem termos técnicos. */
export function renderHome(d: Diagnostics, reserves: MapReserve[] = [], license: License | null = null): string {
  return `${renewalNotice(license)}<header class="view-header">
      <div><h1>${e(t('home.title'))}</h1><p class="lead">${e(t('home.intro'))}</p></div>
      <button data-action="refresh">${icon('refresh', 16)}<span>${e(t('home.recheck'))}</span></button>
    </header>
    <section aria-labelledby="status-title">
      <h2 id="status-title">${e(t('home.status.title'))}</h2>
      <ul class="status-list">${renderGameRow(d)}${renderSaveRow(d)}</ul>
      <p class="muted safety">${icon('shield', 16)}<span>${e(t('home.safety'))}</span></p>
    </section>
    ${renderReserves(reserves)}
    ${renderActions(d)}`;
}

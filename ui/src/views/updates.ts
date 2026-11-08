import { escapeHtml as e } from '../escape';
import { t } from '../i18n';
import { icon } from '../icons';
import type { UpdateInfo } from '../types';

/** Em que pé está a atualização do app. */
export type UpdatePhase = 'idle' | 'checking' | 'none' | 'available' | 'downloading' | 'error';

export interface UpdateState {
  phase: UpdatePhase;
  /** Versão instalada (vazia enquanto não se sabe). */
  current: string;
  /** Versão nova, quando houver. */
  info: UpdateInfo | null;
  /** Download de 0 a 100. */
  progress: number;
  /** O jogador fechou o aviso desta versão. */
  dismissed: boolean;
  /** Procurar ao abrir o app. */
  auto: boolean;
}

const AUTO_KEY = 'gco.autoUpdate';

/** Procurar atualizações ao abrir vem ligado; o jogador pode desligar em Ajustes. */
export function readAutoUpdate(): boolean {
  try {
    return localStorage.getItem(AUTO_KEY) !== 'off';
  } catch {
    return true;
  }
}

export function saveAutoUpdate(on: boolean): void {
  try {
    localStorage.setItem(AUTO_KEY, on ? 'on' : 'off');
  } catch {
    // Sem armazenamento a escolha vale até fechar o app.
  }
}

export function initialUpdateState(): UpdateState {
  return { phase: 'idle', current: '', info: null, progress: 0, dismissed: false, auto: readAutoUpdate() };
}

/** Faixa no alto da tela quando há versão nova (ou durante o download). */
export function renderUpdateBanner(state: UpdateState): string {
  if (state.phase === 'downloading') {
    return `<div class="update-banner" role="status">${icon('refresh', 18)}
      <span>${e(t('update.downloading', { percent: Math.round(state.progress) }))}</span>
      <progress max="100" value="${String(Math.round(state.progress))}"></progress></div>`;
  }
  if (state.phase !== 'available' || !state.info || state.dismissed) return '';
  return `<div class="update-banner" role="status">${icon('sparkle', 18)}
    <span>${e(t('update.banner', { version: state.info.version }))}</span>
    <button class="primary" data-action="update-install">${e(t('update.install'))}</button>
    <button data-action="update-dismiss">${e(t('update.later'))}</button></div>`;
}

function statusLine(state: UpdateState): string {
  switch (state.phase) {
    case 'checking':
      return t('update.checking');
    case 'none':
      return t('update.none');
    case 'available':
      return state.info ? t('update.available', { version: state.info.version }) : '';
    case 'downloading':
      return t('update.downloading', { percent: Math.round(state.progress) });
    case 'error':
      return t('update.failed');
    case 'idle':
      return '';
  }
}

/** Cartão de Ajustes: versão instalada, procurar agora e procurar ao abrir. */
export function renderUpdatesCard(state: UpdateState): string {
  const status = statusLine(state);
  const notes = state.phase === 'available' && state.info?.notes ? `<p class="update-notes">${e(state.info.notes)}</p>` : '';
  const install =
    state.phase === 'available' ? `<button class="primary" data-action="update-install">${e(t('update.install'))}</button>` : '';
  return `<section class="card" aria-labelledby="update-title">
    <h2 id="update-title" class="card-title"><span class="card-icon" aria-hidden="true">${icon('refresh', 18)}</span>${e(t('update.title'))}</h2>
    <p>${e(t('update.version', { version: state.current || '-' }))}</p>
    ${status ? `<p class="muted" role="status">${e(status)}</p>` : ''}${notes}
    <div class="actions">
      <button data-action="update-check" ${state.phase === 'checking' || state.phase === 'downloading' ? 'disabled' : ''}>${icon('refresh', 16)}<span>${e(t('update.check'))}</span></button>${install}
    </div>
    <label class="switch-line"><input type="checkbox" data-action="update-auto" ${state.auto ? 'checked' : ''} /> ${e(t('update.auto'))}</label>
    <p class="muted small">${e(t('update.privacy'))}</p>
  </section>`;
}

import { escapeHtml as e } from '../escape';
import { LOCALES, getLocale, t } from '../i18n';
import { icon } from '../icons';

/** Cartão de Ajustes para escolher o idioma do app. */
export function renderLanguage(): string {
  const current = getLocale();
  const buttons = LOCALES.map(
    (l) =>
      `<button class="seg" data-action="set-language" data-locale="${l.code}" aria-pressed="${String(l.code === current)}" lang="${l.code}">${e(l.name)}</button>`,
  ).join('');
  return `<section class="card" aria-labelledby="lang-title">
    <h2 id="lang-title" class="card-title"><span class="card-icon" aria-hidden="true">${icon('sparkle', 18)}</span>${e(t('lang.title'))}</h2>
    <p class="muted">${e(t('lang.text'))}</p>
    <div class="segmented">${buttons}</div>
  </section>`;
}

import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import type { Achievement, Profile } from '../types';

const RING_RADIUS = 52;
const RING_LENGTH = 2 * Math.PI * RING_RADIUS;

/** Anel de progresso (0 a 100) com um texto no meio. */
export function ring(percent: number, big: string, small: string): string {
  const clamped = Math.min(100, Math.max(0, percent));
  const offset = RING_LENGTH * (1 - clamped / 100);
  return `<div class="ring" role="img" aria-label="${e(`${big} ${small}`)}">
    <svg viewBox="0 0 120 120" width="120" height="120" aria-hidden="true">
      <circle class="ring-track" cx="60" cy="60" r="${String(RING_RADIUS)}" />
      <circle class="ring-fill" cx="60" cy="60" r="${String(RING_RADIUS)}" stroke-dasharray="${RING_LENGTH.toFixed(1)}" stroke-dashoffset="${offset.toFixed(1)}" transform="rotate(-90 60 60)" />
    </svg>
    <div class="ring-text"><strong>${e(big)}</strong><span>${e(small)}</span></div>
  </div>`;
}

/** Percentual de acertos; sem tiros, zero. */
export function accuracy(profile: Profile): number {
  if (profile.shotsFired === 0) return 0;
  return Math.round(((profile.shotsFired - profile.shotsMissed) / profile.shotsFired) * 100);
}

function metric(symbol: IconName, label: string, value: string): string {
  return `<li class="metric"><span class="metric-icon" aria-hidden="true">${icon(symbol, 20)}</span>
    <div><strong>${e(value)}</strong><span>${e(label)}</span></div></li>`;
}

function renderStats(p: Profile): string {
  const items = [
    metric('target', t('profile.kills'), formatCount(p.totalKills)),
    metric('sparkle', t('profile.oneShot'), formatCount(p.oneShotKills)),
    metric('check', t('profile.accuracy'), `${String(accuracy(p))}%`),
    metric('list', t('profile.shots'), formatCount(p.shotsFired)),
    metric('star', t('profile.record'), formatCount(Math.round(p.recordScore))),
    metric('fit', t('profile.distance'), t('profile.meters', { value: formatCount(Math.round(p.furthestShot)) })),
    metric('trophy', t('profile.trophies'), formatCount(p.trophies)),
    metric('map', t('profile.reserves'), formatCount(p.reserves)),
  ].join('');
  return `<section aria-labelledby="stats-title"><h2 id="stats-title">${e(t('profile.stats'))}</h2>
    <ul class="metrics">${items}</ul></section>`;
}

function achievementCard(a: Achievement): string {
  const secret = a.hidden && !a.unlocked;
  const name = secret ? t('profile.hidden') : a.name;
  const text = secret ? '' : a.description;
  const search = `${name} ${text}`.toLowerCase();
  const symbol: IconName = secret ? 'lock' : 'trophy';
  return `<li class="ach ${a.unlocked ? 'ach-done' : 'ach-todo'}" data-state="${a.unlocked ? 'done' : 'todo'}" data-text="${e(search)}">
    <span class="ach-icon" aria-hidden="true">${icon(symbol, 22)}</span>
    <div><strong>${e(name)}</strong><span>${e(text)}</span></div></li>`;
}

function renderAchievements(p: Profile): string {
  if (!p.hasAchievements) {
    return `<section aria-labelledby="ach-title"><h2 id="ach-title">${e(t('profile.achievements'))}</h2>
      <p class="notice notice-warn">${e(t('profile.noAchievements'))}</p></section>`;
  }
  const done = p.achievements.filter((a) => a.unlocked).length;
  const total = p.achievements.length;
  const percent = total === 0 ? 0 : Math.round((done / total) * 100);
  const sorted = [...p.achievements].sort((a, b) => Number(b.unlocked) - Number(a.unlocked));
  const filter = (name: string, label: string, pressed: boolean): string =>
    `<button class="seg" data-ach-filter="${name}" aria-pressed="${String(pressed)}">${e(label)}</button>`;
  return `<section aria-labelledby="ach-title" id="achievements">
    <h2 id="ach-title">${e(t('profile.achievements'))}</h2>
    <div class="ach-head card">
      ${ring(percent, `${String(percent)}%`, t('profile.achievementsOf', { done, total }))}
      <div class="ach-tools">
        <div class="segmented">${filter('all', t('profile.filterAll'), true)}${filter('done', t('profile.filterDone'), false)}${filter('todo', t('profile.filterTodo'), false)}</div>
        <input type="search" id="ach-search" placeholder="${e(t('profile.search'))}" aria-label="${e(t('profile.search'))}" />
      </div>
    </div>
    <ul class="ach-grid" id="ach-grid">${sorted.map(achievementCard).join('')}</ul>
    <p class="muted" id="ach-empty" hidden>${e(t('profile.noMatch'))}</p>
  </section>`;
}

/** Tela de perfil: nível, marcas do caçador e conquistas da Steam. */
export function renderProfile(p: Profile): string {
  return `<header class="profile-hero">
      <div class="avatar" aria-hidden="true"><span>${String(p.level)}</span></div>
      <div class="hero-text">
        <h1>${e(t('profile.title'))}</h1>
        <p class="muted">${e(t('profile.subtitle', { id: p.steamId }))}</p>
        <div class="hero-badges">
          <span class="badge badge-ok">${e(t('profile.level', { level: p.level }))}</span>
          <span class="badge badge-muted">${e(t('profile.xp', { xp: formatCount(p.xp) }))}</span>
          <span class="badge badge-warn">${e(t('profile.cash'))}: ${e(formatCount(p.cash))}</span>
        </div>
      </div>
    </header>
    ${renderStats(p)}
    ${renderAchievements(p)}`;
}

/** Liga o filtro e a busca das conquistas depois que o HTML do perfil existe. */
export function bindProfile(root: HTMLElement): void {
  const grid = root.querySelector<HTMLElement>('#ach-grid');
  const search = root.querySelector<HTMLInputElement>('#ach-search');
  const empty = root.querySelector<HTMLElement>('#ach-empty');
  if (!grid || !search || !empty) return;
  let state = 'all';
  const apply = (): void => {
    const query = search.value.trim().toLowerCase();
    let shown = 0;
    for (const card of grid.querySelectorAll<HTMLElement>('.ach')) {
      const match =
        (state === 'all' || card.dataset['state'] === state) &&
        (query === '' || (card.dataset['text'] ?? '').includes(query));
      card.hidden = !match;
      if (match) shown += 1;
    }
    empty.hidden = shown > 0;
  };
  for (const button of root.querySelectorAll<HTMLButtonElement>('[data-ach-filter]')) {
    button.addEventListener('click', () => {
      state = button.dataset['achFilter'] ?? 'all';
      for (const other of root.querySelectorAll<HTMLButtonElement>('[data-ach-filter]')) {
        other.setAttribute('aria-pressed', String(other === button));
      }
      apply();
    });
  }
  search.addEventListener('input', apply);
}

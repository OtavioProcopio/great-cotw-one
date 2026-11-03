import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import type { Mission, ReserveMissions } from '../types';
import { ring } from './profile';

export type MissionFilter = 'active' | 'all' | 'done' | 'locked';

const FILTERS: MissionFilter[] = ['active', 'all', 'done', 'locked'];

function filterLabel(filter: MissionFilter): string {
  switch (filter) {
    case 'active':
      return t('missions.filterActive');
    case 'all':
      return t('missions.filterAll');
    case 'done':
      return t('missions.filterDone');
    case 'locked':
      return t('missions.filterLocked');
  }
}

function stateLabel(state: Mission['state']): string {
  switch (state) {
    case 'done':
      return t('missions.stateDone');
    case 'active':
      return t('missions.stateActive');
    case 'locked':
      return t('missions.stateLocked');
  }
}

const STATE_ICON: Record<Mission['state'], IconName> = { done: 'check', active: 'target', locked: 'lock' };

/** Missões que o filtro deixa passar. */
export function filterMissions(missions: Mission[], filter: MissionFilter): Mission[] {
  return filter === 'all' ? missions : missions.filter((m) => m.state === filter);
}

function percent(done: number, total: number): number {
  return total === 0 ? 0 : Math.round((done / total) * 100);
}

function missionRow(mission: Mission): string {
  const doneObjectives = mission.objectives.filter((o) => o.done >= o.required).length;
  const bars = mission.objectives
    .map((o) => {
      const share = o.required === 0 ? 100 : Math.round((Math.min(o.done, o.required) / o.required) * 100);
      return `<span class="bar" title="${String(o.done)}/${String(o.required)}"><i style="width:${String(share)}%"></i></span>`;
    })
    .join('');
  const text = mission.text === '' ? t('missions.noText', { id: mission.id }) : mission.text;
  const story = mission.main ? `<span class="badge badge-ok">${e(t('missions.story'))}</span>` : '';
  const progress =
    mission.objectives.length === 0
      ? ''
      : `<div class="objectives"><span class="bars">${bars}</span><span class="muted">${String(doneObjectives)}/${String(mission.objectives.length)}</span></div>`;
  return `<li class="mission mission-${mission.state}">
    <span class="mission-icon" aria-hidden="true">${icon(STATE_ICON[mission.state], 20)}</span>
    <div class="mission-body"><p title="${e(text)}">${e(text)}</p>${progress}
      <div class="mission-meta"><span class="muted">${e(stateLabel(mission.state))}</span>${story}</div></div>
  </li>`;
}

function reserveCard(reserve: ReserveMissions, cover: string | null, selected: boolean): string {
  const image = cover
    ? `<img src="${e(cover)}" alt="" loading="lazy" />`
    : `<span class="cover-empty" aria-hidden="true">${icon('flag', 22)}</span>`;
  const share = percent(reserve.done, reserve.total);
  return `<button class="reserve-card" data-action="pick-mission-reserve" data-reserve="${String(reserve.id)}" aria-pressed="${String(selected)}">
    <span class="cover">${image}</span>
    <span class="reserve-name">${e(reserve.name)}</span>
    <span class="mini-progress" aria-hidden="true"><i style="width:${String(share)}%"></i></span>
  </button>`;
}

/** Tela de missões: uma reserva por vez, com o andamento e a lista filtrável. */
export function renderMissions(
  list: ReserveMissions[],
  covers: ReadonlyMap<number, string | null>,
  selected: number | null,
  filter: MissionFilter,
): string {
  const header = `<header class="view-header">
      <div><h1>${e(t('missions.title'))}</h1><p class="lead">${e(t('missions.subtitle'))}</p></div>
    </header>`;
  const current = list.find((r) => r.id === selected) ?? list[0];
  if (!current) return `${header}<p class="muted">${e(t('missions.none'))}</p>`;
  const total = list.reduce((sum, r) => sum + r.total, 0);
  const done = list.reduce((sum, r) => sum + r.done, 0);
  const cards = list.map((r) => reserveCard(r, covers.get(r.id) ?? null, r.id === current.id)).join('');
  const segments = FILTERS.map(
    (name) =>
      `<button class="seg" data-action="mission-filter" data-filter="${name}" aria-pressed="${String(name === filter)}">${e(filterLabel(name))}</button>`,
  ).join('');
  const shown = filterMissions(current.missions, filter);
  const rows =
    shown.length === 0
      ? `<p class="muted">${e(t('missions.empty'))}</p>`
      : `<ul class="mission-list">${shown.map(missionRow).join('')}</ul>`;
  const share = percent(current.done, current.total);
  return `${header}
    <p class="muted overall">${e(t('missions.overall', { done: formatCount(done), total: formatCount(total), reserves: list.length }))}</p>
    <section aria-labelledby="mr-title"><h2 id="mr-title">${e(t('missions.reserves'))}</h2>
      <div class="reserve-strip big" role="group" aria-label="${e(t('missions.reserves'))}">${cards}</div></section>
    <section class="card reserve-summary" aria-label="${e(current.name)}">
      ${ring(share, `${String(share)}%`, t('missions.of', { done: current.done, total: current.total }))}
      <div><h2>${e(current.name)}</h2>
        <p>${e(t('missions.main', { done: current.mainDone, total: current.mainTotal }))}</p>
        <p class="muted">${e(t('missions.active', { count: current.active }))}</p></div>
    </section>
    <div class="segmented">${segments}</div>
    ${rows}`;
}

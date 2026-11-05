import { escapeHtml as e } from '../escape';
import { formatCount, formatEpoch } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { AnimalReserve, Harvest, SpeciesEntry } from '../types';

/** Tom (0 a 359) estável de uma espécie, para colorir o ícone dela. */
export function hueOf(id: string): number {
  let sum = 0;
  for (const char of id) sum = (sum * 31 + char.charCodeAt(0)) % 360;
  return sum;
}

/** Reservas do jogo onde a espécie vive. */
export function reservesOf(list: AnimalReserve[], species: string): AnimalReserve[] {
  return list.filter((r) => r.species.some((s) => s.id === species));
}

function reserveCard(reserve: AnimalReserve, selected: boolean): string {
  const image = reserve.cover
    ? `<img src="${e(reserve.cover)}" alt="" loading="lazy" />`
    : `<span class="cover-empty" aria-hidden="true">${icon('paw', 24)}</span>`;
  const state = reserve.played ? t('animals.played') : t('animals.notPlayed');
  return `<button class="reserve-card" data-action="pick-animal-reserve" data-reserve="${String(reserve.id)}" aria-pressed="${String(selected)}" title="${e(state)}">
    <span class="cover">${image}${reserve.played ? '<span class="played-dot" aria-hidden="true"></span>' : ''}</span>
    <span class="reserve-name">${e(reserve.name)}</span>
    <span class="reserve-sub">${e(t('animals.count', { count: reserve.species.length }))}</span>
  </button>`;
}

function speciesCard(entry: SpeciesEntry, selected: boolean): string {
  const living = entry.living;
  const detail = living
    ? t('animals.living', { animals: living.animals, groups: living.groups })
    : t('animals.noSave');
  const star =
    living && living.greatOnes > 0
      ? `<span class="badge badge-warn">${icon('star', 12)} ${e(t('animals.greatOnes', { count: living.greatOnes }))}</span>`
      : '';
  const diamonds =
    living && living.diamonds > 0
      ? `<span class="badge badge-ok">${icon('star', 12)} ${e(t('animals.diamonds', { count: living.diamonds }))}</span>`
      : '';
  return `<li><button class="species-card" data-action="pick-species" data-species="${e(entry.id)}" data-name="${e(entry.name.toLowerCase())}" aria-pressed="${String(selected)}" style="--hue:${String(hueOf(entry.id))}">
    <span class="species-icon" aria-hidden="true">${icon('paw', 22)}</span>
    <span class="species-text"><strong>${e(entry.name)}</strong><span class="muted">${e(detail)}</span>${diamonds}${star}</span>
  </button></li>`;
}

function medalText(medal: number): string {
  return medal <= 0 ? t('animals.noMedal') : '★'.repeat(Math.min(5, medal));
}

function harvestRow(harvest: Harvest): string {
  return `<li><strong>${e(formatCount(Math.round(harvest.score)))}</strong>
    <span class="medal" title="${e(t('animals.medal', { count: harvest.medal }))}">${e(medalText(harvest.medal))}</span>
    <span class="muted">${e(harvest.at === 0 ? '' : formatEpoch(harvest.at))}</span></li>`;
}

function metric(label: string, value: string): string {
  return `<li class="metric"><div><strong>${e(value)}</strong><span>${e(label)}</span></div></li>`;
}

function detailCard(list: AnimalReserve[], reserve: AnimalReserve, entry: SpeciesEntry): string {
  const where = reservesOf(list, entry.id)
    .map(
      (r) =>
        `<button class="chip-link" data-action="open-animal" data-reserve="${String(r.id)}" data-species="${e(entry.id)}" aria-pressed="${String(r.id === reserve.id)}">${e(r.name)}</button>`,
    )
    .join('');
  const living = entry.living;
  const here = living
    ? `<ul class="metrics small">
        ${metric(t('animals.living', { animals: living.animals, groups: living.groups }), formatCount(living.animals))}
        ${metric(t('animals.sexes', { males: living.males, females: living.females }), `♂ ${String(living.males)} · ♀ ${String(living.females)}`)}
        ${metric(t('animals.bestScore'), formatCount(Math.round(living.bestScore)))}
        ${metric(t('animals.heaviest'), t('animals.kg', { value: formatCount(Math.round(living.heaviest)) }))}
        ${metric('Great One', formatCount(living.greatOnes))}
        ${metric(t('animals.diamondsLabel'), formatCount(living.diamonds))}
        ${living.diamondMin === null ? '' : metric(t('animals.diamondMin'), formatCount(Math.round(living.diamondMin)))}
      </ul>`
    : `<p class="muted">${e(t('animals.noSave'))}</p>`;
  const harvests =
    entry.harvests.length === 0
      ? `<p class="muted">${e(t('animals.noHarvests'))}</p>`
      : `<ol class="harvests">${entry.harvests.map(harvestRow).join('')}</ol>`;
  return `<section class="card species-detail" style="--hue:${String(hueOf(entry.id))}" aria-label="${e(entry.name)}">
    <div class="detail-head">
      <span class="species-icon big" aria-hidden="true">${icon('paw', 30)}</span>
      <div><h2>${e(entry.name)}</h2>
        <p class="muted">${e(reserve.name)} · ${e(t('animals.areas', { count: formatCount(entry.areas) }))}</p></div>
      <button data-action="pick-species" data-species="" aria-label="${e(t('animals.close'))}" title="${e(t('animals.close'))}">${icon('close', 16)}</button>
    </div>
    <h3>${e(t('animals.where'))}</h3><div class="chip-links">${where}</div>
    <h3>${e(t('animals.here'))}</h3>${here}
    <h3>${e(t('animals.harvests'))}</h3>${harvests}
  </section>`;
}

/** Wiki dos animais: escolhe a reserva, vê as espécies e abre a ficha de uma delas. */
export function renderAnimals(list: AnimalReserve[], reserveId: number | null, speciesId: string | null): string {
  const header = `<header class="view-header">
      <div><h1>${e(t('animals.title'))}</h1><p class="lead">${e(t('animals.subtitle'))}</p></div>
    </header>`;
  const reserve = list.find((r) => r.id === reserveId) ?? list[0];
  if (!reserve) return `${header}<p class="muted">${e(t('animals.none'))}</p>`;
  const entry = reserve.species.find((s) => s.id === speciesId) ?? null;
  const cards = list.map((r) => reserveCard(r, r.id === reserve.id)).join('');
  const grid = reserve.species.map((s) => speciesCard(s, s.id === entry?.id)).join('');
  return `${header}
    <section aria-labelledby="ar-title"><h2 id="ar-title">${e(t('animals.reserves'))}</h2>
      <div class="reserve-strip big" role="group" aria-label="${e(t('animals.reserves'))}">${cards}</div></section>
    ${entry ? detailCard(list, reserve, entry) : ''}
    <section aria-label="${e(reserve.name)}">
      <div class="species-tools"><h2>${e(reserve.name)}</h2>
        <input type="search" id="animal-search" placeholder="${e(t('animals.search'))}" aria-label="${e(t('animals.search'))}" /></div>
      <ul class="species-grid" id="species-grid">${grid}</ul>
      <p class="muted" id="animal-empty" hidden>${e(t('animals.empty'))}</p>
    </section>`;
}

/** Liga a busca por nome depois que o HTML da wiki existe. */
export function bindAnimals(root: HTMLElement): void {
  const grid = root.querySelector<HTMLElement>('#species-grid');
  const search = root.querySelector<HTMLInputElement>('#animal-search');
  const empty = root.querySelector<HTMLElement>('#animal-empty');
  if (!grid || !search || !empty) return;
  search.addEventListener('input', () => {
    const query = search.value.trim().toLowerCase();
    let shown = 0;
    for (const item of grid.querySelectorAll<HTMLElement>('li')) {
      const name = item.querySelector<HTMLElement>('[data-name]')?.dataset['name'] ?? '';
      const match = query === '' || name.includes(query);
      item.hidden = !match;
      if (match) shown += 1;
    }
    empty.hidden = shown > 0;
  });
}

import { escapeHtml as e } from '../escape';
import { type EquipTab, KINDS, MAX_CASH, type OwnFilter, RANK_KEYS, type RankKey, collection, itemsById, maxAmount, rankValue, rankWeapons, visibleItems } from '../equipmentlogic';
import { formatCount, formatEpoch } from '../format';
import { t } from '../i18n';
import { icon, type IconName } from '../icons';
import type { EquipmentItem, EquipmentKind, Inventory, Loadout } from '../types';

/** O que o usuário escolheu na página. */
export interface EquipState {
  tab: EquipTab;
  kind: EquipmentKind;
  own: OwnFilter;
  rank: RankKey;
  /** Arma aberta na ficha. */
  open: string | null;
}

export function initialEquipState(): EquipState {
  return { tab: 'arsenal', kind: 'weapon', own: 'all', rank: 'kills', open: null };
}

const KIND_ICON: Record<EquipmentKind, IconName> = {
  weapon: 'target',
  ammo: 'list',
  sight: 'fit',
  optic: 'fit',
  lure_caller: 'bird',
  deployable: 'flag',
  consumable: 'drop',
  vehicle: 'map',
};

/** Ícone do item: a imagem do jogo quando já foi carregada, senão o símbolo da categoria. */
function thumb(item: EquipmentItem, icons: Record<string, string>, size = 56): string {
  const url = icons[item.id];
  const fallback = `<span class="eq-fallback" aria-hidden="true">${icon(KIND_ICON[item.kind], 26)}</span>`;
  return `<span class="eq-thumb" style="--size:${String(size)}px">${
    url ? `<img src="${e(url)}" alt="" />` : `${fallback}<img data-equip-icon="${e(item.id)}" alt="" hidden />`
  }</span>`;
}

function tabs(state: EquipState): string {
  const tab = (id: EquipTab): string =>
    `<button class="seg" data-action="equip-tab" data-tab="${id}" aria-pressed="${String(state.tab === id)}">${e(t(`equip.tab.${id}`))}</button>`;
  return `<div class="segmented" role="group">${tab('arsenal')}${tab('ranking')}${tab('loadouts')}</div>`;
}

/** Ações do Premium de um item; quem não tem Premium vê o botão com cadeado que leva à tela do Premium. */
function actions(item: EquipmentItem, premium: boolean): string {
  const stackable = maxAmount(item.kind) > 1;
  const lock = (label: string): string =>
    `<button class="eq-act locked" data-action="show-premium" title="${e(t('equip.premiumOnly'))}">${icon('lock', 14)} ${e(label)}</button>`;
  const act = (action: string, label: string): string =>
    `<button class="eq-act" data-action="${action}" data-id="${e(item.id)}">${e(label)}</button>`;
  if (!item.owned) {
    const give = premium ? act('equip-give', t('equip.give')) : lock(t('equip.give'));
    const shop = item.unlocked ? '' : premium ? act('equip-unlock', t('equip.unlockShop')) : lock(t('equip.unlockShop'));
    return `<div class="eq-actions">${give}${shop}</div>`;
  }
  if (!stackable) return '';
  if (!premium) return `<div class="eq-actions">${lock(t('equip.setAmount'))}</div>`;
  return `<div class="eq-actions"><label>${e(t('equip.amountLabel'))}
    <input type="number" class="eq-amount" data-eq-amount="${e(item.id)}" min="1" max="${String(maxAmount(item.kind))}" value="${String(item.amount)}" /></label>
    ${act('equip-set-amount', t('equip.setAmount'))}</div>`;
}

function cashForm(inv: Inventory, premium: boolean): string {
  if (!premium) {
    return `<div class="eq-cash"><button class="eq-act locked" data-action="show-premium" title="${e(t('equip.premiumOnly'))}">${icon('lock', 14)} ${e(t('equip.setCash'))}</button></div>`;
  }
  return `<div class="eq-cash"><label>${e(t('equip.cashLabel'))}
    <input type="number" id="eq-cash" min="0" max="${String(MAX_CASH)}" value="${String(inv.cash)}" /></label>
    <button class="eq-act" data-action="equip-set-cash">${e(t('equip.setCash'))}</button>
    <small class="muted">${e(t('equip.cashLimit', { max: formatCount(MAX_CASH) }))}</small></div>`;
}

function header(inv: Inventory): string {
  const levels = (['rifle', 'handgun', 'shotgun', 'archery'] as const)
    .map((name, i) => `<li class="metric"><span class="metric-icon" aria-hidden="true">${icon('target', 20)}</span>
      <div><strong>${e(formatCount(inv.weaponLevels[i] ?? 0))}</strong><span>${e(t(`equip.level.${name}`))}</span></div></li>`)
    .join('');
  return `<ul class="metrics small">
    <li class="metric"><span class="metric-icon" aria-hidden="true">${icon('star', 20)}</span>
      <div><strong>${e(formatCount(inv.cash))}</strong><span>${e(t('equip.cash'))}</span></div></li>${levels}</ul>`;
}

function itemBadges(item: EquipmentItem): string {
  const parts: string[] = [];
  if (item.owned) {
    const text = item.kind === 'ammo' || item.kind === 'consumable' || item.kind === 'lure_caller' || item.kind === 'deployable'
      ? t('equip.amount', { count: formatCount(item.amount) })
      : t('equip.owned');
    parts.push(`<span class="badge badge-ok">${e(text)}</span>`);
  } else if (item.unlocked) {
    parts.push(`<span class="badge badge-warn">${e(t('equip.unlocked'))}</span>`);
  } else {
    parts.push(`<span class="badge badge-muted">${e(t('equip.notOwned'))}</span>`);
  }
  if (item.inBackpack) parts.push(`<span class="badge badge-muted">${e(t('equip.backpack'))}</span>`);
  return parts.join(' ');
}

function itemMeta(item: EquipmentItem): string {
  const meta: string[] = [];
  if (item.class) meta.push(t(`equip.class.${item.class}`));
  if (item.price > 0) meta.push(t('equip.price', { value: formatCount(item.price) }));
  if (item.weight !== null) meta.push(t('equip.weight', { value: item.weight.toLocaleString() }));
  if (item.boxSize !== null) meta.push(t('equip.box', { count: item.boxSize }));
  return meta.join(' · ');
}

function itemCard(item: EquipmentItem, icons: Record<string, string>, premium: boolean): string {
  const body = `${thumb(item, icons)}
    <span class="eq-text"><strong>${e(item.name)}</strong><small>${e(itemMeta(item))}</small><span>${itemBadges(item)}</span></span>`;
  const search = e(`${item.name} ${item.id}`.toLowerCase());
  return item.kind === 'weapon'
    ? `<li data-text="${search}"><button class="eq-card" data-action="equip-open" data-id="${e(item.id)}">${body}</button>${actions(item, premium)}</li>`
    : `<li data-text="${search}"><div class="eq-card">${body}</div>${actions(item, premium)}</li>`;
}

function arsenal(inv: Inventory, state: EquipState, icons: Record<string, string>, premium: boolean): string {
  const progress = collection(inv.items);
  const chips = KINDS.map((kind) => {
    const p = progress[kind];
    return `<button class="seg" data-action="equip-kind" data-kind="${kind}" aria-pressed="${String(state.kind === kind)}">${e(t(`equip.kind.${kind}`))} <small>${e(t('equip.count', { owned: p.owned, total: p.total }))}</small></button>`;
  }).join('');
  const own = (id: OwnFilter): string =>
    `<button class="seg" data-action="equip-own" data-own="${id}" aria-pressed="${String(state.own === id)}">${e(t(`equip.filter.${id}`))}</button>`;
  const list = visibleItems(inv.items, state.kind, state.own);
  const body =
    list.length === 0
      ? `<p class="muted">${e(t('equip.empty'))}</p>`
      : `<ul class="eq-grid" id="eq-grid">${list.map((item) => itemCard(item, icons, premium)).join('')}</ul>`;
  return `<div class="eq-kinds segmented">${chips}</div>
    <div class="eq-tools"><div class="segmented">${own('all')}${own('owned')}${own('missing')}</div>
      <input type="search" id="eq-search" placeholder="${e(t('equip.search'))}" aria-label="${e(t('equip.search'))}" /></div>${body}`;
}

function statLine(label: string, value: string): string {
  return `<li class="metric"><div><strong>${e(value)}</strong><span>${e(label)}</span></div></li>`;
}

function related(label: string, ids: string[], inv: Inventory, icons: Record<string, string>): string {
  const found = itemsById(inv.items, ids);
  if (found.length === 0) return '';
  const rows = found.map((item) => `<li class="eq-mini">${thumb(item, icons, 36)}<span><strong>${e(item.name)}</strong> ${itemBadges(item)}</span></li>`).join('');
  return `<section><h3>${e(label)}</h3><ul class="eq-list">${rows}</ul></section>`;
}

function weaponSheet(inv: Inventory, id: string, icons: Record<string, string>): string {
  const item = inv.items.find((i) => i.id === id);
  if (!item) return '';
  const stats = item.stats;
  const lines =
    stats && stats.shotsFired > 0
      ? [
          statLine(t('equip.stat.kills'), formatCount(stats.kills)),
          statLine(t('equip.stat.oneShot'), formatCount(stats.oneShotKills)),
          statLine(t('equip.stat.shots'), formatCount(stats.shotsFired)),
          statLine(t('equip.stat.accuracy'), stats.accuracy === null ? '-' : `${String(Math.round(stats.accuracy))}%`),
          statLine(t('equip.stat.record'), stats.recordSpecies ? t('equip.recordOn', { score: formatCount(Math.round(stats.recordScore)), species: stats.recordSpecies }) : formatCount(Math.round(stats.recordScore))),
          statLine(t('equip.stat.distance'), stats.furthestShot === null ? '-' : t('profile.meters', { value: formatCount(Math.round(stats.furthestShot)) })),
          ...(stats.purchasedAt === null ? [] : [statLine(t('equip.stat.bought'), formatEpoch(stats.purchasedAt))]),
        ].join('')
      : `<li class="muted">${e(t('equip.noStats'))}</li>`;
  return `<button data-action="equip-close">${icon('back', 18)} ${e(t('equip.back'))}</button>
    <div class="card eq-sheet-head">${thumb(item, icons, 96)}
      <div><h2>${e(item.name)}</h2><p class="muted">${e(itemMeta(item))}</p><p>${itemBadges(item)}</p></div></div>
    <ul class="metrics small">${lines}</ul>
    ${related(t('equip.ammoFits'), item.ammo, inv, icons)}
    ${related(t('equip.sightsFit'), item.sights, inv, icons)}`;
}

function rankingValue(item: EquipmentItem, key: RankKey): string {
  const value = rankValue(item, key);
  if (value === null) return '-';
  if (key === 'accuracy') return `${String(Math.round(value))}%`;
  if (key === 'distance') return t('profile.meters', { value: formatCount(Math.round(value)) });
  return formatCount(Math.round(value));
}

function ranking(inv: Inventory, state: EquipState, icons: Record<string, string>): string {
  const sorts = RANK_KEYS.map(
    (key) => `<button class="seg" data-action="equip-rank" data-rank="${key}" aria-pressed="${String(state.rank === key)}">${e(t(`equip.rank.${key}`))}</button>`,
  ).join('');
  const list = rankWeapons(inv.items, state.rank);
  const rows = list
    .map(
      (item, i) => `<li><button class="eq-card eq-row" data-action="equip-open" data-id="${e(item.id)}">
        <span class="eq-pos">${String(i + 1)}</span>${thumb(item, icons, 44)}
        <span class="eq-text"><strong>${e(item.name)}</strong><small>${e(itemMeta(item))}</small></span>
        <strong class="eq-value">${e(rankingValue(item, state.rank))}</strong></button></li>`,
    )
    .join('');
  return `<div class="segmented">${sorts}</div>${list.length === 0 ? `<p class="muted">${e(t('equip.rankEmpty'))}</p>` : `<ol class="eq-grid eq-rank">${rows}</ol>`}`;
}

function loadoutCard(loadout: Loadout, inv: Inventory, icons: Record<string, string>): string {
  const state = loadout.active
    ? `<span class="badge badge-ok">${e(t('equip.loadout.active'))}</span>`
    : loadout.unlocked
      ? ''
      : `<span class="badge badge-muted">${e(t('equip.loadout.locked'))}</span>`;
  const items = itemsById(inv.items, loadout.items);
  const missing = itemsById(inv.items, loadout.missing);
  const chips = items.map((item) => `<li class="eq-chip">${thumb(item, icons, 28)}<span>${e(item.name)}</span></li>`).join('');
  const lost = missing.length
    ? `<p class="notice notice-warn">${e(t('equip.loadout.missing'))}: ${e(missing.map((m) => m.name).join(', '))}</p>`
    : '';
  const name = loadout.name || t('equip.loadout.unnamed', { number: loadout.index + 1 });
  return `<li class="card"><h3>${e(name)} ${state}</h3>${
    items.length === 0 ? `<p class="muted">${e(t('equip.loadout.empty'))}</p>` : `<ul class="eq-chips">${chips}</ul>`
  }${lost}</li>`;
}

function loadouts(inv: Inventory, icons: Record<string, string>): string {
  if (inv.loadouts.length === 0) return `<p class="muted">${e(t('equip.loadout.none'))}</p>`;
  return `<ul class="eq-loadouts">${inv.loadouts.map((l) => loadoutCard(l, inv, icons)).join('')}</ul>`;
}

/** Página de equipamentos: arsenal, ranking das armas e conjuntos salvos. */
export function renderEquipment(inv: Inventory, state: EquipState, icons: Record<string, string>, premium = false): string {
  const content =
    state.open !== null
      ? weaponSheet(inv, state.open, icons)
      : state.tab === 'arsenal'
        ? arsenal(inv, state, icons, premium)
        : state.tab === 'ranking'
          ? ranking(inv, state, icons)
          : loadouts(inv, icons);
  return `<header><h1>${e(t('equip.title'))}</h1><p class="muted">${e(t('equip.subtitle'))}</p></header>
    ${state.open === null ? `${header(inv)}${cashForm(inv, premium)}${tabs(state)}` : ''}${content}`;
}

/** Filtro por texto no arsenal, feito sem refazer a página (o campo mantém o foco). */
export function bindEquipment(root: HTMLElement): void {
  const input = root.querySelector<HTMLInputElement>('#eq-search');
  const grid = root.querySelector<HTMLElement>('#eq-grid');
  if (!input || !grid) return;
  input.addEventListener('input', () => {
    const needle = input.value.trim().toLowerCase();
    for (const card of grid.querySelectorAll<HTMLElement>('li[data-text]')) {
      card.hidden = needle !== '' && !(card.dataset['text'] ?? '').includes(needle);
    }
  });
}

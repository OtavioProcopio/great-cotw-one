import { escapeHtml as e } from '../escape';
import { formatCount, formatDecimal } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import { thumb } from './thumb';
import type { AdvicePick, AnimalReserve, EquipmentItem, GearAdvice, Inventory, Kit, KitLure, KitWeapon, UnlockTrack } from '../types';

/** O que o usuário escolheu no montador. */
export interface AdvisorState {
  reserve: number | null;
  species: string | null;
  /** `animal`: sugestão para um animal; `reserve`: kit para a reserva inteira. */
  mode?: 'animal' | 'reserve';
}

export function initialAdvisorState(): AdvisorState {
  return { reserve: null, species: null, mode: 'animal' };
}

function chip(action: string, data: string, label: string, pressed: boolean): string {
  return `<button class="seg" data-action="${action}" ${data} aria-pressed="${String(pressed)}">${e(label)}</button>`;
}

function number(value: number | null | undefined, digits = 0): string {
  return value === null || value === undefined ? '-' : formatDecimal(value, digits);
}

function metrics(weapon: EquipmentItem | undefined, ammo: EquipmentItem | undefined, pick: AdvicePick): string {
  const parts = [
    t('adv.classes', { min: pick.classRange[0], max: pick.classRange[1] }),
    t('adv.accuracy', { value: number(weapon?.attributes.accuracy ?? null) }),
    t('adv.recoil', { value: number(weapon?.attributes.recoil ?? null) }),
    t('adv.range', { value: number(ammo?.attributes.range ?? null) }),
    t('adv.penetration', { value: number(ammo?.attributes.penetration ?? null) }),
  ];
  return parts.map((p) => `<li>${e(p)}</li>`).join('');
}

/** Peça do conjunto: ícone grande com o nome e o papel dela (arma, munição ou mira). */
function part(role: string, item: EquipmentItem | undefined, id: string, icons: Record<string, string>, have: boolean | null): string {
  const label = item?.name ?? id.replace('equipment_', '').replaceAll('_', ' ');
  const state = have === null ? '' : have ? `<span class="badge badge-ok">${e(t('equip.owned'))}</span>` : `<span class="badge badge-muted">${e(t('equip.notOwned'))}</span>`;
  const picture = item ? thumb(item, icons, 96) : `<span class="eq-thumb" style="--size:96px"></span>`;
  return `<li class="eq-part">${picture}<small>${e(role)}</small><strong>${e(label)}</strong>${state}</li>`;
}

function trackName(track: UnlockTrack): string {
  return t(`adv.track.${track}`);
}

function pickCard(pick: AdvicePick, items: Map<string, EquipmentItem>, premium: boolean, icons: Record<string, string>): string {
  const weapon = items.get(pick.weapon);
  const ammo = items.get(pick.ammo);
  const sightItem = pick.sight ? items.get(pick.sight) : undefined;
  const ready = pick.hasWeapon && pick.hasAmmo;
  const badges: string[] = [];
  if (ready) badges.push(`<span class="badge badge-ok">${e(t('adv.haveAll'))}</span>`);
  else {
    badges.push(`<span class="badge badge-warn">${e(t('adv.cost', { value: formatCount(pick.cost) }))}</span>`);
    if (!pick.available && pick.unlock) {
      badges.push(`<span class="badge badge-muted">${icon('lock', 12)} ${e(t('adv.unlockAt', { level: pick.unlock.level, track: trackName(pick.unlock.track) }))}</span>`);
    }
  }
  const action = ready
    ? ''
    : premium
      ? `<button class="eq-act" data-action="adv-give" data-weapon="${e(pick.weapon)}" data-ammo="${e(pick.ammo)}">${e(t('adv.giveSet'))}</button>`
      : `<button class="eq-act locked" data-action="show-premium" title="${e(t('equip.premiumOnly'))}">${icon('lock', 14)} ${e(t('adv.giveSet'))}</button>`;
  const sight = pick.sight ? part(t('adv.partSight'), sightItem, pick.sight, icons, true) : '';
  return `<li class="card eq-pick">
    <div class="eq-pick-head"><span class="badge badge-muted">${e(t('adv.score', { value: Math.round(pick.score) }))}</span>${badges.join(' ')}</div>
    <ul class="eq-parts">${part(t('adv.partWeapon'), weapon, pick.weapon, icons, pick.hasWeapon)}${part(t('adv.partAmmo'), ammo, pick.ammo, icons, pick.hasAmmo)}${sight}</ul>
    <ul class="eq-stats">${metrics(weapon, ammo, pick)}</ul>${action}</li>`;
}

function section(title: string, picks: AdvicePick[], empty: string, items: Map<string, EquipmentItem>, premium: boolean, icons: Record<string, string>): string {
  const body = picks.length === 0 ? `<p class="muted">${e(empty)}</p>` : `<ul class="eq-loadouts eq-pick-list">${picks.map((p) => pickCard(p, items, premium, icons)).join('')}</ul>`;
  return `<section><h2>${e(title)}</h2>${body}</section>`;
}

function lureLine(item: EquipmentItem | undefined, lure: KitLure, icons: Record<string, string>): string {
  const attrs = item?.attributes;
  const facts = [
    attrs?.lureRange ? t('kit.lureRange', { value: number(attrs.lureRange) }) : '',
    attrs?.lureChance ? t('kit.lureChance', { value: number(attrs.lureChance) }) : '',
    lure.owned ? '' : t('adv.cost', { value: formatCount(lure.price) }),
  ].filter((x) => x !== '');
  const picture = item ? thumb(item, icons, 72) : '';
  return `<li class="eq-part">${picture}<strong>${e(item?.name ?? lure.id)}</strong>
    <small>${e(t('kit.attracts', { names: lure.covers.join(', ') }))}</small>${facts.length ? `<small>${e(facts.join(' · '))}</small>` : ''}</li>`;
}

function kitWeapon(entry: KitWeapon, items: Map<string, EquipmentItem>, premium: boolean, icons: Record<string, string>): string {
  return pickCard(entry.pick, items, premium, icons).replace(
    '<li class="card eq-pick">',
    `<li class="card eq-pick"><p class="eq-covers">${e(t('kit.covers', { names: entry.covers.join(', ') }))}</p>`,
  );
}

/** Kit para a reserva inteira: armas que cobrem todos os animais, iscas, extras e o peso total. */
function renderKit(inv: Inventory, kit: Kit, premium: boolean, icons: Record<string, string>, limit: number | null): string {
  const items = new Map(inv.items.map((i) => [i.id, i]));
  const list = (picks: KitWeapon[]): string => `<ul class="eq-loadouts eq-pick-list">${picks.map((w) => kitWeapon(w, items, premium, icons)).join('')}</ul>`;
  const extras = kit.extras
    .map((id) => items.get(id))
    .filter((i): i is EquipmentItem => i !== undefined)
    .map((i) => `<li class="eq-part">${thumb(i, icons, 72)}<strong>${e(i.name)}</strong><small>${e(t('equip.weight', { value: number(i.weight, 2) }))}</small></li>`)
    .join('');
  const lures = kit.lures.map((l) => lureLine(items.get(l.id), l, icons)).join('');
  const buyLures = kit.luresToBuy.map((l) => lureLine(items.get(l.id), l, icons)).join('');
  const weight = `<div class="card eq-weight" data-weight="${String(kit.weight)}">
    <div><strong>${e(t('kit.weight', { value: number(kit.weight, 2) }))}</strong>
      <p class="muted" id="kit-limit-note">${e(limitNote(kit.weight, limit))}</p></div>
    <label>${e(t('kit.limitLabel'))}<input type="number" id="kit-limit" min="1" step="0.5" value="${limit === null ? '' : String(limit)}" placeholder="30" /></label></div>`;
  return `<div class="card"><strong>${e(t('kit.species', { count: kit.species.length }))}</strong>
      <p class="muted">${e(kit.species.join(', '))}</p><p class="muted">${e(t('kit.how'))}</p></div>${weight}
    <section><h2>${e(t('kit.weapons'))}</h2>${kit.weapons.length ? list(kit.weapons) : `<p class="muted">${e(t('kit.noWeapons'))}</p>`}</section>
    ${kit.toBuy.length ? `<section><h2>${e(t('kit.buyWeapons'))}</h2>${list(kit.toBuy)}</section>` : ''}
    ${kit.uncovered.length ? `<p class="notice notice-warn">${e(t('kit.uncovered', { names: kit.uncovered.join(', ') }))}</p>` : ''}
    <section><h2>${e(t('kit.lures'))}</h2>${lures ? `<ul class="eq-parts">${lures}</ul>` : `<p class="muted">${e(t('kit.noLures'))}</p>`}
      ${buyLures ? `<h3>${e(t('kit.luresBuy'))}</h3><ul class="eq-parts">${buyLures}</ul>` : ''}</section>
    ${extras ? `<section><h2>${e(t('kit.extras'))}</h2><ul class="eq-parts">${extras}</ul></section>` : ''}`;
}

/** Frase que compara o peso do kit com o limite que o jogador digitou. */
export function limitNote(weight: number, limit: number | null): string {
  if (limit === null || !(limit > 0)) return t('kit.limitHint');
  const diff = Math.round((limit - weight) * 100) / 100;
  return diff >= 0 ? t('kit.limitOk', { value: number(diff, 2) }) : t('kit.limitOver', { value: number(-diff, 2) });
}

/** Montador: escolhe a reserva e a espécie e mostra o equipamento sugerido. */
export function renderAdvisor(
  inv: Inventory,
  animals: AnimalReserve[],
  state: AdvisorState,
  advice: GearAdvice | null,
  premium: boolean,
  icons: Record<string, string> = {},
  kit: Kit | null = null,
  limit: number | null = null,
): string {
  const played = animals.filter((r) => r.species.length > 0);
  const reserves = played.map((r) => chip('adv-reserve', `data-reserve="${String(r.id)}"`, r.name, state.reserve === r.id)).join('');
  const chosen = played.find((r) => r.id === state.reserve);
  const species = chosen
    ? chosen.species.map((s) => chip('adv-species', `data-species="${e(s.id)}"`, s.name, state.species === s.id)).join('')
    : '';
  const mode = state.mode ?? 'animal';
  const modeButton = (id: 'animal' | 'reserve'): string =>
    `<button class="seg" data-action="adv-mode" data-mode="${id}" aria-pressed="${String(mode === id)}">${e(t(`adv.mode.${id}`))}</button>`;
  const switcher = `<div class="segmented">${modeButton('animal')}${modeButton('reserve')}</div>`;
  if (mode === 'reserve') {
    const body = !state.reserve ? '' : kit ? renderKit(inv, kit, premium, icons, limit) : `<p class="notice notice-warn">${e(t('kit.none'))}</p>`;
    return `${switcher}<p class="muted">${e(t('kit.intro'))}</p>
      <h3>${e(t('adv.pickReserve'))}</h3><div class="eq-kinds segmented">${reserves}</div>${body}`;
  }
  const head = `${switcher}<p class="muted">${e(t('adv.intro'))}</p>
    <h3>${e(t('adv.pickReserve'))}</h3><div class="eq-kinds segmented">${reserves}</div>
    ${chosen ? `<h3>${e(t('adv.pickSpecies'))}</h3><div class="eq-kinds segmented">${species}</div>` : ''}`;
  if (!state.species) return head;
  if (!advice) return `${head}<p class="notice notice-warn">${e(t('adv.noClass'))}</p>`;
  const items = new Map(inv.items.map((i) => [i.id, i]));
  return `${head}<div class="card"><strong>${e(advice.name)}</strong> · ${e(t('adv.huntClass', { value: advice.huntClass }))}
      <p class="muted">${e(t('adv.how'))}</p></div>
    ${section(t('adv.ready'), advice.ready, t('adv.noneReady'), items, premium, icons)}
    ${section(t('adv.buy'), advice.toBuy, t('adv.noneBuy'), items, premium, icons)}`;
}

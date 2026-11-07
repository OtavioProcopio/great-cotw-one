import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { AdvicePick, AnimalReserve, EquipmentItem, GearAdvice, Inventory, UnlockTrack } from '../types';

/** O que o usuário escolheu no montador. */
export interface AdvisorState {
  reserve: number | null;
  species: string | null;
}

export function initialAdvisorState(): AdvisorState {
  return { reserve: null, species: null };
}

function chip(action: string, data: string, label: string, pressed: boolean): string {
  return `<button class="seg" data-action="${action}" ${data} aria-pressed="${String(pressed)}">${e(label)}</button>`;
}

function itemName(items: Map<string, EquipmentItem>, id: string): string {
  return items.get(id)?.name ?? id.replace('equipment_', '').replaceAll('_', ' ');
}

function number(value: number | null, digits = 0): string {
  return value === null ? '-' : value.toLocaleString(undefined, { maximumFractionDigits: digits });
}

function metrics(weapon: EquipmentItem | undefined, ammo: EquipmentItem | undefined, pick: AdvicePick): string {
  const parts = [
    t('adv.classes', { min: pick.classRange[0], max: pick.classRange[1] }),
    t('adv.accuracy', { value: number(weapon?.attributes.accuracy ?? null) }),
    t('adv.recoil', { value: number(weapon?.attributes.recoil ?? null) }),
    t('adv.range', { value: number(ammo?.attributes.range ?? null) }),
    t('adv.penetration', { value: number(ammo?.attributes.penetration ?? null) }),
  ];
  return parts.join(' · ');
}

function trackName(track: UnlockTrack): string {
  return t(`adv.track.${track}`);
}

function pickCard(pick: AdvicePick, items: Map<string, EquipmentItem>, premium: boolean): string {
  const weapon = items.get(pick.weapon);
  const ammo = items.get(pick.ammo);
  const ready = pick.hasWeapon && pick.hasAmmo;
  const badges: string[] = [];
  if (ready) badges.push(`<span class="badge badge-ok">${e(t('adv.haveAll'))}</span>`);
  else {
    badges.push(`<span class="badge badge-warn">${e(t('adv.cost', { value: formatCount(pick.cost) }))}</span>`);
    if (!pick.available && pick.unlock) {
      badges.push(`<span class="badge badge-muted">${icon('lock', 12)} ${e(t('adv.unlockAt', { level: pick.unlock.level, track: trackName(pick.unlock.track) }))}</span>`);
    }
  }
  const sight = pick.sight ? `<small>${e(t('adv.sight', { name: itemName(items, pick.sight) }))}</small>` : '';
  const action = ready
    ? ''
    : premium
      ? `<button class="eq-act" data-action="adv-give" data-weapon="${e(pick.weapon)}" data-ammo="${e(pick.ammo)}">${e(t('adv.giveSet'))}</button>`
      : `<button class="eq-act locked" data-action="show-premium" title="${e(t('equip.premiumOnly'))}">${icon('lock', 14)} ${e(t('adv.giveSet'))}</button>`;
  return `<li class="card eq-pick">
    <h3>${e(itemName(items, pick.weapon))} <small>+ ${e(itemName(items, pick.ammo))}</small>
      <span class="badge badge-muted">${e(t('adv.score', { value: Math.round(pick.score) }))}</span></h3>
    <p class="muted">${e(metrics(weapon, ammo, pick))}</p>${sight}
    <p>${badges.join(' ')}</p>${action}</li>`;
}

function section(title: string, picks: AdvicePick[], empty: string, items: Map<string, EquipmentItem>, premium: boolean): string {
  const body = picks.length === 0 ? `<p class="muted">${e(empty)}</p>` : `<ul class="eq-loadouts">${picks.map((p) => pickCard(p, items, premium)).join('')}</ul>`;
  return `<section><h2>${e(title)}</h2>${body}</section>`;
}

/** Montador: escolhe a reserva e a espécie e mostra o equipamento sugerido. */
export function renderAdvisor(
  inv: Inventory,
  animals: AnimalReserve[],
  state: AdvisorState,
  advice: GearAdvice | null,
  premium: boolean,
): string {
  const played = animals.filter((r) => r.species.length > 0);
  const reserves = played.map((r) => chip('adv-reserve', `data-reserve="${String(r.id)}"`, r.name, state.reserve === r.id)).join('');
  const chosen = played.find((r) => r.id === state.reserve);
  const species = chosen
    ? chosen.species.map((s) => chip('adv-species', `data-species="${e(s.id)}"`, s.name, state.species === s.id)).join('')
    : '';
  const head = `<p class="muted">${e(t('adv.intro'))}</p>
    <h3>${e(t('adv.pickReserve'))}</h3><div class="eq-kinds segmented">${reserves}</div>
    ${chosen ? `<h3>${e(t('adv.pickSpecies'))}</h3><div class="eq-kinds segmented">${species}</div>` : ''}`;
  if (!state.species) return head;
  if (!advice) return `${head}<p class="notice notice-warn">${e(t('adv.noClass'))}</p>`;
  const items = new Map(inv.items.map((i) => [i.id, i]));
  return `${head}<div class="card"><strong>${e(advice.name)}</strong> · ${e(t('adv.huntClass', { value: advice.huntClass }))}
      <p class="muted">${e(t('adv.how'))}</p></div>
    ${section(t('adv.ready'), advice.ready, t('adv.noneReady'), items, premium)}
    ${section(t('adv.buy'), advice.toBuy, t('adv.noneBuy'), items, premium)}`;
}

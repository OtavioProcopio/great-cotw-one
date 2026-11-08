import { escapeHtml as e } from '../escape';
import { icon, type IconName } from '../icons';
import type { EquipmentItem, EquipmentKind } from '../types';

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
export function thumb(item: EquipmentItem, icons: Record<string, string>, size = 56): string {
  const url = icons[item.id];
  const fallback = `<span class="eq-fallback" aria-hidden="true">${icon(KIND_ICON[item.kind], size >= 96 ? 44 : 26)}</span>`;
  return `<span class="eq-thumb" style="--size:${String(size)}px">${
    url ? `<img src="${e(url)}" alt="" />` : `${fallback}<img data-equip-icon="${e(item.id)}" alt="" hidden />`
  }</span>`;
}

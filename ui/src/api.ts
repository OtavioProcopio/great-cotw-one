import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { ask, open } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { type Update, check } from '@tauri-apps/plugin-updater';

import type { AnimalReserve, CommandError, Diagnostics, EditRequest, EditResult, EquipmentChange, EquipmentEditResult, GearAdvice, HistoryItem, Inventory, Kit, License, MapReserve, Profile, PurchaseStatus, ReserveMap, ReserveMissions, RestoreResult, RewriteResult, SpeciesEditor, UnlockRequest, UnlockResult, UpdateInfo, ZonesPreview } from './types';

/** Atualização achada pela última procura (o objeto do plugin, que sabe baixar e instalar). */
let found: Update | null = null;

/** Tudo que a interface precisa do app; substituível nos testes. */
export interface Api {
  diagnostics(): Promise<Diagnostics>;
  selectSaveDir(path: string): Promise<void>;
  selectGameDir(path: string): Promise<void>;
  resetChoices(): Promise<void>;
  history(): Promise<HistoryItem[]>;
  restore(backup: string): Promise<RestoreResult>;
  /** Regrava um arquivo do save sem mudanças (teste de gravação segura). */
  rewriteUnchanged(file: string): Promise<RewriteResult>;
  /** Reservas que o mapa consegue mostrar. */
  mapReserves(): Promise<MapReserve[]>;
  /** Wiki dos animais: espécies de cada reserva e o que o save sabe delas. */
  animals(): Promise<AnimalReserve[]>;
  /** Os animais de uma espécie numa reserva e as regras de edição (ver é gratuito). */
  speciesEditor(reserve: number, species: string): Promise<SpeciesEditor>;
  /** Grava a edição de animais que já existem, com backup antes. Exige Premium. */
  editAnimals(request: EditRequest): Promise<EditResult>;
  /** Perfil do jogador: nível, marcas e conquistas da Steam. */
  profile(): Promise<Profile>;
  /** Equipamentos do jogador: posse, quantidades, estatísticas por arma e conjuntos salvos. */
  equipment(): Promise<Inventory>;
  /** Sugestões de arma, munição e mira para uma espécie; `null` se o jogo não der classe a ela. */
  gearAdvice(species: string): Promise<GearAdvice | null>;
  reserveKit(reserve: number): Promise<Kit | null>;
  /** Versão instalada do app. */
  appVersion(): Promise<string>;
  /** Procura uma versão nova assinada; `null` se já está na última. */
  checkUpdate(): Promise<UpdateInfo | null>;
  /** Baixa e instala a versão achada por `checkUpdate` e reabre o app; `progress` recebe de 0 a 100. */
  installUpdate(progress: (percent: number) => void): Promise<void>;
  /** Entrega itens, ajusta quantidades e dinheiro, com backup antes. Exige Premium. */
  editEquipment(changes: EquipmentChange[]): Promise<EquipmentEditResult>;
  /** Ícones (`data:image/png`) dos itens pedidos, lidos do jogo; os que não existirem ficam de fora. */
  equipmentIcons(ids: string[]): Promise<Record<string, string>>;
  /** Progresso das missões em cada reserva. */
  missions(): Promise<ReserveMissions[]>;
  /** Zonas, grupos, espécies e imagem de fundo de uma reserva. */
  reserveMap(reserve: number): Promise<ReserveMap>;
  /** O que a liberação do mapa faria em cada reserva (não grava nada). */
  zonesPreview(): Promise<ZonesPreview>;
  /** Libera o mapa (reservas, animais ou necessidades escolhidos), com backup antes. Exige Premium. */
  unlockZones(request: UnlockRequest): Promise<UnlockResult>;
  /** Avisa o app do idioma escolhido (nomes de animais, missões e conquistas seguem ele). */
  setLanguage(tag: string): Promise<void>;
  /** Situação da licença Premium. */
  license(): Promise<License>;
  /** Ativa uma chave de licença colada pelo usuário (confere sem internet). */
  activateLicense(key: string): Promise<License>;
  /** Remove a chave de licença guardada neste computador. */
  removeLicense(): Promise<License>;
  startPurchase(plan: 'monthly' | 'yearly', token: string): Promise<void>;
  checkPurchase(): Promise<PurchaseStatus>;
  cancelPurchase(): Promise<void>;
  /** Abre o seletor de pasta; `null` se o usuário cancelar. */
  pickFolder(title: string): Promise<string | null>;
  /** Pergunta ao usuário; `okLabel` é o texto do botão que confirma. */
  confirm(message: string, okLabel: string): Promise<boolean>;
}

export function isCommandError(value: unknown): value is CommandError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    typeof value.code === 'string' &&
    'message' in value &&
    typeof value.message === 'string'
  );
}

export const tauriApi: Api = {
  diagnostics: () => invoke<Diagnostics>('diagnostics'),
  selectSaveDir: (path) => invoke('select_save_dir', { path }),
  selectGameDir: (path) => invoke('select_game_dir', { path }),
  resetChoices: () => invoke('reset_choices'),
  history: () => invoke<HistoryItem[]>('history'),
  restore: (backup) => invoke<RestoreResult>('restore_backup', { backup }),
  rewriteUnchanged: (file) => invoke<RewriteResult>('rewrite_unchanged', { file }),
  mapReserves: () => invoke<MapReserve[]>('map_reserves'),
  animals: () => invoke<AnimalReserve[]>('animals'),
  speciesEditor: (reserve, species) => invoke<SpeciesEditor>('species_editor', { reserve, species }),
  editAnimals: (request) => invoke<EditResult>('edit_animals', { request }),
  profile: () => invoke<Profile>('profile'),
  equipment: () => invoke<Inventory>('equipment'),
  gearAdvice: (species) => invoke<GearAdvice | null>('gear_advice', { species }),
  reserveKit: (reserve) => invoke<Kit | null>('reserve_kit', { reserve }),
  appVersion: () => getVersion(),
  checkUpdate: async () => {
    found = await check();
    return found ? { version: found.version, notes: found.body ?? '', date: found.date ?? null } : null;
  },
  installUpdate: async (progress) => {
    if (!found) throw new Error('nenhuma atualização encontrada');
    let total = 0;
    let done = 0;
    await found.downloadAndInstall((event) => {
      if (event.event === 'Started') total = event.data.contentLength ?? 0;
      if (event.event === 'Progress') {
        done += event.data.chunkLength;
        progress(total > 0 ? Math.min(100, (done / total) * 100) : 0);
      }
      if (event.event === 'Finished') progress(100);
    });
    await relaunch();
  },
  editEquipment: (changes) => invoke<EquipmentEditResult>('edit_equipment', { changes }),
  equipmentIcons: (ids) => invoke<Record<string, string>>('equipment_icons', { ids }),
  missions: () => invoke<ReserveMissions[]>('missions'),
  reserveMap: (reserve) => invoke<ReserveMap>('reserve_map', { reserve }),
  zonesPreview: () => invoke<ZonesPreview>('zones_preview'),
  unlockZones: (request) => invoke<UnlockResult>('unlock_zones', { request }),
  setLanguage: (tag) => invoke('set_language', { tag }),
  license: () => invoke<License>('license'),
  activateLicense: (key) => invoke<License>('activate_license', { key }),
  removeLicense: () => invoke<License>('remove_license'),
  startPurchase: (plan, token) => invoke<undefined>('start_purchase', { plan, token }),
  checkPurchase: () => invoke<PurchaseStatus>('check_purchase'),
  cancelPurchase: () => invoke<undefined>('cancel_purchase'),
  pickFolder: async (title) => {
    const chosen = await open({ directory: true, multiple: false, title });
    return typeof chosen === 'string' ? chosen : null;
  },
  confirm: (message, okLabel) => ask(message, { kind: 'warning', okLabel, cancelLabel: 'Cancelar' }),
};

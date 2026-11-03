import { invoke } from '@tauri-apps/api/core';
import { ask, open } from '@tauri-apps/plugin-dialog';

import type {
  AnimalReserve,
  CommandError,
  Diagnostics,
  HistoryItem,
  MapReserve,
  Profile,
  RestoreResult,
  ReserveMissions,
  ReserveMap,
  RewriteResult,
  UnlockResult,
  ZonesPreview,
} from './types';

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
  /** Perfil do jogador: nível, marcas e conquistas da Steam. */
  profile(): Promise<Profile>;
  /** Progresso das missões em cada reserva. */
  missions(): Promise<ReserveMissions[]>;
  /** Zonas, grupos, espécies e imagem de fundo de uma reserva. */
  reserveMap(reserve: number): Promise<ReserveMap>;
  /** O que a liberação do mapa faria em cada reserva (não grava nada). */
  zonesPreview(): Promise<ZonesPreview>;
  /** Libera o mapa nas reservas indicadas, com backup antes. */
  unlockZones(reserves: number[]): Promise<UnlockResult>;
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
  profile: () => invoke<Profile>('profile'),
  missions: () => invoke<ReserveMissions[]>('missions'),
  reserveMap: (reserve) => invoke<ReserveMap>('reserve_map', { reserve }),
  zonesPreview: () => invoke<ZonesPreview>('zones_preview'),
  unlockZones: (reserves) => invoke<UnlockResult>('unlock_zones', { reserves }),
  pickFolder: async (title) => {
    const chosen = await open({ directory: true, multiple: false, title });
    return typeof chosen === 'string' ? chosen : null;
  },
  confirm: (message, okLabel) => ask(message, { kind: 'warning', okLabel, cancelLabel: 'Cancelar' }),
};

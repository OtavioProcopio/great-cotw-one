// Dados de exemplo para ver a interface no navegador (`npm run dev` + `?mock`). Não vão para o app.
import { mockIPC } from '@tauri-apps/api/mocks';

import type { Diagnostics, HistoryItem, ReserveMap } from './types';

const diagnostics: Diagnostics = {
  limitedMode: false,
  game: {
    found: true,
    root: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\theHunterCotW',
    archives: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\theHunterCotW\\archives_win64',
    tabFiles: 144,
    buildId: '26051119',
    manual: false,
  },
  save: {
    status: 'found',
    account: {
      steamId: '76561198000000000',
      path: 'C:\\Users\\voce\\OneDrive\\Documentos\\Avalanche Studios\\COTW\\Saves\\76561198000000000',
      lastModified: 1_793_600_000,
    },
    accounts: [],
    searched: [],
    manual: false,
  },
  reserves: [
    { id: 0, name: 'Hirschfelden' },
    { id: 2, name: 'Medved-Taiga' },
    { id: 99, name: 'Reserva 99' },
  ],
  files: [
    { name: 'animal_population_0', size: 105_074, kind: 'population', reserve: 0, state: 'ok' },
    { name: 'animal_population_2', size: 90_561, kind: 'population', reserve: 2, state: 'ok' },
    { name: 'found_need_zones_adf', size: 436_671, kind: 'known', reserve: null, state: 'ok' },
    { name: 'settings_adf', size: 478, kind: 'known', reserve: null, state: 'unreadable' },
    { name: 'steam_autocloud.vdf', size: 52, kind: 'unrecognized', reserve: null, state: 'unrecognized' },
  ],
};

const history: HistoryItem[] = [
  {
    id: 2,
    at: '2026-11-02_15-30-00',
    kind: 'change',
    function: 'Liberar zonas',
    summary: '12 zonas de necessidade adicionadas em 3 reservas',
    files: ['found_need_zones_adf'],
    backup: 'C:\\Users\\voce\\AppData\\Roaming\\Great COTW One\\backups\\2026-11-02_15-30-00',
    backupMissing: false,
  },
  {
    id: 1,
    at: '2026-10-28_21-04-10',
    kind: 'change',
    function: 'Editor de animais',
    summary: '1 diamante por grupo',
    files: ['animal_population_0', 'animal_population_2'],
    backup: 'C:\\Users\\voce\\AppData\\Roaming\\Great COTW One\\backups\\2026-10-28_21-04-10',
    backupMissing: true,
  },
];

/** Reserva fictícia para ver o mapa no navegador, sem o jogo. */
function mockMap(): ReserveMap {
  const species = [
    { id: 'brown_bear', name: 'Urso-pardo' },
    { id: 'moose', name: 'Alce' },
    { id: 'reindeer', name: 'Rena' },
  ];
  const zones: ReserveMap['zones'] = [];
  for (let i = 0; i < 90; i += 1) {
    const kind = (i % 3) + 1;
    zones.push({
      x: -12000 + (i % 10) * 640 + (i % 7) * 40,
      z: -12000 + Math.floor(i / 10) * 700 + (i % 5) * 60,
      kind,
      species: i % 3,
      pending: i % 4 === 0,
      start: [4, 8, 12, 16, 20][i % 5] ?? 4,
      end: [8, 12, 16, 20, 0][i % 5] ?? 8,
    });
  }
  const groups: ReserveMap['groups'] = [
    {
      number: 1,
      species: 1,
      stops: [1, 4, 7, 10, 13, 16],
      animals: [
        { male: true, weight: 560, score: 210, greatOne: true },
        { male: false, weight: 410, score: 0, greatOne: false },
      ],
    },
  ];
  return { id: 2, name: 'Medved-Taiga', image: null, x0: -16384, z0: -16384, span: 16384, approximate: false, species, zones, groups };
}

export function installMock(): void {
  mockIPC((command, args) => {
    switch (command) {
      case 'diagnostics':
        return diagnostics;
      case 'history':
        return history;
      case 'animals':
        return [
          {
            id: 2,
            name: 'Medved-Taiga',
            cover: null,
            played: true,
            species: [
              { id: 'moose', name: 'Alce', areas: 1200, living: { groups: 3, animals: 9, males: 4, females: 5, greatOnes: 1, bestScore: 312, heaviest: 640 }, harvests: [{ score: 301, medal: 3, at: 1622753604 }] },
              { id: 'brown_bear', name: 'Urso-pardo', areas: 800, living: null, harvests: [] },
            ],
          },
        ];
      case 'profile':
        return {
          steamId: '76561198125950350',
          level: 60,
          xp: 293_384,
          cash: 300_618,
          shotsFired: 4_120,
          shotsMissed: 910,
          totalKills: 612,
          oneShotKills: 301,
          recordScore: 900,
          furthestShot: 412,
          trophies: 26,
          reserves: 14,
          hasAchievements: true,
          achievements: [
            { id: 'ach_001', name: 'A milha', description: 'Viaje 1 milha (1,609 quilômetro) a pé.', hidden: false, unlocked: true },
            { id: 'ach_002', name: 'Maratona', description: 'Viaje 26,2 milhas a pé.', hidden: false, unlocked: false },
          ],
        };
      case 'missions':
        return [
          {
            id: 2,
            name: 'Medved-Taiga',
            total: 92,
            done: 17,
            active: 6,
            mainTotal: 40,
            mainDone: 12,
            missions: [
              { id: 273, text: 'Fotografe um alce no Lago Zhizni.', state: 'active', main: true, objectives: [{ done: 1, required: 3 }] },
              { id: 306, text: 'Abata e recolha um lince.', state: 'done', main: true, objectives: [{ done: 1, required: 1 }] },
            ],
          },
        ];
      case 'map_reserves':
        return [
          { id: 2, name: 'Medved-Taiga', cover: null },
          { id: 0, name: 'Hirschfelden', cover: null },
        ];
      case 'reserve_map':
        return mockMap();
      case 'zones_preview':
        return {
          total: 7_330,
          reserves: [
            { id: 2, name: 'Medved-Taiga', state: 'ready', newZones: 3_511, estimated: 89, skipped: 0 },
            { id: 13, name: 'Revontuli Coast', state: 'ready', newZones: 3_819, estimated: 181, skipped: 805 },
            { id: 0, name: 'Hirschfelden', state: 'nothingToDo', newZones: 0, estimated: 0, skipped: 0 },
            { id: 18, name: 'Reserva 18', state: 'needsDiscovery', newZones: 0, estimated: 0, skipped: 0 },
          ],
        };
      case 'unlock_zones':
        return { zonesAdded: 7_330, reserves: 2, backup: 'C:\\...\\backups\\2026-11-02_16-10-00' };
      case 'restore_backup':
        return { safetyBackup: 'C:\\...\\backups\\2026-11-02_16-00-00' };
      case 'rewrite_unchanged':
        return { backup: 'C:\\...\\backups\\2026-11-02_16-05-00', oldSize: 90_561, newSize: 85_622 };
      case 'plugin:dialog|message':
        // `ask` devolve o rótulo do botão apertado: confirma sempre.
        return (args as { buttons?: { OkCancelCustom?: string[] } }).buttons?.OkCancelCustom?.[0] ?? 'Yes';
      case 'plugin:dialog|open':
        return 'C:\\Pasta\\Escolhida';
      default:
        return null;
    }
  });
}

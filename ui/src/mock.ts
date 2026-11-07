// Dados de exemplo para ver a interface no navegador (`npm run dev` + `?mock`). Não vão para o app.
import { mockIPC } from '@tauri-apps/api/mocks';

import { mockAnimals, mockMapReserves, mockMissions, mockProfile, terrainArt } from './mock-data';
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
    { id: 1, name: 'Layton Lake' },
    { id: 2, name: 'Medved-Taiga' },
    { id: 8, name: 'Cuatro Colinas' },
    { id: 16, name: 'Emerald Coast' },
    { id: 17, name: 'Sundarpatan' },
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
  for (let i = 0; i < 260; i += 1) {
    const angle = i * 2.39996;
    const radius = 600 + (i % 37) * 175;
    zones.push({
      x: -8192 + Math.cos(angle) * radius * 1.1,
      z: -8192 + Math.sin(angle) * radius,
      kind: (i % 3) + 1,
      species: i % 3,
      pending: i % 5 === 0,
      start: [4, 8, 12, 16, 20][i % 5] ?? 4,
      end: [8, 12, 16, 20, 0][i % 5] ?? 8,
    });
  }
  const groups: ReserveMap['groups'] = [
    {
      number: 1,
      species: 1,
      stops: [1, 4, 7, 10, 13, 16, 22],
      animals: [
        { male: true, weight: 560, score: 210, greatOne: true },
        { male: false, weight: 410, score: 0, greatOne: false },
      ],
    },
  ];
  return { id: 2, name: 'Medved-Taiga', image: terrainArt(), x0: -16384, z0: -16384, span: 16384, approximate: false, species, zones, groups };
}

export function installMock(): void {
  mockIPC((command, args) => {
    switch (command) {
      case 'diagnostics':
        return diagnostics;
      case 'history':
        return history;
      case 'animals':
        return mockAnimals();
      case 'species_editor':
        return {
          reserve: 2,
          reserveName: 'Medved-Taiga',
          species: 'moose',
          name: 'Alce',
          greatOneAllowed: true,
          limits: {
            male: { minWeight: 90, maxWeight: 660, minScore: 0, maxScore: 330, typicalWeight: 400, typicalScore: 150 },
            female: { minWeight: 80, maxWeight: 500, minScore: 0, maxScore: 200, typicalWeight: 320, typicalScore: 90 },
          },
          medals: { bronze: 32.3, silver: 86.2, gold: 140.1, diamond: 275, max: 301.96 },
          groups: [
            {
              number: 1,
              animals: [
                { index: 0, male: true, weight: 560, score: 210, greatOne: true },
                { index: 1, male: false, weight: 410, score: 0, greatOne: false },
              ],
            },
          ],
        };
      case 'edit_animals':
        return { animals: 1, backup: 'C:\\...\\backups\\2026-11-02_17-00-00' };
      case 'profile':
        return mockProfile();
      case 'missions':
        return mockMissions();
      case 'map_reserves':
        return mockMapReserves();
      case 'reserve_map':
        return mockMap();
      case 'zones_preview':
        return {
          total: 7_330,
          reserves: [
            { id: 2, name: 'Medved-Taiga', state: 'ready', newZones: 3_511, estimated: 89, skipped: 0, species: [{ id: 'moose', name: 'Alce', needs: [1_000, 1_500, 900] }, { id: 'brown_bear', name: 'Urso-pardo', needs: [40, 40, 31] }] },
            { id: 13, name: 'Revontuli Coast', state: 'ready', newZones: 3_819, estimated: 181, skipped: 805, species: [{ id: 'moose', name: 'Alce', needs: [1_900, 1_000, 919] }] },
            { id: 0, name: 'Hirschfelden', state: 'nothingToDo', newZones: 0, estimated: 0, skipped: 0, species: [] },
            { id: 18, name: 'Reserva 18', state: 'needsDiscovery', newZones: 0, estimated: 0, skipped: 0, species: [] },
          ],
        };
      case 'start_purchase':
      case 'cancel_purchase':
        return undefined;
      case 'check_purchase':
        return { state: 'none', plan: null, license: { premium: false, state: 'free', email: null, expires: null, daysLeft: null, plan: null } };
      case 'license':
        return new URLSearchParams(location.search).has('premium')
          ? { premium: true, state: 'active', email: 'ana@exemplo.com', expires: 1_830_000_000, daysLeft: 200, plan: 'yearly' }
          : { premium: false, state: 'free', email: null, expires: null, daysLeft: null, plan: null };
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

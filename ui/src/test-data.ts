// Dados de exemplo compartilhados pelos testes.
import type { Api } from './api';
import type { License, SpeciesEditor, UnlockRequest } from './types';
import type { AnimalReserve, Diagnostics, HistoryItem, MapReserve, Profile, ReserveMap, ReserveMissions, ZonesPreview } from './types';
import { mockInventory } from './mock-data';

export const foundDiagnostics: Diagnostics = {
  limitedMode: false,
  game: {
    found: true,
    root: 'C:\\Steam\\steamapps\\common\\theHunterCotW',
    archives: 'C:\\Steam\\steamapps\\common\\theHunterCotW\\archives_win64',
    tabFiles: 144,
    buildId: '26051119',
    manual: false,
  },
  save: {
    status: 'found',
    account: { steamId: '111', path: 'C:\\Saves\\111', lastModified: 1_793_600_000 },
    accounts: [],
    searched: [],
    manual: false,
  },
  reserves: [
    { id: 2, name: 'Medved-Taiga' },
    { id: 99, name: 'Reserva 99' },
  ],
  files: [
    { name: 'animal_population_2', size: 90_561, kind: 'population', reserve: 2, state: 'ok' },
    { name: 'settings_adf', size: 478, kind: 'known', reserve: null, state: 'unreadable' },
    { name: '<script>alert(1)</script>', size: 5, kind: 'unrecognized', reserve: null, state: 'unrecognized' },
  ],
};

export const historyItems: HistoryItem[] = [
  {
    id: 2,
    at: '2026-11-02_15-30-00',
    kind: 'change',
    function: 'Liberar zonas',
    summary: '12 zonas adicionadas',
    files: ['found_need_zones_adf'],
    backup: 'C:\\Backups\\2026-11-02_15-30-00',
    backupMissing: false,
  },
  {
    id: 1,
    at: '2026-10-28_21-04-10',
    kind: 'restore',
    function: 'Restaurar backup',
    summary: 'Pasta restaurada',
    files: [],
    backup: 'C:\\Backups\\2026-10-28_21-04-10',
    backupMissing: true,
  },
];

export const zonesPreview: ZonesPreview = {
  total: 7_330,
  reserves: [
    {
      id: 2,
      name: 'Medved-Taiga',
      state: 'ready',
      newZones: 3_511,
      estimated: 89,
      skipped: 0,
      species: [
        { id: 'moose', name: 'Alce', needs: [1_000, 1_500, 900] },
        { id: 'brown_bear', name: 'Urso-pardo', needs: [40, 40, 31] },
      ],
    },
    {
      id: 13,
      name: 'Revontuli Coast',
      state: 'ready',
      newZones: 3_819,
      estimated: 0,
      skipped: 805,
      species: [{ id: 'moose', name: 'Alce', needs: [1_900, 1_000, 919] }],
    },
    { id: 0, name: 'Hirschfelden', state: 'nothingToDo', newZones: 0, estimated: 0, skipped: 0, species: [] },
    { id: 18, name: 'Reserva 18', state: 'needsDiscovery', newZones: 0, estimated: 0, skipped: 0, species: [] },
    { id: 5, name: 'Reserva 5', state: 'noData', newZones: 0, estimated: 0, skipped: 0, species: [] },
  ],
};

export const mapReserves: MapReserve[] = [
  { id: 2, name: 'Medved-Taiga', cover: null },
  { id: 0, name: 'Hirschfelden', cover: 'data:image/jpeg;base64,AAAA' },
];

/** Reserva pequena: 6 zonas (2 ainda não descobertas), 2 espécies e 2 grupos. */
export const sampleMap: ReserveMap = {
  id: 2,
  name: 'Medved-Taiga',
  image: null,
  x0: -16384,
  z0: -16384,
  span: 16384,
  approximate: false,
  species: [
    { id: 'brown_bear', name: 'Urso-pardo' },
    { id: 'moose', name: 'Alce' },
  ],
  zones: [
    { x: -12000, z: -12000, kind: 1, species: 0, pending: false, start: 4, end: 8 },
    { x: -11000, z: -11500, kind: 2, species: 0, pending: false, start: 8, end: 12 },
    { x: -10000, z: -11000, kind: 3, species: 0, pending: true, start: 12, end: 16 },
    { x: -9000, z: -10500, kind: 1, species: 1, pending: false, start: 20, end: 0 },
    { x: -8000, z: -10000, kind: 2, species: 1, pending: true, start: 0, end: 4 },
    { x: -7000, z: -9500, kind: 3, species: -1, pending: false, start: 4, end: 8 },
  ],
  groups: [
    {
      number: 1,
      species: 0,
      stops: [0, 1, 2],
      animals: [
        { male: true, weight: 480.4, score: 210.2, greatOne: true },
        { male: false, weight: 350, score: 0, greatOne: false },
      ],
    },
    {
      number: 2,
      species: 0,
      stops: [0, 2],
      animals: [{ male: true, weight: 300, score: 90, greatOne: false }],
    },
    { number: 3, species: 1, stops: [3, 4, 99], animals: [] },
  ],
};

export const FREE_LICENSE: License = {
  premium: false,
  state: 'free',
  email: null,
  expires: null,
  daysLeft: null,
  plan: null,
};
export const ACTIVE_LICENSE: License = {
  premium: true,
  state: 'active',
  email: 'ana@exemplo.com',
  expires: 1_830_000_000,
  daysLeft: 200,
  plan: 'yearly',
};

export const sampleEditor: SpeciesEditor = {
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

export const sampleAnimals: AnimalReserve[] = [
  {
    id: 2,
    name: 'Medved-Taiga',
    cover: 'data:image/jpeg;base64,AAAA',
    played: true,
    species: [
      {
        id: 'moose',
        name: 'Alce',
        group: 'deer',
        areas: 1_200,
        living: { groups: 3, animals: 9, males: 4, females: 5, greatOnes: 1, diamonds: 2, diamondMin: 255.1, bestScore: 312.4, heaviest: 640 },
        harvests: [
          { score: 301.5, medal: 3, at: 1_622_753_604 },
          { score: 120, medal: 0, at: 0 },
        ],
      },
      { id: 'brown_bear', name: '<i>Urso-pardo</i>', group: 'bear', areas: 800, living: null, harvests: [] },
    ],
  },
  {
    id: 8,
    name: 'Cuatro Colinas',
    cover: null,
    played: false,
    species: [{ id: 'moose', name: 'Alce', group: 'deer', areas: 40, living: null, harvests: [] }],
  },
];

export const sampleProfile: Profile = {
  steamId: '76561198125950350',
  level: 60,
  xp: 293_384,
  cash: 300_618,
  shotsFired: 1_000,
  shotsMissed: 250,
  totalKills: 124,
  oneShotKills: 44,
  recordScore: 900.4,
  furthestShot: 387.2,
  trophies: 26,
  reserves: 14,
  hasAchievements: true,
  achievements: [
    { id: 'ach_001', name: 'A milha', description: 'Viaje 1 milha a pé.', hidden: false, unlocked: false },
    { id: 'ach_002', name: 'Maratona', description: 'Viaje 42 km a pé.', hidden: false, unlocked: true },
    { id: 'ach_003', name: '<b>Segredo</b>', description: 'Escondida.', hidden: true, unlocked: false },
  ],
};

export const sampleMissions: ReserveMissions[] = [
  {
    id: 2,
    name: 'Medved-Taiga',
    total: 4,
    done: 1,
    active: 2,
    mainTotal: 3,
    mainDone: 1,
    missions: [
      { id: 1, text: 'Crie pressão de caça.', state: 'done', main: true, objectives: [{ done: 1, required: 1 }] },
      {
        id: 2,
        text: 'Fotografe um alce.',
        state: 'active',
        main: true,
        objectives: [
          { done: 1, required: 2 },
          { done: 0, required: 1 },
        ],
      },
      { id: 3, text: '', state: 'active', main: false, objectives: [] },
      { id: 4, text: 'Abata um urso.', state: 'locked', main: true, objectives: [{ done: 0, required: 0 }] },
    ],
  },
  {
    id: 0,
    name: 'Hirschfelden',
    total: 0,
    done: 0,
    active: 0,
    mainTotal: 0,
    mainDone: 0,
    missions: [],
  },
];

/** Falha de comando como o Tauri entrega: tem `code` e `message`. */
export function commandError(code: string, message = 'detalhe'): Promise<never> {
  return Promise.reject(Object.assign(new Error(message), { code }));
}

/** API falsa: cada método devolve valores de exemplo e registra as chamadas. */
export function fakeApi(overrides: Partial<Api> = {}): Api & { calls: string[]; requests: UnlockRequest[]; languages: string[] } {
  const calls: string[] = [];
  const requests: UnlockRequest[] = [];
  const languages: string[] = [];
  const record =
    <T>(name: string, value: T) =>
    (...args: unknown[]): Promise<T> => {
      calls.push([name, ...args.map(String)].join(':'));
      return Promise.resolve(value);
    };
  return {
    calls,
    requests,
    languages,
    diagnostics: record('diagnostics', foundDiagnostics),
    selectSaveDir: record('selectSaveDir', undefined),
    selectGameDir: record('selectGameDir', undefined),
    resetChoices: record('resetChoices', undefined),
    history: record('history', historyItems),
    restore: record('restore', { safetyBackup: 'C:\\Backups\\seguranca' }),
    rewriteUnchanged: record('rewriteUnchanged', {
      backup: 'C:\\Backups\\teste',
      oldSize: 90_561,
      newSize: 85_622,
    }),
    setLanguage: (tag) => {
      languages.push(tag);
      return Promise.resolve();
    },
    license: record('license', FREE_LICENSE),
    activateLicense: record('activateLicense', ACTIVE_LICENSE),
    removeLicense: record('removeLicense', FREE_LICENSE),
    startPurchase: record('startPurchase', undefined),
    checkPurchase: record('checkPurchase', { state: 'none', plan: null, license: FREE_LICENSE }),
    cancelPurchase: record('cancelPurchase', undefined),
    animals: record('animals', sampleAnimals),
    speciesEditor: record('speciesEditor', sampleEditor),
    editAnimals: record('editAnimals', { animals: 1, backup: 'C:\\Backups\\animais' }),
    profile: record('profile', sampleProfile),
    equipment: record('equipment', mockInventory()),
    equipmentIcons: record('equipmentIcons', {}),
    missions: record('missions', sampleMissions),
    mapReserves: record('mapReserves', mapReserves),
    reserveMap: record('reserveMap', sampleMap),
    zonesPreview: record('zonesPreview', zonesPreview),
    unlockZones: (request) => {
      requests.push(request);
      calls.push(`unlockZones:${request.reserves.join(',')}`);
      return Promise.resolve({ zonesAdded: 7_330, reserves: 2, backup: 'C:\\Backups\\mapa' });
    },
    pickFolder: record('pickFolder', 'C:\\Escolhida'),
    confirm: record('confirm', true),
    ...overrides,
  };
}

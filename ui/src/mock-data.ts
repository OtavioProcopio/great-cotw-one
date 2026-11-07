// Dados e artes de exemplo para ver a interface (e tirar prints) sem o jogo. Nada aqui vem do jogo:
// as capas e o mapa são desenhos simples feitos em SVG. Só em desenvolvimento.
import type { AnimalReserve, EquipmentItem, GearAdvice, Inventory, MapReserve, Profile, ReserveMissions } from './types';

function svgUrl(svg: string): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

/** Capa de reserva: céu em degradê, montanhas e sol, na cor (matiz) pedida. */
export function coverArt(hue: number): string {
  return svgUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 250 160">
    <defs><linearGradient id="s" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="hsl(${String(hue)} 60% 62%)"/><stop offset="1" stop-color="hsl(${String((hue + 40) % 360)} 70% 82%)"/>
    </linearGradient></defs>
    <rect width="250" height="160" fill="url(#s)"/>
    <circle cx="190" cy="42" r="18" fill="hsl(${String((hue + 40) % 360)} 90% 92%)" opacity=".9"/>
    <path d="M0 120 L50 70 L90 105 L140 55 L200 110 L250 80 V160 H0Z" fill="hsl(${String(hue)} 35% 38%)"/>
    <path d="M0 140 L60 105 L120 135 L180 100 L250 138 V160 H0Z" fill="hsl(${String(hue)} 40% 26%)"/>
  </svg>`);
}

/** "Mapa" desenhado: terreno verde, relevo e lagos, só para o fundo do mapa de exemplo. */
export function terrainArt(): string {
  return svgUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
    <defs><radialGradient id="g" cx=".5" cy=".5" r=".6"><stop offset="0" stop-color="#6f8f55"/><stop offset="1" stop-color="#3d5a3a"/></radialGradient>
    <filter id="b"><feGaussianBlur stdDeviation="14"/></filter></defs>
    <rect width="1024" height="1024" fill="#16231a"/>
    <rect x="40" y="40" width="944" height="944" rx="120" fill="url(#g)"/>
    <g filter="url(#b)" opacity=".55"><ellipse cx="320" cy="360" rx="190" ry="120" fill="#a8946a"/><ellipse cx="700" cy="640" rx="210" ry="150" fill="#8b7c58"/><ellipse cx="560" cy="250" rx="130" ry="90" fill="#9db27a"/></g>
    <path d="M120 600 C300 520 420 700 620 620 S900 560 940 640" stroke="#5aa4c8" stroke-width="26" fill="none" stroke-linecap="round"/>
    <ellipse cx="760" cy="300" rx="90" ry="60" fill="#5aa4c8"/><ellipse cx="260" cy="800" rx="70" ry="44" fill="#5aa4c8"/>
  </svg>`);
}

const NAMES: [number, string, number][] = [
  [2, 'Medved-Taiga', 205],
  [0, 'Hirschfelden', 110],
  [1, 'Layton Lake', 150],
  [8, 'Cuatro Colinas', 38],
  [16, 'Emerald Coast', 175],
  [17, 'Sundarpatan', 280],
];

export function mockMapReserves(): MapReserve[] {
  return NAMES.map(([id, name, hue]) => ({ id, name, cover: coverArt(hue) }));
}

const ACHIEVEMENTS: [string, string, boolean, boolean][] = [
  ['A milha', 'Viaje 1 milha a pé.', false, true],
  ['Maratona', 'Viaje 42 quilômetros a pé.', false, true],
  ['Primeiro troféu', 'Pendure o seu primeiro troféu no salão.', false, true],
  ['Tiro certeiro', 'Abata um animal a mais de 300 metros.', false, true],
  ['Fotógrafo', 'Fotografe 10 espécies diferentes.', false, true],
  ['Caçador de lendas', 'Abata um animal Great One.', false, false],
  ['Colecionador', 'Tenha 50 troféus no salão.', false, false],
  ['Conquista secreta', '', true, false],
  ['Explorador', 'Descubra todas as zonas de uma reserva.', false, false],
  ['Mestre dos rifles', 'Chegue ao nível máximo de rifles.', false, true],
];

export function mockProfile(): Profile {
  return {
    steamId: '76561198000000000',
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
    achievements: ACHIEVEMENTS.map(([name, description, hidden, unlocked], i) => ({
      id: `ach_${String(i + 1).padStart(3, '0')}`,
      name,
      description,
      hidden,
      unlocked,
    })),
  };
}

export function mockMissions(): ReserveMissions[] {
  const texts = [
    'Crie pressão de caça perto do Lago Zhizni.',
    'Fotografe um alce na margem do rio.',
    'Abata e recolha um lince nas Terras de Lesnye.',
    'Encontre as pegadas de um urso-pardo.',
    'Vá até a cabana abandonada.',
    'Identifique os rastros de uma rena.',
    'Abata um javali a mais de 150 metros.',
    'Fotografe três espécies diferentes.',
  ];
  const states = ['done', 'done', 'active', 'active', 'active', 'locked', 'locked', 'locked'] as const;
  const build = (id: number, name: string, total: number, done: number, active: number): ReserveMissions => ({
    id,
    name,
    total,
    done,
    active,
    mainTotal: Math.round(total * 0.6),
    mainDone: Math.round(done * 0.7),
    missions: texts.map((text, i) => ({
      id: id * 100 + i,
      text,
      state: states[i] ?? 'locked',
      main: i % 3 !== 2,
      objectives:
        states[i] === 'locked'
          ? [{ done: 0, required: 2 }]
          : [
              { done: states[i] === 'done' ? 2 : 1, required: 2 },
              { done: states[i] === 'done' ? 1 : 0, required: 1 },
            ],
    })),
  });
  return [build(2, 'Medved-Taiga', 92, 51, 6), build(0, 'Hirschfelden', 124, 118, 6), build(8, 'Cuatro Colinas', 76, 7, 5)];
}

const SPECIES: [string, string][] = [
  ['moose', 'Alce'],
  ['brown_bear', 'Urso-pardo'],
  ['reindeer', 'Rena'],
  ['wild_boar', 'Javali'],
  ['eurasian_lynx', 'Lince-euroasiático'],
  ['gray_wolf', 'Lobo-cinzento'],
  ['musk_deer', 'Cervo-almiscarado'],
  ['capercaillie', 'Tetraz-grande'],
];

export function mockAnimals(): AnimalReserve[] {
  return NAMES.map(([id, name, hue], r) => ({
    id,
    name,
    cover: coverArt(hue),
    played: r < 4,
    species: SPECIES.map(([sid, sname], i) => ({
      id: sid,
      name: sname,
      group: ['deer', 'cat', 'bear', 'bird', 'canid', 'pig'][i % 6] ?? '',
      areas: 300 + ((i * 137 + r * 41) % 900),
      living:
        r < 4 && i % 3 !== 2
          ? {
              groups: 2 + ((i + r) % 5),
              animals: 6 + ((i * 7 + r) % 30),
              males: 3 + (i % 9),
              females: 3 + ((i + r) % 12),
              greatOnes: i % 4 === 0 ? 1 : 0,
              diamonds: i % 3 === 0 ? 2 : 0,
              diamondMin: 120,
              bestScore: 180 + i * 23,
              heaviest: 90 + i * 70,
            }
          : null,
      harvests:
        i % 2 === 0
          ? [
              { score: 280 - i * 9, medal: 3, at: 1_790_000_000 },
              { score: 210 - i * 7, medal: 1, at: 1_788_000_000 },
            ]
          : [],
    })),
  }));
}

function item(partial: Partial<EquipmentItem> & Pick<EquipmentItem, 'id' | 'kind' | 'name'>): EquipmentItem {
  return {
    price: 0,
    weight: null,
    class: null,
    boxSize: null,
    icon: null,
    owned: false,
    unlocked: false,
    inBackpack: false,
    amount: 0,
    ammo: [],
    sights: [],
    stats: null,
    attributes: { accuracy: null, recoil: null, reload: null, range: null, penetration: null, expansion: null, classes: [], zoom: null },
    unlock: null,
    ...partial,
  };
}

/** Equipamentos de exemplo (nomes e números inventados para o desenvolvimento). */
export function mockInventory(): Inventory {
  return {
    cash: 300_618,
    level: 60,
    weaponLevels: [30, 23, 25, 21],
    items: [
      item({
        id: 'equipment_weapon_ba_rifle_270_01',
        kind: 'weapon',
        name: '.270 Bolt-Action Rifle',
        class: 'rifle',
        price: 12_000,
        weight: 4,
        owned: true,
        inBackpack: true,
        amount: 1,
        ammo: ['equipment_ammo_270_sp_01', 'equipment_ammo_270_pt_01'],
        sights: ['equipment_sight_rifle_scope_3_9x44mm_01'],
        attributes: { accuracy: 55, recoil: 64, reload: 49, range: null, penetration: null, expansion: null, classes: [], zoom: null },
        stats: { shotsFired: 100, shotsMissed: 25, accuracy: 75, kills: 40, oneShotKills: 22, recordScore: 812.5, recordSpecies: 'Alce', furthestShot: 250.5, purchasedAt: 1_700_000_000 },
      }),
      item({
        id: 'equipment_weapon_longbow_01',
        kind: 'weapon',
        name: 'Longbow',
        class: 'archery',
        weight: 2.5,
        unlocked: true,
        ammo: [],
        sights: [],
      }),
      item({ id: 'equipment_ammo_270_sp_01', kind: 'ammo', name: '.270 Soft-Point Bullet', price: 440, weight: 0.03, boxSize: 10, owned: true, amount: 203, attributes: { accuracy: null, recoil: null, reload: null, range: 150, penetration: 17, expansion: 24, classes: [4, 5, 6, 7, 8], zoom: null } }),
      item({ id: 'equipment_ammo_270_pt_01', kind: 'ammo', name: '.270 Polymer-Tip Bullet', price: 560, weight: 0.03, boxSize: 10 }),
      item({ id: 'equipment_sight_rifle_scope_3_9x44mm_01', kind: 'sight', name: '3-9x 44mm Rifle Scope', weight: 0.5, owned: true, amount: 1 }),
      item({ id: 'equipment_optics_binoculars_01', kind: 'optic', name: '8x42mm Binoculars', owned: true, amount: 1 }),
      item({ id: 'equipment_lure_caller_roe_deer_01', kind: 'lure_caller', name: 'Roe Deer Caller', price: 1500, owned: true, amount: 1 }),
      item({ id: 'equipment_tent_01', kind: 'deployable', name: 'Tent', price: 8000 }),
      item({ id: 'equipment_consumable_first_aid_kit_01', kind: 'consumable', name: 'First Aid Kit', owned: true, amount: 5, inBackpack: true }),
      item({ id: 'equipment_atv_01', kind: 'vehicle', name: 'ATV', owned: true, amount: 2 }),
    ],
    loadouts: [
      {
        index: 0,
        name: 'Alces',
        unlocked: true,
        active: true,
        items: ['equipment_weapon_ba_rifle_270_01', 'equipment_ammo_270_sp_01'],
        missing: ['equipment_weapon_longbow_01'],
      },
      { index: 1, name: '', unlocked: true, active: false, items: [], missing: [] },
    ],
  };
}

/** Sugestões de exemplo para um alce (classe 8). */
export function mockAdvice(): GearAdvice {
  return {
    species: 'moose',
    name: 'Alce',
    huntClass: 8,
    ready: [
      { weapon: 'equipment_weapon_ba_rifle_270_01', ammo: 'equipment_ammo_270_sp_01', sight: 'equipment_sight_rifle_scope_3_9x44mm_01', hasWeapon: true, hasAmmo: true, cost: 0, available: true, unlock: null, classRange: [4, 8], score: 72.5 },
    ],
    toBuy: [
      { weapon: 'equipment_weapon_longbow_01', ammo: 'equipment_ammo_270_pt_01', sight: null, hasWeapon: false, hasAmmo: false, cost: 12_560, available: true, unlock: { track: 'rifle', level: 7, need: 450 }, classRange: [4, 8], score: 64 },
    ],
  };
}

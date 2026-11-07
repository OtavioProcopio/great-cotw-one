// Espelha os dados enviados pelo Rust (`cotw-core`), em camelCase.

export interface GameDiagnostic {
  found: boolean;
  root: string | null;
  archives: string | null;
  tabFiles: number;
  buildId: string | null;
  manual: boolean;
}

export interface AccountDiagnostic {
  steamId: string;
  path: string;
  /** Segundos desde 1970. */
  lastModified: number | null;
}

export type SaveStatus = 'found' | 'choose' | 'notFound';

export interface SaveDiagnostic {
  status: SaveStatus;
  account: AccountDiagnostic | null;
  accounts: AccountDiagnostic[];
  searched: string[];
  manual: boolean;
}

export type FileState = 'ok' | 'unreadable' | 'unrecognized';
export type FileKind = 'population' | 'foundIcons' | 'fogOfWar' | 'known' | 'unrecognized';

export interface FileDiagnostic {
  name: string;
  size: number;
  kind: FileKind;
  reserve: number | null;
  state: FileState;
}

export interface ReserveDiagnostic {
  id: number;
  name: string;
}

export interface Diagnostics {
  game: GameDiagnostic;
  save: SaveDiagnostic;
  reserves: ReserveDiagnostic[];
  files: FileDiagnostic[];
  limitedMode: boolean;
}

export interface HistoryItem {
  id: number;
  /** `2026-11-02_15-30-00`. */
  at: string;
  kind: 'change' | 'restore';
  function: string;
  summary: string;
  files: string[];
  backup: string;
  backupMissing: boolean;
}

export interface RewriteResult {
  backup: string;
  oldSize: number;
  newSize: number;
}

export type ReserveState = 'ready' | 'nothingToDo' | 'needsDiscovery' | 'noData';

export interface ReserveSummary {
  id: number;
  name: string;
  state: ReserveState;
  newZones: number;
  /** Zonas com horário estimado (o jogo corrige quando o jogador passa por lá). */
  estimated: number;
  /** Zonas que não puderam ser calculadas. */
  skipped: number;
  species: SpeciesNeeds[];
}

export interface ZonesPreview {
  reserves: ReserveSummary[];
  total: number;
}

export interface UnlockResult {
  zonesAdded: number;
  reserves: number;
  backup: string;
}

export interface SpeciesNeeds {
  id: string;
  name: string;
  /** Zonas a liberar de comer, beber e descansar. */
  needs: [number, number, number];
}

export interface Limits {
  minWeight: number;
  maxWeight: number;
  minScore: number;
  maxScore: number;
  typicalWeight: number;
  typicalScore: number;
}

export interface SexLimits {
  male: Limits;
  female: Limits;
}

export interface EditableAnimal {
  index: number;
  male: boolean;
  weight: number;
  score: number;
  greatOne: boolean;
}

export interface EditableGroup {
  number: number;
  animals: EditableAnimal[];
}

export interface MedalBands {
  bronze: number;
  silver: number;
  gold: number;
  diamond: number;
  max: number;
}

export interface SpeciesEditor {
  reserve: number;
  reserveName: string;
  species: string;
  name: string;
  greatOneAllowed: boolean;
  limits: SexLimits;
  medals: MedalBands | null;
  groups: EditableGroup[];
}

export interface AnimalChange {
  group: number;
  index: number;
  male?: boolean;
  weight?: number;
  score?: number;
  greatOne?: boolean;
}

export interface EditRequest {
  reserve: number;
  species: string;
  changes: AnimalChange[];
}

export interface EditResult {
  animals: number;
  backup: string;
}

export interface License {
  premium: boolean;
  state: 'free' | 'active' | 'expiring' | 'expired';
  email: string | null;
  /** Vencimento, em segundos desde 1970. */
  expires: number | null;
  daysLeft: number | null;
  plan: 'monthly' | 'yearly' | null;
}

export interface PurchaseStatus {
  /** `none`: nada em andamento; `waiting`: esperando o pagamento; `activated`: Premium ligado. */
  state: 'none' | 'waiting' | 'activated';
  plan: 'monthly' | 'yearly' | null;
  license: License;
}

export interface UnlockRequest {
  reserves: number[];
  species: string[] | null;
  needs: number[] | null;
}

export interface Living {
  groups: number;
  animals: number;
  males: number;
  females: number;
  greatOnes: number;
  diamonds: number;
  diamondMin: number | null;
  bestScore: number;
  heaviest: number;
}

export interface Harvest {
  score: number;
  medal: number;
  at: number;
}

export interface SpeciesEntry {
  id: string;
  name: string;
  group: string;
  areas: number;
  living: Living | null;
  harvests: Harvest[];
}

export interface AnimalReserve {
  id: number;
  name: string;
  cover: string | null;
  played: boolean;
  species: SpeciesEntry[];
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  hidden: boolean;
  unlocked: boolean;
}

export interface Profile {
  steamId: string;
  level: number;
  xp: number;
  cash: number;
  shotsFired: number;
  shotsMissed: number;
  totalKills: number;
  oneShotKills: number;
  recordScore: number;
  furthestShot: number;
  trophies: number;
  reserves: number;
  achievements: Achievement[];
  hasAchievements: boolean;
}

export interface MissionObjective {
  done: number;
  required: number;
}

export interface Mission {
  id: number;
  text: string;
  state: 'done' | 'active' | 'locked';
  main: boolean;
  objectives: MissionObjective[];
}

export interface ReserveMissions {
  id: number;
  name: string;
  total: number;
  done: number;
  active: number;
  mainTotal: number;
  mainDone: number;
  missions: Mission[];
}

export interface MapReserve {
  id: number;
  name: string;
  /** Foto de capa do jogo (`data:image/jpeg;base64,...`), se houver. */
  cover: string | null;
}

export interface MapSpecies {
  id: string;
  name: string;
}

export interface MapZone {
  x: number;
  z: number;
  /** 1 comer, 2 beber, 3 descansar. */
  kind: number;
  /** Índice em `species`; -1 se desconhecida. */
  species: number;
  /** Ainda não está no save. */
  pending: boolean;
  start: number;
  end: number;
}

export interface MapAnimal {
  male: boolean;
  weight: number;
  score: number;
  greatOne: boolean;
}

export interface MapGroup {
  number: number;
  species: number;
  /** Índices em `zones`, na ordem da rota. */
  stops: number[];
  animals: MapAnimal[];
}

export interface ReserveMap {
  id: number;
  name: string;
  /** `data:image/jpeg;base64,...` */
  image: string | null;
  x0: number;
  z0: number;
  span: number;
  approximate: boolean;
  species: MapSpecies[];
  zones: MapZone[];
  groups: MapGroup[];
}

export type EquipmentKind = 'weapon' | 'ammo' | 'sight' | 'optic' | 'lure_caller' | 'deployable' | 'consumable' | 'vehicle';

export type WeaponClass = 'rifle' | 'handgun' | 'shotgun' | 'archery';

export interface WeaponStats {
  shotsFired: number;
  shotsMissed: number;
  /** Porcentagem de acertos; `null` sem tiros. */
  accuracy: number | null;
  kills: number;
  oneShotKills: number;
  recordScore: number;
  recordSpecies: string | null;
  furthestShot: number | null;
  /** Segundos desde 1970. */
  purchasedAt: number | null;
}

export interface EquipmentItem {
  id: string;
  kind: EquipmentKind;
  name: string;
  price: number;
  weight: number | null;
  class: WeaponClass | null;
  boxSize: number | null;
  icon: string | null;
  owned: boolean;
  unlocked: boolean;
  inBackpack: boolean;
  amount: number;
  ammo: string[];
  sights: string[];
  stats: WeaponStats | null;
}

/** Mudança pedida ao editor de equipamentos (Premium). */
export type EquipmentChange =
  | { action: 'give'; id: string; amount?: number }
  | { action: 'setAmount'; id: string; amount: number }
  | { action: 'unlock'; id: string }
  | { action: 'setCash'; cash: number };

export interface EquipmentEditResult {
  changes: number;
  backup: string;
}

export interface Loadout {
  index: number;
  name: string;
  unlocked: boolean;
  active: boolean;
  items: string[];
  missing: string[];
}

export interface Inventory {
  cash: number;
  /** Rifle, pistola, espingarda, arco. */
  weaponLevels: [number, number, number, number];
  items: EquipmentItem[];
  loadouts: Loadout[];
}

export interface RestoreResult {
  safetyBackup: string;
}

/** Erro devolvido pelos comandos: `code` é estável e traduzível. */
export interface CommandError {
  code: string;
  message: string;
}

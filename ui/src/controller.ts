import { isCommandError, type Api } from './api';
import { formatCount, formatStamp } from './format';
import { LOCALES, type Locale, errorText, getLocale, saveLocale, setLocale, t } from './i18n';
import { icon, type IconName } from './icons';
import type { AnimalReserve, Diagnostics, EditRequest, HistoryItem, License, MapReserve, Profile, PurchaseStatus, ReserveMap, ReserveMissions, ZonesPreview } from './types';
import { renderDiagnostics } from './views/diagnostics';
import { defaultRewriteFile, renderHome } from './views/home';
import { renderHistory } from './views/history';
import { bindAnimals, renderAnimals } from './views/animals';
import type { EditorHooks, SaveOutcome } from './views/animalEdit';
import { MapView } from './views/map';
import { type MissionFilter, renderMissions } from './views/missions';
import { bindProfile, renderProfile } from './views/profile';
import { renderPremium } from './views/premium';
import { renderUnlock } from './views/unlock';
import { type UnlockMode, type UnlockState, initialUnlockState, summarize, toRequest, toggled } from './unlocklogic';
import { escapeHtml } from './escape';

export type View = 'home' | 'premium' | 'profile' | 'animals' | 'map' | 'missions' | 'history' | 'diagnostics' | 'unlock';

interface Notice {
  kind: 'ok' | 'error';
  text: string;
}

/** Mensagem para qualquer erro vindo de um comando. */
export function describeError(error: unknown): string {
  if (isCommandError(error)) return errorText(error.code, error.message);
  return t('error.generic', { message: error instanceof Error ? error.message : String(error) });
}

/** De quantos em quantos milissegundos o app pergunta se o pagamento foi confirmado. */
const PURCHASE_POLL_MS = 4000;

/** Código secreto da compra: 24 letras e números aleatórios, que só este app conhece. */
function newPurchaseCode(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(18));
  return btoa(String.fromCharCode(...bytes)).replaceAll('+', '-').replaceAll('/', '_');
}

const FREE: License = { premium: false, state: 'free', email: null, expires: null, daysLeft: null, plan: null };

function parseMode(value: string | undefined): UnlockMode {
  return value === 'animal' || value === 'need' ? value : 'all';
}

function parseFilter(value: string | undefined): MissionFilter {
  return value === 'all' || value === 'done' || value === 'locked' ? value : 'active';
}

/** Liga as telas ao app: navegação, ações dos botões e mensagens. */
export class Controller {
  private view: View = 'home';
  private diagnostics: Diagnostics | null = null;
  private history: HistoryItem[] | null = null;
  private zones: ZonesPreview | null = null;
  private unlockState: UnlockState = initialUnlockState();
  private purchase: 'monthly' | 'yearly' | null = null;
  private purchaseTimer: number | null = null;
  private license: License = { premium: false, state: 'free', email: null, expires: null, daysLeft: null, plan: null };
  private mapReserves: MapReserve[] | null = null;
  private homeReserves: MapReserve[] = [];
  private profile: Profile | null = null;
  private animals: AnimalReserve[] | null = null;
  private animalReserve: number | null = null;
  private animalSpecies: string | null = null;
  private missions: ReserveMissions[] | null = null;
  private missionCovers = new Map<number, string | null>();
  private missionChoice: number | null = null;
  private missionFilter: MissionFilter = 'active';
  private mapData: ReserveMap | null = null;
  private mapChoice: number | null = null;
  private mapView: MapView | null = null;
  private notice: Notice | null = null;
  private loading = false;

  constructor(
    private readonly root: HTMLElement,
    private readonly api: Api,
  ) {
    root.addEventListener('click', (event) => {
      const target = event.target instanceof Element ? event.target.closest<HTMLElement>('[data-action]') : null;
      if (target) void this.handle(target);
    });
  }

  /** Mostra a tela inicial. */
  async start(): Promise<void> {
    await this.api.setLanguage(getLocale()).catch(() => undefined);
    this.license = await this.api.license().catch((): License => FREE);
    await this.resumePurchase();
    await this.show('home');
  }

  /** Se o app foi fechado no meio de uma compra, volta a esperar o pagamento (ou ativa, se já saiu). */
  private async resumePurchase(): Promise<void> {
    try {
      this.applyPurchase(await this.api.checkPurchase());
    } catch {
      // Sem loja ou sem internet: nada a retomar agora.
    }
  }

  /** Compra pelo app: abre o pagamento no navegador e espera a confirmação, sem copiar chave. */
  private async buy(plan: 'monthly' | 'yearly'): Promise<void> {
    if (this.purchase !== null) return;
    try {
      await this.api.startPurchase(plan, newPurchaseCode());
      this.purchase = plan;
      this.notice = { kind: 'ok', text: t('purchase.opened') };
      this.watchPurchase();
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    this.render();
    window.scrollTo({ top: 0 });
  }

  private watchPurchase(): void {
    this.stopWatching();
    this.purchaseTimer = window.setInterval(() => {
      void this.pollPurchase();
    }, PURCHASE_POLL_MS);
  }

  private stopWatching(): void {
    if (this.purchaseTimer !== null) window.clearInterval(this.purchaseTimer);
    this.purchaseTimer = null;
  }

  private async pollPurchase(): Promise<void> {
    try {
      this.applyPurchase(await this.api.checkPurchase());
      this.render();
    } catch {
      // Sem internet agora: continua esperando e tenta de novo.
    }
  }

  private applyPurchase(status: PurchaseStatus): void {
    if (status.state === 'waiting') {
      this.purchase = status.plan;
      if (this.purchaseTimer === null) this.watchPurchase();
      return;
    }
    this.purchase = null;
    this.stopWatching();
    if (status.state === 'activated') {
      this.license = status.license;
      this.notice = { kind: 'ok', text: t('purchase.done') };
    }
  }

  private async cancelPurchase(): Promise<void> {
    await this.api.cancelPurchase().catch(() => undefined);
    this.purchase = null;
    this.stopWatching();
    this.notice = null;
    this.render();
  }

  async show(view: View): Promise<void> {
    this.view = view;
    this.notice = null;
    if (view === 'unlock') {
      this.zones = null;
      this.unlockState = initialUnlockState();
    }
    await this.reload();
  }

  private async reload(): Promise<void> {
    this.loading = true;
    this.render();
    try {
      if (this.view === 'history') this.history = await this.api.history();
      else if (this.view === 'unlock') this.zones = await this.api.zonesPreview();
      else if (this.view === 'map') await this.loadMap();
      else if (this.view === 'premium') this.license = await this.api.license().catch((): License => FREE);
      else if (this.view === 'profile') this.profile = await this.api.profile();
      else if (this.view === 'animals') this.animals = await this.api.animals();
      else if (this.view === 'missions') await this.loadMissions();
      else {
        this.diagnostics = await this.api.diagnostics();
        this.homeReserves = this.view === 'home' ? await this.reservesOnHome(this.diagnostics) : [];
      }
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    this.loading = false;
    this.render();
  }

  /** Reservas reconhecidas no jogo, para a galeria da tela inicial; sem jogo ou sem save, nenhuma. */
  private async reservesOnHome(diagnostics: Diagnostics): Promise<MapReserve[]> {
    if (!diagnostics.game.found || diagnostics.save.status !== 'found') return [];
    try {
      return await this.api.mapReserves();
    } catch {
      return [];
    }
  }

  private async handle(button: HTMLElement): Promise<void> {
    const { action, path, backup, at, reserve, plan } = button.dataset;
    switch (action) {
      case 'show-home':
        await this.show('home');
        break;
      case 'show-profile':
        await this.show('profile');
        break;
      case 'show-animals':
        await this.show('animals');
        break;
      case 'pick-animal-reserve':
        this.animalReserve = Number(reserve);
        this.animalSpecies = null;
        this.render();
        break;
      case 'pick-species':
        this.animalSpecies = button.dataset['species'] || null;
        this.render();
        break;
      case 'open-animal':
        this.animalReserve = Number(reserve);
        this.animalSpecies = button.dataset['species'] ?? null;
        this.render();
        break;
      case 'show-missions':
        await this.show('missions');
        break;
      case 'pick-mission-reserve':
        this.missionChoice = Number(reserve);
        this.render();
        break;
      case 'mission-filter':
        this.missionFilter = parseFilter(button.dataset['filter']);
        this.render();
        break;
      case 'show-map':
        await this.show('map');
        break;
      case 'open-reserve':
        this.mapChoice = Number(reserve);
        await this.show('map');
        break;
      case 'show-unlock':
        await this.show('unlock');
        break;
      case 'unlock-run':
        await this.unlock();
        break;
      case 'activate-license':
        await this.activateLicense();
        break;
      case 'remove-license':
        await this.removeLicense();
        break;
      case 'set-language':
        await this.changeLanguage(button.dataset['locale']);
        break;
      case 'show-premium':
      case 'premium-info':
        await this.show('premium');
        break;
      case 'buy':
        await this.buy(plan === 'monthly' ? 'monthly' : 'yearly');
        break;
      case 'cancel-purchase':
        await this.cancelPurchase();
        break;
      case 'toggle-reserve': {
        const ready = (this.zones?.reserves ?? []).filter((r) => r.state === 'ready').map((r) => r.id);
        const current = this.unlockState.reserves ?? new Set(ready);
        this.unlockState = { ...this.unlockState, reserves: toggled(current, Number(reserve)) };
        this.render();
        break;
      }
      case 'unlock-mode':
        this.unlockState = { ...this.unlockState, mode: parseMode(button.dataset['mode']) };
        this.render();
        break;
      case 'toggle-species':
        this.unlockState = { ...this.unlockState, species: toggled(this.unlockState.species, button.dataset['species'] ?? '') };
        this.render();
        break;
      case 'toggle-need':
        this.unlockState = { ...this.unlockState, needs: toggled(this.unlockState.needs, Number(button.dataset['need'])) };
        this.render();
        break;
      case 'show-diagnostics':
        await this.show('diagnostics');
        break;
      case 'show-history':
        await this.show('history');
        break;
      case 'refresh':
        await this.reload();
        break;
      case 'use-account':
        if (path) await this.run(() => this.api.selectSaveDir(path));
        break;
      case 'choose-save-folder':
        await this.chooseFolder(t('save.choose'), (chosen) => this.api.selectSaveDir(chosen));
        break;
      case 'choose-game-folder':
        await this.chooseFolder(t('game.choose'), (chosen) => this.api.selectGameDir(chosen));
        break;
      case 'reset-choices':
        await this.run(() => this.api.resetChoices());
        break;
      case 'restore':
        if (backup) await this.restore(backup, at ?? '');
        break;
      case 'rewrite-unchanged':
        await this.rewriteUnchanged();
        break;
      default:
        break;
    }
  }

  private async chooseFolder(title: string, apply: (path: string) => Promise<void>): Promise<void> {
    const chosen = await this.api.pickFolder(title);
    if (chosen) await this.run(() => apply(chosen));
  }

  /** Executa uma ação que muda configurações e recarrega a tela; erros viram aviso. */
  private async run(action: () => Promise<unknown>): Promise<void> {
    try {
      await action();
      this.notice = null;
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  private async rewriteUnchanged(): Promise<void> {
    const file = defaultRewriteFile(this.diagnostics?.files ?? []);
    if (!file) return;
    if (!(await this.api.confirm(t('tools.confirm'), t('tools.confirmOk')))) return;
    try {
      await this.api.rewriteUnchanged(file);
      this.notice = { kind: 'ok', text: t('tools.done') };
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  /** Carrega a lista de reservas e os dados da escolhida (a anterior, se ainda existir). */
  private async loadMap(): Promise<void> {
    this.mapReserves = await this.api.mapReserves();
    const keep = this.mapReserves.find((r) => r.id === this.mapChoice);
    this.mapChoice = keep?.id ?? this.mapReserves[0]?.id ?? null;
    this.mapData = this.mapChoice === null ? null : await this.api.reserveMap(this.mapChoice);
  }

  /** Missões de todas as reservas e as capas delas (sem capa, o cartão mostra um ícone). */
  private async loadMissions(): Promise<void> {
    this.missions = await this.api.missions();
    const covers = await this.api.mapReserves().catch((): MapReserve[] => []);
    this.missionCovers = new Map(covers.map((r) => [r.id, r.cover]));
    if (!this.missions.some((r) => r.id === this.missionChoice)) this.missionChoice = this.missions[0]?.id ?? null;
  }

  /** O que o painel de edição do mapa usa do app. */
  private editorHooks(): EditorHooks {
    return {
      premium: this.license.premium,
      load: (reserve, species) => this.api.speciesEditor(reserve, species),
      save: (request) => this.saveAnimals(request),
      openPremium: () => {
        void this.show('premium');
      },
      reload: () => {
        void this.reload();
      },
    };
  }

  private async saveAnimals(request: EditRequest): Promise<SaveOutcome> {
    if (!(await this.api.confirm(t('edit.confirm'), t('edit.confirmOk')))) return { status: 'cancelled' };
    try {
      await this.api.editAnimals(request);
      this.notice = { kind: 'ok', text: t('edit.done') };
      return { status: 'saved' };
    } catch (error) {
      return { status: 'error', message: describeError(error) };
    }
  }

  private async chooseMapReserve(id: number): Promise<void> {
    this.mapChoice = id;
    this.notice = null;
    await this.reload();
  }

  /** Troca o idioma: guarda a escolha, avisa o app e refaz a tela com os textos novos. */
  private async changeLanguage(code: string | undefined): Promise<void> {
    const locale: Locale | undefined = LOCALES.find((l) => l.code === code)?.code;
    if (locale === undefined || locale === getLocale()) return;
    setLocale(locale);
    saveLocale(locale);
    await this.api.setLanguage(locale).catch(() => undefined);
    // Nomes de animais, missões e conquistas vêm do app no idioma antigo: refaz todos.
    this.animals = null;
    this.missions = null;
    this.profile = null;
    this.mapData = null;
    this.mapReserves = null;
    this.zones = null;
    this.notice = null;
    await this.reload();
  }

  /** Lê a chave colada em Ajustes e a ativa; o app confere tudo sem internet. */
  private async activateLicense(): Promise<void> {
    const field = this.root.querySelector<HTMLTextAreaElement>('#license-key');
    const key = field?.value.trim() ?? '';
    if (key === '') {
      this.notice = { kind: 'error', text: t('license.empty') };
      this.render();
      return;
    }
    try {
      this.license = await this.api.activateLicense(key);
      this.notice = { kind: 'ok', text: t('license.done') };
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  private async removeLicense(): Promise<void> {
    if (!(await this.api.confirm(t('license.removeConfirm'), t('license.removeOk')))) return;
    try {
      this.license = await this.api.removeLicense();
      this.notice = { kind: 'ok', text: t('license.removed') };
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  private async unlock(): Promise<void> {
    if (!this.zones) return;
    if (!this.license.premium) {
      await this.show('premium');
      return;
    }
    const totals = summarize(this.zones, this.unlockState);
    if (totals.total === 0) return;
    const params = { count: formatCount(totals.total), reserves: totals.reserves };
    if (!(await this.api.confirm(t('unlock.confirm', params), t('unlock.confirmOk')))) return;
    try {
      await this.api.unlockZones(toRequest(this.zones, this.unlockState));
      this.notice = { kind: 'ok', text: t('unlock.done', params) };
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  private async restore(backup: string, at: string): Promise<void> {
    if (!(await this.api.confirm(t('history.restoreConfirm', { at: formatStamp(at) }), t('history.restoreOk')))) return;
    try {
      await this.api.restore(backup);
      this.notice = { kind: 'ok', text: t('history.restored') };
    } catch (error) {
      this.notice = { kind: 'error', text: describeError(error) };
    }
    await this.reload();
  }

  private render(): void {
    this.mapView?.dispose();
    this.mapView = null;
    const waiting = this.loading && !this.currentData() && this.view !== 'unlock' && this.view !== 'premium';
    const body = waiting
      ? `<p class="muted">${escapeHtml(t(this.view === 'map' ? 'map.loading' : this.view === 'missions' ? 'missions.loading' : this.view === 'animals' ? 'animals.loading' : 'common.loading'))}</p>`
      : this.renderBody();
    const notice = this.notice
      ? `<p class="notice notice-${this.notice.kind}" role="alert">${escapeHtml(this.notice.text)}</p>`
      : '';
    const item = (view: View, symbol: IconName, label: string, action: string, premium = false): string =>
      `<button class="tab nav-item"${premium && !this.license.premium ? ' data-premium="true"' : ''} data-action="${action}" title="${escapeHtml(label)}" aria-current="${this.view === view ? 'page' : 'false'}">${icon(symbol, 20)}<span>${escapeHtml(label)}</span></button>`;
    this.root.innerHTML = `<div class="shell">
      <nav class="sidebar" aria-label="Telas">
        <div class="brand"><span class="brand-mark" aria-hidden="true">${icon('paw', 20)}</span><span class="brand-name">${escapeHtml(t('app.name'))}</span></div>
        ${item('home', 'home', t('nav.home'), 'show-home')}
        ${item('profile', 'user', t('nav.profile'), 'show-profile')}
        ${item('animals', 'paw', t('nav.animals'), 'show-animals')}
        ${item('map', 'map', t('nav.map'), 'show-map')}
        ${item('missions', 'flag', t('nav.missions'), 'show-missions')}
        ${item('unlock', 'unlock', t('nav.unlock'), 'show-unlock', true)}
        ${item('history', 'backup', t('nav.history'), 'show-history')}
        <span class="nav-spacer"></span>
        ${item('premium', 'star', t('nav.premium'), 'show-premium')}
        ${item('diagnostics', 'settings', t('nav.diagnostics'), 'show-diagnostics')}
      </nav>
      <main class="${this.view === 'map' ? 'wide' : ''}" aria-busy="${String(this.loading)}">${notice}${body}</main>
    </div>`;
    this.mountMap();
    bindProfile(this.root);
    bindAnimals(this.root);
  }

  /** Monta o mapa interativo depois que o HTML da tela existe. */
  private mountMap(): void {
    const host = this.root.querySelector<HTMLElement>('#map-host');
    if (!host || !this.mapData || !this.mapReserves) return;
    this.mapView = new MapView(
      host,
      this.mapReserves,
      this.mapData,
      (id) => {
        void this.chooseMapReserve(id);
      },
      this.editorHooks(),
    );
    this.mapView.mount();
  }

  private currentData(): Diagnostics | HistoryItem[] | ZonesPreview | ReserveMap | MapReserve[] | Profile | ReserveMissions[] | AnimalReserve[] | null {
    if (this.view === 'unlock') return this.zones;
    if (this.view === 'map') return this.mapData ?? this.mapReserves;
    if (this.view === 'profile') return this.profile;
    if (this.view === 'animals') return this.animals;
    if (this.view === 'missions') return this.missions;
    return this.view === 'history' ? this.history : this.diagnostics;
  }

  private renderMap(): string {
    if (this.mapReserves?.length === 0) return `<p class="muted">${escapeHtml(t('map.empty'))}</p>`;
    return this.mapData ? '<div id="map-host"></div>' : '';
  }

  private renderBody(): string {
    if (this.view === 'history') return this.history ? renderHistory(this.history) : '';
    if (this.view === 'unlock') return renderUnlock(this.zones, this.unlockState, this.license.premium);
    if (this.view === 'map') return this.renderMap();
    if (this.view === 'profile') return this.profile ? renderProfile(this.profile) : '';
    if (this.view === 'premium') return renderPremium(this.license, this.purchase);
    if (this.view === 'animals') {
      return this.animals ? renderAnimals(this.animals, this.animalReserve, this.animalSpecies) : '';
    }
    if (this.view === 'missions') {
      return this.missions ? renderMissions(this.missions, this.missionCovers, this.missionChoice, this.missionFilter) : '';
    }
    if (!this.diagnostics) return '';
    return this.view === 'home'
      ? renderHome(this.diagnostics, this.homeReserves, this.license)
      : renderDiagnostics(this.diagnostics);
  }
}

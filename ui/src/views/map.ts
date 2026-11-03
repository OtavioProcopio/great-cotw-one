import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon, iconPath, type IconName } from '../icons';
import {
  type Filters,
  type Stop,
  type View,
  NEEDS,
  boundsOf,
  centerOn,
  countBySpecies,
  fitView,
  formatHour,
  groupsOfSpecies,
  screenToWorld,
  slotText,
  stopsNow,
  stopsOf,
  visibleZones,
  worldToScreen,
  zoomAt,
} from '../maplogic';
import type { MapGroup, MapReserve, MapZone, ReserveMap } from '../types';

const COLORS: Record<number, string> = { 1: '#bcd84e', 2: '#4fbcd1', 3: '#f4e856' };
const NEED_ICONS: Record<number, IconName> = { 1: 'leaf', 2: 'drop', 3: 'moon' };
const FALLBACK_GLYPHS: Record<number, string> = { 1: '🌿', 2: '💧', 3: '🌙' };
const ACCENT = '#ee8a4c';
const BACKGROUND = '#050705';
const GRID_LINE = 'rgba(236,240,230,0.10)';
const GRID_TEXT = 'rgba(236,240,230,0.62)';
const FALLBACK_SIZE = { width: 800, height: 600 };

function needLabel(kind: number): string {
  return kind === 1 || kind === 2 || kind === 3 ? t(`map.need.${String(kind) as '1' | '2' | '3'}`) : '';
}

function reserveStrip(reserves: MapReserve[], current: number): string {
  return reserves
    .map((r) => {
      const cover = r.cover
        ? `<img src="${e(r.cover)}" alt="" loading="lazy" />`
        : `<span class="cover-empty" aria-hidden="true">${icon('map', 22)}</span>`;
      return `<button class="reserve-card" data-reserve="${String(r.id)}" aria-pressed="${String(r.id === current)}">
        <span class="cover">${cover}</span><span class="reserve-name">${e(r.name)}</span></button>`;
    })
    .join('');
}

function shell(reserves: MapReserve[], map: ReserveMap): string {
  const chips = NEEDS.map(
    (kind) =>
      `<button class="chip" data-need="${String(kind)}" aria-pressed="true" style="--tone:${COLORS[kind] ?? ''}">${icon(NEED_ICONS[kind] ?? 'leaf', 16)}<span>${e(needLabel(kind))}</span></button>`,
  ).join('');
  const unlock = map.zones.some((z) => z.pending)
    ? `<button class="primary wide-button" data-action="show-unlock">${icon('unlock', 16)}<span>${e(t('map.unlock'))}</span></button>`
    : '';
  const noImage = map.image ? '' : `<p class="notice notice-warn">${e(t('map.noImage'))}</p>`;
  const approx = map.image && map.approximate ? `<p class="notice notice-warn">${e(t('map.approx'))}</p>` : '';
  return `<div class="map-app">
    <aside class="map-side">
      <header class="map-title"><h1>${e(t('map.title'))}</h1><p>${e(t('map.subtitle'))}</p></header>
      <section class="group">
        <h2 class="lbl">${e(t('map.reserves'))}</h2>
        <div class="reserve-strip" id="map-reserve" role="group" aria-label="${e(t('map.reserve'))}">${reserveStrip(reserves, map.id)}</div>
      </section>
      <section class="group">
        <h2 class="lbl">${e(t('map.needs'))}</h2>
        <div class="chips">${chips}</div>
        <label class="switch"><input type="checkbox" id="map-pending" checked /><span class="track" aria-hidden="true"></span><span class="switch-text">${e(t('map.pending'))}</span></label>
        <div class="switch-row">
          <label class="switch"><input type="checkbox" id="map-byhour" /><span class="track" aria-hidden="true"></span><span class="switch-text">${e(t('map.byHour'))}</span></label>
          <span class="hour" id="map-hour-text">06:00</span>
        </div>
        <input type="range" id="map-hour" min="0" max="23.5" step="0.5" value="6" aria-label="${e(t('map.hour'))}" />
        ${unlock}
        ${noImage}${approx}
      </section>
      <section class="group trk">
        <h2 class="lbl">${e(t('map.follow'))}</h2>
        <p class="legend">${e(t('map.followHint'))}</p>
        <div class="two">
          <select id="map-gsp" aria-label="${e(t('map.speciesLabel'))}"></select>
          <select id="map-grp" aria-label="${e(t('map.groupLabel'))}"></select>
        </div>
        <div class="gsum" id="map-gsum"></div>
        <div class="anim" id="map-anim"></div>
        <ol class="route" id="map-route"></ol>
        <p class="legend" id="map-route-hint" hidden>${e(t('map.routeHint'))}</p>
      </section>
      <section class="group">
        <div class="row"><h2 class="lbl">${e(t('map.speciesTitle'))}</h2>
          <span class="mini"><button id="map-all">${e(t('map.all'))}</button><button id="map-none">${e(t('map.none'))}</button></span></div>
        <div class="splist" id="map-species"></div>
      </section>
      <p class="credit">${e(t('map.credit'))}</p>
    </aside>
    <section class="map-main">
      <canvas id="map-canvas"></canvas>
      <div class="map-stats"><strong id="map-count">0</strong><span>${e(t('map.count'))}</span></div>
      <div class="map-hud"><span>X</span><b id="map-x">—</b><span>Z</span><b id="map-z">—</b><span>${e(t('map.hud'))}</span></div>
      <div class="map-zoom">
        <button id="map-zin" aria-label="${e(t('map.zoomIn'))}" title="${e(t('map.zoomIn'))}">${icon('plus', 18)}</button>
        <button id="map-zout" aria-label="${e(t('map.zoomOut'))}" title="${e(t('map.zoomOut'))}">${icon('minus', 18)}</button>
        <button id="map-zfit" aria-label="${e(t('map.zoomFit'))}" title="${e(t('map.zoomFit'))}">${icon('fit', 18)}</button>
      </div>
      <div class="map-tip" id="map-tip" hidden></div>
    </section>
  </div>`;
}

/** Mapa interativo de uma reserva: filtros, rastreador de grupo e canvas com zoom e arraste. */
export class MapView {
  private filters: Filters;
  private view: View = { scale: 0.05, x: 0, y: 0 };
  private visible: MapZone[] = [];
  private hover: MapZone | null = null;
  private pinned: MapZone | null = null;
  private group: { data: MapGroup; stops: Stop[]; species: number } | null = null;
  private hour = 6;
  private readonly iconShapes = new Map<number, Path2D>();
  private readonly image: HTMLImageElement | null;
  private canvas!: HTMLCanvasElement;
  private tip!: HTMLElement;
  private drag: { x: number; y: number; vx: number; vy: number; moved: boolean } | null = null;
  private readonly onResize = (): void => {
    this.draw();
  };

  constructor(
    private readonly root: HTMLElement,
    private readonly reserves: MapReserve[],
    private readonly map: ReserveMap,
    private readonly onReserve: (id: number) => void,
  ) {
    this.filters = {
      needs: new Set(NEEDS),
      species: new Set(map.species.map((_, i) => i)),
      showPending: true,
      hour: null,
    };
    this.image = map.image ? new Image() : null;
  }

  mount(): void {
    this.root.innerHTML = shell(this.reserves, this.map);
    const canvas = this.q('#map-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) throw new Error('#map-canvas não é um canvas');
    this.canvas = canvas;
    this.tip = this.q('#map-tip');
    if (this.image && this.map.image) {
      this.image.onload = () => {
        this.draw();
      };
      this.image.src = this.map.image;
    }
    this.bind();
    this.fillTrackerSpecies();
    this.fit();
    this.refresh();
    window.addEventListener('resize', this.onResize);
  }

  dispose(): void {
    window.removeEventListener('resize', this.onResize);
  }

  private q(selector: string): HTMLElement {
    const found = this.root.querySelector<HTMLElement>(selector);
    if (!found) throw new Error(`${selector} ausente`);
    return found;
  }

  private select(selector: string): HTMLSelectElement {
    const found = this.q(selector);
    if (!(found instanceof HTMLSelectElement)) throw new Error(`${selector} não é um seletor`);
    return found;
  }

  private input(selector: string): HTMLInputElement {
    const found = this.q(selector);
    if (!(found instanceof HTMLInputElement)) throw new Error(`${selector} não é um campo`);
    return found;
  }

  private size(): { width: number; height: number } {
    const box = this.canvas.getBoundingClientRect();
    return box.width > 0 && box.height > 0 ? { width: box.width, height: box.height } : FALLBACK_SIZE;
  }

  private fit(): void {
    const { width, height } = this.size();
    this.view = fitView(boundsOf(this.map), width, height);
  }

  // ---------- filtros ----------

  private bind(): void {
    for (const card of this.root.querySelectorAll<HTMLButtonElement>('#map-reserve [data-reserve]')) {
      card.addEventListener('click', () => {
        const id = Number(card.dataset['reserve']);
        if (id !== this.map.id) this.onReserve(id);
      });
    }
    for (const chip of this.root.querySelectorAll<HTMLButtonElement>('[data-need]')) {
      chip.addEventListener('click', () => {
        const kind = Number(chip.dataset['need']);
        const needs = new Set(this.filters.needs);
        if (needs.has(kind)) needs.delete(kind);
        else needs.add(kind);
        chip.setAttribute('aria-pressed', String(needs.has(kind)));
        this.filters = { ...this.filters, needs };
        this.refresh();
      });
    }
    this.input('#map-pending').addEventListener('change', (event) => {
      this.filters = { ...this.filters, showPending: (event.target as HTMLInputElement).checked };
      this.refresh();
    });
    const byHour = this.input('#map-byhour');
    const slider = this.input('#map-hour');
    const onHour = (): void => {
      this.hour = Number(slider.value);
      this.q('#map-hour-text').textContent = formatHour(this.hour);
      this.filters = { ...this.filters, hour: byHour.checked ? this.hour : null };
      this.markNow();
      this.refresh();
    };
    byHour.addEventListener('change', onHour);
    slider.addEventListener('input', onHour);
    this.q('#map-all').addEventListener('click', () => {
      this.setSpecies(new Set(this.map.species.map((_, i) => i)));
    });
    this.q('#map-none').addEventListener('click', () => {
      this.setSpecies(new Set());
    });
    this.select('#map-gsp').addEventListener('change', () => {
      this.fillGroups();
      this.onlySpecies(Number(this.select('#map-gsp').value));
    });
    this.select('#map-grp').addEventListener('change', () => {
      this.pickGroup();
    });
    this.q('#map-zin').addEventListener('click', () => {
      this.zoom(1.6);
    });
    this.q('#map-zout').addEventListener('click', () => {
      this.zoom(1 / 1.6);
    });
    this.q('#map-zfit').addEventListener('click', () => {
      this.fit();
      this.draw();
    });
    this.bindCanvas();
  }

  private setSpecies(species: Set<number>): void {
    this.filters = { ...this.filters, species };
    this.refresh();
  }

  private onlySpecies(index: number): void {
    if (index >= 0) this.setSpecies(new Set([index]));
  }

  /** Recalcula o que aparece e atualiza contagens e desenho. */
  private refresh(): void {
    this.visible = visibleZones(this.map.zones, this.filters);
    this.q('#map-count').textContent = formatCount(this.visible.length);
    this.renderSpecies();
    this.pinned = null;
    this.hover = null;
    this.tip.hidden = true;
    this.draw();
  }

  private renderSpecies(): void {
    const counts = countBySpecies(this.map, this.filters);
    const list = this.q('#map-species');
    list.innerHTML = this.map.species
      .map((species, i) => {
        const on = this.filters.species.has(i);
        return `<button class="sp${on ? '' : ' off'}" data-species="${String(i)}" aria-pressed="${String(on)}">
          <span class="sp-name">${e(species.name)}</span><em>${formatCount(counts.get(i) ?? 0)}</em></button>`;
      })
      .join('');
    for (const button of list.querySelectorAll<HTMLButtonElement>('[data-species]')) {
      button.addEventListener('click', () => {
        const index = Number(button.dataset['species']);
        const species = new Set(this.filters.species);
        if (species.has(index)) species.delete(index);
        else species.add(index);
        this.setSpecies(species);
      });
    }
  }

  // ---------- rastreador de grupo ----------

  private fillTrackerSpecies(): void {
    const withGroups = this.map.species
      .map((species, index) => ({ species, index }))
      .filter(({ index }) => this.map.groups.some((g) => g.species === index));
    this.q('#map-gsp').innerHTML = withGroups
      .map(({ species, index }) => `<option value="${String(index)}">${e(species.name)}</option>`)
      .join('');
    this.fillGroups();
  }

  private fillGroups(): void {
    const species = Number(this.select('#map-gsp').value);
    const groups = Number.isNaN(species) ? [] : groupsOfSpecies(this.map, species);
    const none = `<option value="">${e(t('map.groupNone'))}</option>`;
    this.q('#map-grp').innerHTML =
      none +
      groups
        .map((g) => {
          const marks = g.animals.some((a) => a.greatOne) ? '⭐ ' : '';
          return `<option value="${String(g.number)}">${e(
            t('map.groupOption', { marks, n: g.number, count: g.animals.length, stops: g.stops.length }),
          )}</option>`;
        })
        .join('');
    this.pickGroup();
  }

  private pickGroup(): void {
    const route = this.q('#map-route');
    route.innerHTML = '';
    this.q('#map-gsum').textContent = '';
    this.q('#map-anim').innerHTML = '';
    this.q('#map-route-hint').hidden = true;
    this.group = null;
    const number = Number(this.select('#map-grp').value);
    const species = Number(this.select('#map-gsp').value);
    const data = this.map.groups.find((g) => g.number === number && g.species === species);
    if (data) {
      const stops = stopsOf(data, this.map.zones);
      this.group = { data, stops, species };
      const males = data.animals.filter((a) => a.male).length;
      this.q('#map-gsum').textContent = t('map.groupSummary', {
        count: data.animals.length,
        males,
        females: data.animals.length - males,
        stops: stops.length,
      });
      this.q('#map-anim').innerHTML = data.animals
        .map((a) => {
          const sex = a.male ? '♂' : '♀';
          const label = `${sex} ${a.weight.toFixed(0)} kg${a.male && a.score > 0 ? ` · ${a.score.toFixed(0)}` : ''}${a.greatOne ? ' ⭐' : ''}`;
          return `<span class="${a.greatOne ? 'go' : ''}" title="${e(a.male ? t('map.male') : t('map.female'))}">${e(label)}</span>`;
        })
        .join('');
      route.innerHTML = stops.map((s) => this.routeItem(s)).join('');
      for (const item of route.querySelectorAll<HTMLLIElement>('li[data-stop]')) {
        item.addEventListener('click', () => {
          const stop = stops.find((s) => s.zoneIndex === Number(item.dataset['stop']));
          if (stop) this.focusStop(stop);
        });
      }
      this.q('#map-route-hint').hidden = false;
      this.fitGroup(stops);
      this.markNow();
    }
    this.draw();
  }

  private routeItem(stop: Stop): string {
    const { zone } = stop;
    const timeText = zone.end === zone.start && zone.start === 0 ? t('map.stopNoTime') : slotText(zone.start, zone.end);
    const note = zone.pending ? t('map.stopPending') : t('map.stopSaved');
    return `<li data-stop="${String(stop.zoneIndex)}" tabindex="0"><span class="n">${String(stop.n)}</span>
      <span class="lab"><span>${e(needLabel(zone.kind))}</span><span class="h${zone.pending ? ' e' : ''}">${e(timeText)} · ${e(note)}</span></span></li>`;
  }

  private markNow(): void {
    const now = this.group ? stopsNow(this.group.stops, this.hour) : new Set<Stop>();
    for (const item of this.root.querySelectorAll<HTMLLIElement>('li[data-stop]')) {
      const stop = this.group?.stops.find((s) => s.zoneIndex === Number(item.dataset['stop']));
      item.classList.toggle('now', stop !== undefined && now.has(stop));
    }
    this.draw();
  }

  private fitGroup(stops: Stop[]): void {
    if (stops.length === 0) return;
    const { width, height } = this.size();
    const xs = stops.map((s) => s.zone.x);
    const zs = stops.map((s) => s.zone.z);
    this.view = fitView(
      { minX: Math.min(...xs), maxX: Math.max(...xs), minZ: Math.min(...zs), maxZ: Math.max(...zs) },
      width,
      height,
      300,
    );
    this.view.scale = Math.min(this.view.scale, 1.2);
  }

  private focusStop(stop: Stop): void {
    const { width, height } = this.size();
    this.view = centerOn(this.view, stop.zone.x, stop.zone.z, width, height);
    this.pinned = stop.zone;
    this.hover = stop.zone;
    this.draw();
    const [px, py] = worldToScreen(this.view, stop.zone.x, stop.zone.z);
    this.showTip(stop.zone, px, py);
  }

  // ---------- canvas ----------

  private zoom(factor: number): void {
    const { width, height } = this.size();
    this.view = zoomAt(this.view, factor, width / 2, height / 2);
    this.draw();
  }

  private nearest(px: number, py: number): MapZone | null {
    const limit = (this.group ? 26 : 18) ** 2;
    let best: MapZone | null = null;
    let bestDistance = limit;
    const candidates = this.group ? this.group.stops.map((s) => s.zone) : this.visible;
    for (const zone of candidates) {
      const [x, y] = worldToScreen(this.view, zone.x, zone.z);
      const distance = (x - px) ** 2 + (y - py) ** 2;
      if (distance < bestDistance) {
        bestDistance = distance;
        best = zone;
      }
    }
    return best;
  }

  private bindCanvas(): void {
    const canvas = this.canvas;
    canvas.addEventListener('pointerdown', (event) => {
      this.drag = { x: event.offsetX, y: event.offsetY, vx: this.view.x, vy: this.view.y, moved: false };
    });
    canvas.addEventListener('pointermove', (event) => {
      const [wx, wz] = screenToWorld(this.view, event.offsetX, event.offsetY);
      this.q('#map-x').textContent = String(Math.round(wx));
      this.q('#map-z').textContent = String(Math.round(wz));
      if (this.drag) {
        const dx = event.offsetX - this.drag.x;
        const dy = event.offsetY - this.drag.y;
        if (Math.abs(dx) + Math.abs(dy) > 3) this.drag.moved = true;
        this.view = { ...this.view, x: this.drag.vx + dx, y: this.drag.vy + dy };
        this.draw();
        return;
      }
      if (!this.pinned) {
        const zone = this.nearest(event.offsetX, event.offsetY);
        if (zone !== this.hover) {
          this.hover = zone;
          this.draw();
        }
        if (zone) this.showTip(zone, event.offsetX, event.offsetY);
        else this.tip.hidden = true;
      }
    });
    const finish = (event: PointerEvent): void => {
      const drag = this.drag;
      this.drag = null;
      if (drag && !drag.moved) {
        const zone = this.nearest(event.offsetX, event.offsetY);
        this.pinned = zone;
        this.hover = zone;
        if (zone) this.showTip(zone, event.offsetX, event.offsetY);
        else this.tip.hidden = true;
        this.draw();
      }
    };
    canvas.addEventListener('pointerup', finish);
    canvas.addEventListener('pointerleave', () => {
      this.drag = null;
      if (!this.pinned) {
        this.hover = null;
        this.tip.hidden = true;
        this.draw();
      }
    });
    canvas.addEventListener(
      'wheel',
      (event) => {
        event.preventDefault();
        this.view = zoomAt(this.view, Math.exp(-event.deltaY * 0.0015), event.offsetX, event.offsetY);
        this.draw();
      },
      { passive: false },
    );
  }

  private showTip(zone: MapZone, px: number, py: number): void {
    const species = zone.species >= 0 ? (this.map.species[zone.species]?.name ?? '') : '';
    const status = zone.pending ? t('map.tipPending') : t('map.tipSaved');
    const approx = zone.pending ? ` (${t('map.tipApprox')})` : '';
    this.tip.innerHTML = `<h3>${e(species)}</h3>
      <div class="t">${icon(NEED_ICONS[zone.kind] ?? 'leaf', 14)} ${e(needLabel(zone.kind))} · ${e(status)}</div>
      <div><span class="k">${e(t('map.tipTime'))}:</span> ${e(slotText(zone.start, zone.end))}${e(approx)}</div>
      <div class="m">X ${String(Math.round(zone.x))} · Z ${String(Math.round(zone.z))}</div>`;
    this.tip.hidden = false;
    const { width, height } = this.size();
    this.tip.style.left = `${String(Math.min(width - 270, Math.max(8, px + 14)))}px`;
    this.tip.style.top = `${String(Math.min(height - 130, Math.max(8, py + 14)))}px`;
  }

  private draw(): void {
    const ctx = this.canvas.getContext('2d');
    if (!ctx) return;
    const { width, height } = this.size();
    const ratio = window.devicePixelRatio || 1;
    this.canvas.width = Math.round(width * ratio);
    this.canvas.height = Math.round(height * ratio);
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, width, height);
    this.drawImage(ctx);
    this.drawGrid(ctx, width, height);
    this.drawZones(ctx, width, height);
    this.drawRoute(ctx);
    const focus = this.pinned ?? this.hover;
    if (focus) {
      const [x, y] = worldToScreen(this.view, focus.x, focus.z);
      ctx.beginPath();
      ctx.arc(x, y, 16, 0, Math.PI * 2);
      ctx.strokeStyle = ACCENT;
      ctx.lineWidth = 2.5;
      ctx.stroke();
    }
  }

  private drawImage(ctx: CanvasRenderingContext2D): void {
    if (!this.image?.complete || this.image.naturalWidth === 0) return;
    const [x, y] = worldToScreen(this.view, this.map.x0, this.map.z0);
    const side = this.map.span * this.view.scale;
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(this.image, x, y, side, side);
  }

  private drawGrid(ctx: CanvasRenderingContext2D, width: number, height: number): void {
    const step = 1000;
    ctx.strokeStyle = GRID_LINE;
    ctx.fillStyle = GRID_TEXT;
    ctx.lineWidth = 1;
    ctx.font = '11px ui-monospace, "Cascadia Mono", monospace';
    ctx.setLineDash([2, 4]);
    const [wx0, wz0] = screenToWorld(this.view, 0, 0);
    const [wx1, wz1] = screenToWorld(this.view, width, height);
    const labels = this.view.scale * step > 48;
    for (let x = Math.ceil(wx0 / step) * step; x <= wx1; x += step) {
      const [sx] = worldToScreen(this.view, x, 0);
      ctx.beginPath();
      ctx.moveTo(sx, 0);
      ctx.lineTo(sx, height);
      ctx.stroke();
      if (labels) ctx.fillText(String(x), sx + 3, 12);
    }
    for (let z = Math.ceil(wz0 / step) * step; z <= wz1; z += step) {
      const [, sy] = worldToScreen(this.view, 0, z);
      ctx.beginPath();
      ctx.moveTo(0, sy);
      ctx.lineTo(width, sy);
      ctx.stroke();
      if (labels) ctx.fillText(String(z), 3, sy - 3);
    }
    ctx.setLineDash([]);
  }

  private drawZones(ctx: CanvasRenderingContext2D, width: number, height: number): void {
    const big = this.view.scale > 0.16;
    const radius = big ? Math.min(15, 4 + this.view.scale * 40) : Math.max(2.2, this.view.scale * 30);
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.font = `${String(Math.round(radius * 1.15))}px system-ui, "Segoe UI Emoji", sans-serif`;
    for (const zone of this.visible) {
      const [x, y] = worldToScreen(this.view, zone.x, zone.z);
      if (x < -20 || y < -20 || x > width + 20 || y > height + 20) continue;
      ctx.globalAlpha = (zone.pending ? 0.6 : 1) * (this.group ? 0.3 : 1);
      ctx.beginPath();
      ctx.arc(x, y, big ? radius : radius, 0, Math.PI * 2);
      ctx.fillStyle = COLORS[zone.kind] ?? '#ffffff';
      ctx.fill();
      if (zone.pending) {
        ctx.save();
        ctx.setLineDash([3, 3]);
        ctx.lineWidth = 1.5;
        ctx.strokeStyle = '#ffffff';
        ctx.beginPath();
        ctx.arc(x, y, radius + 3, 0, Math.PI * 2);
        ctx.stroke();
        ctx.restore();
      }
      if (big) this.drawNeed(ctx, zone.kind, x, y, radius * 1.25);
    }
    ctx.globalAlpha = 1;
  }

  /** Desenha o ícone da necessidade (folha, gota ou lua); sem `Path2D`, cai para o emoji. */
  private drawNeed(ctx: CanvasRenderingContext2D, kind: number, x: number, y: number, size: number): void {
    const name = NEED_ICONS[kind];
    if (!name || typeof Path2D === 'undefined') {
      ctx.fillStyle = '#10140f';
      ctx.font = `${String(Math.round(size))}px system-ui, "Segoe UI Emoji", sans-serif`;
      ctx.fillText(FALLBACK_GLYPHS[kind] ?? '', x, y + 1);
      return;
    }
    let path = this.iconShapes.get(kind);
    if (!path) {
      path = new Path2D(iconPath(name));
      this.iconShapes.set(kind, path);
    }
    ctx.save();
    ctx.translate(x - size / 2, y - size / 2);
    ctx.scale(size / 24, size / 24);
    ctx.lineWidth = 2.4;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.strokeStyle = '#10140f';
    ctx.stroke(path);
    ctx.restore();
  }

  private drawRoute(ctx: CanvasRenderingContext2D): void {
    if (!this.group) return;
    const stops = this.group.stops;
    const now = stopsNow(stops, this.hour);
    const points = stops.map((s) => worldToScreen(this.view, s.zone.x, s.zone.z));
    ctx.setLineDash([7, 6]);
    ctx.lineWidth = 3;
    ctx.strokeStyle = ACCENT;
    ctx.beginPath();
    points.forEach(([x, y], i) => {
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    if (points.length > 2 && points[0]) ctx.lineTo(points[0][0], points[0][1]);
    ctx.stroke();
    ctx.setLineDash([]);
    const size = Math.max(22, Math.min(40, this.view.scale * 90));
    stops.forEach((stop, i) => {
      const [x, y] = points[i] ?? [0, 0];
      if (now.has(stop)) {
        ctx.beginPath();
        ctx.arc(x, y, size / 2 + 11, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(255,255,255,0.85)';
        ctx.fill();
        ctx.lineWidth = 3;
        ctx.strokeStyle = ACCENT;
        ctx.stroke();
      }
      ctx.beginPath();
      ctx.arc(x, y, size / 2 + 3, 0, Math.PI * 2);
      ctx.fillStyle = COLORS[stop.zone.kind] ?? ACCENT;
      ctx.fill();
      this.drawNeed(ctx, stop.zone.kind, x, y, size * 0.62);
      ctx.beginPath();
      ctx.arc(x - size / 2 - 2, y - size / 2 - 2, 9, 0, Math.PI * 2);
      ctx.fillStyle = '#18201a';
      ctx.fill();
      ctx.lineWidth = 2;
      ctx.strokeStyle = ACCENT;
      ctx.stroke();
      ctx.fillStyle = '#ffffff';
      ctx.font = '600 11px ui-monospace, "Cascadia Mono", monospace';
      ctx.fillText(String(stop.n), x - size / 2 - 2, y - size / 2 - 1);
    });
  }
}

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { boundsOf, fitView, worldToScreen } from '../maplogic';
import { drawCalls } from '../test-setup';
import { mapReserves, sampleMap } from '../test-data';
import type { ReserveMap } from '../types';
import { MapView } from './map';

let root: HTMLElement;
const mounted: MapView[] = [];

beforeEach(() => {
  root = document.createElement('div');
  document.body.replaceChildren(root);
  drawCalls.length = 0;
});

afterEach(() => {
  for (const view of mounted.splice(0)) view.dispose();
});

function mount(map: ReserveMap = sampleMap, onReserve = vi.fn()): { view: MapView; onReserve: typeof onReserve } {
  const view = new MapView(root, mapReserves, map, onReserve);
  view.mount();
  mounted.push(view);
  return { view, onReserve };
}

const $ = (selector: string): HTMLElement => {
  const found = root.querySelector<HTMLElement>(selector);
  if (!found) throw new Error(`${selector} ausente`);
  return found;
};

const $input = (selector: string): HTMLInputElement => {
  const found = $(selector);
  if (!(found instanceof HTMLInputElement)) throw new Error(`${selector} não é um campo`);
  return found;
};

const $select = (selector: string): HTMLSelectElement => {
  const found = $(selector);
  if (!(found instanceof HTMLSelectElement)) throw new Error(`${selector} não é um seletor`);
  return found;
};

const text = (selector: string): string => $(selector).textContent;

/** Evento do ponteiro com as coordenadas relativas ao canvas (o jsdom não calcula `offsetX`). */
function pointer(type: string, x: number, y: number): void {
  const event = new MouseEvent(type, { bubbles: true, cancelable: true });
  Object.defineProperty(event, 'offsetX', { value: x });
  Object.defineProperty(event, 'offsetY', { value: y });
  $('#map-canvas').dispatchEvent(event);
}

/** Onde a zona `index` aparece na tela com o enquadramento inicial (área de 800 x 600 do jsdom). */
function screenOf(index: number): [number, number] {
  const view = fitView(boundsOf(sampleMap), 800, 600);
  const zone = sampleMap.zones[index];
  if (!zone) throw new Error('zona ausente');
  return worldToScreen(view, zone.x, zone.z);
}

describe('MapView', () => {
  it('shows the reserve, the need filters, the species and how many zones are on the map', () => {
    mount();
    expect([...root.querySelectorAll('#map-reserve [data-reserve]')].map((o) => o.textContent.trim())).toEqual([
      'Medved-Taiga',
      'Hirschfelden',
    ]);
    expect($('#map-reserve [data-reserve="2"]').getAttribute('aria-pressed')).toBe('true');
    expect(root.querySelectorAll('#map-reserve img')).toHaveLength(1);
    expect(root.querySelectorAll('[data-need]')).toHaveLength(3);
    expect(text('#map-count')).toBe('6');
    const species = [...root.querySelectorAll('#map-species .sp')].map((s) => s.textContent.replace(/\s+/g, ''));
    expect(species).toEqual(['Urso-pardo3', 'Alce2']);
    expect(drawCalls).toContain('fillRect');
  });

  it('hides a need when its chip is switched off', () => {
    mount();
    $('[data-need="1"]').click();
    expect($('[data-need="1"]').getAttribute('aria-pressed')).toBe('false');
    expect(text('#map-count')).toBe('4');
    $('[data-need="1"]').click();
    expect(text('#map-count')).toBe('6');
  });

  it('hides the places not discovered yet on request', () => {
    mount();
    const box = $input('#map-pending');
    box.checked = false;
    box.dispatchEvent(new Event('change'));
    expect(text('#map-count')).toBe('4');
  });

  it('shows only what is active at the chosen hour', () => {
    mount();
    const byHour = $input('#map-byhour');
    byHour.checked = true;
    byHour.dispatchEvent(new Event('change'));
    expect(text('#map-hour-text')).toBe('06:00');
    expect(text('#map-count')).toBe('2');
    const slider = $input('#map-hour');
    slider.value = '13';
    slider.dispatchEvent(new Event('input'));
    expect(text('#map-hour-text')).toBe('13:00');
    expect(text('#map-count')).toBe('1');
  });

  it('switches species on and off, all or none', () => {
    mount();
    $('#map-none').click();
    expect(text('#map-count')).toBe('1');
    $('#map-species [data-species="1"]').click();
    expect(text('#map-count')).toBe('3');
    $('#map-species [data-species="1"]').click();
    $('#map-all').click();
    expect(text('#map-count')).toBe('6');
  });

  it('follows a group: summary, animals, route with times and the Great One mark', () => {
    mount();
    expect([...root.querySelectorAll('#map-gsp option')].map((o) => o.textContent)).toEqual(['Urso-pardo', 'Alce']);
    const groups = [...root.querySelectorAll('#map-grp option')].map((o) => o.textContent);
    expect(groups[0]).toBe('Nenhum grupo');
    expect(groups[1]).toContain('⭐ Grupo 1');

    const select = $select('#map-grp');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    expect(text('#map-gsum')).toContain('2 animais: 1 machos e 1 fêmeas');
    expect(text('#map-anim')).toContain('⭐');
    expect(text('#map-anim')).toContain('480 kg');
    const route = [...root.querySelectorAll('#map-route li')].map((li) => li.textContent.replace(/\s+/g, ' ').trim());
    expect(route).toHaveLength(3);
    expect(route[0]).toContain('04:00 – 08:00');
    expect(route[0]).toContain('horário gravado no seu jogo');
    expect(route[2]).toContain('ainda não descoberto');
    expect($('#map-route-hint').hidden).toBe(false);

    select.value = '';
    select.dispatchEvent(new Event('change'));
    expect(root.querySelectorAll('#map-route li')).toHaveLength(0);
  });

  it('highlights where the group should be at the chosen hour', () => {
    mount();
    const select = $select('#map-grp');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    const slider = $input('#map-hour');
    slider.value = '9';
    slider.dispatchEvent(new Event('input'));
    const now = [...root.querySelectorAll('#map-route li.now')];
    expect(now).toHaveLength(1);
    expect(now[0]?.textContent).toContain('Beber');
  });

  it('picking a species for the tracker also filters the map to it', () => {
    mount();
    const species = $select('#map-gsp');
    species.value = '1';
    species.dispatchEvent(new Event('change'));
    expect(text('#map-count')).toBe('3');
    expect([...root.querySelectorAll('#map-grp option')].map((o) => o.textContent)).toHaveLength(2);
  });

  it('centers on a stop and shows its details when the route item is clicked', () => {
    mount();
    const select = $select('#map-grp');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    $('#map-route li[data-stop="1"]').click();
    expect($('#map-tip').hidden).toBe(false);
    expect(text('#map-tip')).toContain('Beber');
    expect(text('#map-tip')).toContain('Já descoberta');
    expect(text('#map-tip')).toContain('08:00 – 12:00');
  });

  it('shows the details of a zone under the pointer and its world coordinates', () => {
    mount();
    const [x, y] = screenOf(2);
    pointer('pointermove', x, y);
    expect($('#map-tip').hidden).toBe(false);
    expect(text('#map-tip')).toContain('Descansar');
    expect(text('#map-tip')).toContain('Ainda não descoberta');
    expect(text('#map-tip')).toContain('aproximado');
    expect(text('#map-x')).toBe('-10000');
    expect(text('#map-z')).toBe('-11000');
    pointer('pointermove', 5, 5);
    expect($('#map-tip').hidden).toBe(true);
  });

  it('pins a zone on click and unpins it when clicking empty ground', () => {
    mount();
    const [x, y] = screenOf(0);
    pointer('pointerdown', x, y);
    pointer('pointerup', x, y);
    expect($('#map-tip').hidden).toBe(false);
    pointer('pointermove', 5, 5);
    expect($('#map-tip').hidden).toBe(false);
    pointer('pointerdown', 5, 5);
    pointer('pointerup', 5, 5);
    expect($('#map-tip').hidden).toBe(true);
  });

  it('drags the map without pinning anything', () => {
    mount();
    const [x, y] = screenOf(0);
    pointer('pointerdown', x, y);
    pointer('pointermove', x + 40, y + 20);
    pointer('pointerup', x + 40, y + 20);
    expect($('#map-tip').hidden).toBe(true);
    pointer('pointerdown', 10, 10);
    pointer('pointerleave', 10, 10);
  });

  it('zooms with the wheel and the buttons and fits the reserve again', () => {
    mount();
    const before = drawCalls.length;
    const wheel = new WheelEvent('wheel', { deltaY: -200, cancelable: true });
    Object.defineProperty(wheel, 'offsetX', { value: 400 });
    Object.defineProperty(wheel, 'offsetY', { value: 300 });
    $('#map-canvas').dispatchEvent(wheel);
    expect(wheel.defaultPrevented).toBe(true);
    $('#map-zin').click();
    $('#map-zout').click();
    $('#map-zfit').click();
    expect(drawCalls.length).toBeGreaterThan(before);
  });

  it('draws the need icons as vector paths when the browser has Path2D', () => {
    const paths: string[] = [];
    vi.stubGlobal(
      'Path2D',
      vi.fn(function Path2DStub(d: string) {
        paths.push(d);
      }),
    );
    try {
      mount();
      for (let i = 0; i < 6; i += 1) $('#map-zin').click();
      expect(drawCalls).toContain('stroke');
      expect(paths.length).toBeGreaterThan(0);
      expect(new Set(paths).size).toBeLessThanOrEqual(3);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it('asks the host to load another reserve', () => {
    const { onReserve } = mount();
    $('#map-reserve [data-reserve="2"]').click();
    expect(onReserve).not.toHaveBeenCalled();
    $('#map-reserve [data-reserve="0"]').click();
    expect(onReserve).toHaveBeenCalledWith(0);
  });

  it('offers to unlock the missing places only when there are some', () => {
    mount();
    expect(root.querySelector('[data-action="show-unlock"]')).not.toBeNull();
    mount({ ...sampleMap, zones: sampleMap.zones.map((z) => ({ ...z, pending: false })) });
    expect(root.querySelector('[data-action="show-unlock"]')).toBeNull();
  });

  it('warns when the game has no map image and loads the image when it has', () => {
    mount();
    expect(root.textContent).toContain('O app ainda não conhece a imagem do mapa');
    mount({ ...sampleMap, image: 'data:image/jpeg;base64,AAAA' });
    expect(root.textContent).not.toContain('O app ainda não conhece a imagem do mapa');
  });

  it('warns when the image was fitted by the zones instead of a known placement', () => {
    mount({ ...sampleMap, image: 'data:image/jpeg;base64,AAAA' });
    expect(root.textContent).not.toContain('encaixou a imagem');
    mount({ ...sampleMap, image: 'data:image/jpeg;base64,AAAA', approximate: true });
    expect(root.textContent).toContain('encaixou a imagem');
  });

  it('handles a reserve without zones, species or groups', () => {
    mount({ ...sampleMap, zones: [], groups: [], species: [] });
    expect(text('#map-count')).toBe('0');
    expect(root.querySelectorAll('#map-gsp option')).toHaveLength(0);
  });

  it('stops listening to the window after dispose', () => {
    const { view } = mount();
    view.dispose();
    const before = drawCalls.length;
    window.dispatchEvent(new Event('resize'));
    expect(drawCalls.length).toBe(before);
  });
});

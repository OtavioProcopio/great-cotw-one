import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { changeOf, formOf, limitsOf, presetChanges, suggestFor, validate } from '../editlogic';
import { mapReserves, sampleEditor, sampleMap } from '../test-data';
import type { EditRequest } from '../types';
import { AnimalEditPanel, type EditorHooks, NO_EDITOR, type SaveOutcome } from './animalEdit';
import { MapView } from './map';

const male = sampleEditor.groups[0]?.animals[0];
const female = sampleEditor.groups[0]?.animals[1];
if (!male || !female) throw new Error('dados de teste');

describe('edit rules', () => {
  const form = formOf(male);

  it('uses the limits of the chosen sex and accepts what fits', () => {
    expect(limitsOf(sampleEditor.limits, true).maxWeight).toBe(660);
    expect(limitsOf(sampleEditor.limits, false).maxWeight).toBe(500);
    expect(validate(sampleEditor.limits, form)).toEqual({ weight: true, score: true });
  });

  it('refuses weight or score outside what the game has for that sex', () => {
    expect(validate(sampleEditor.limits, { ...form, weight: 700 })).toEqual({ weight: false, score: true });
    expect(validate(sampleEditor.limits, { ...form, score: 400 })).toEqual({ weight: true, score: false });
    // 560 kg cabe num macho, mas não numa fêmea.
    expect(validate(sampleEditor.limits, { ...form, male: false, score: 100 })).toEqual({ weight: false, score: true });
    expect(validate(sampleEditor.limits, { ...form, weight: Number.NaN }).weight).toBe(false);
    expect(validate(sampleEditor.limits, { ...form, weight: -1 }).weight).toBe(false);
  });

  it('suggests the typical values of the new sex', () => {
    expect(suggestFor(sampleEditor.limits, false)).toEqual({ weight: 320, score: 90 });
    expect(suggestFor(sampleEditor.limits, true)).toEqual({ weight: 400, score: 150 });
  });

  it('sends only what changed, and nothing when nothing changed', () => {
    expect(changeOf(1, male, form)).toBeNull();
    expect(changeOf(1, male, { ...form, weight: 561 })).toEqual({ group: 1, index: 0, weight: 561 });
    expect(changeOf(1, male, { ...form, male: false, weight: 300, score: 80, greatOne: false })).toEqual({
      group: 1,
      index: 0,
      male: false,
      weight: 300,
      score: 80,
      greatOne: false,
    });
    expect(changeOf(1, male, { ...form, weight: 560.04 })).toBeNull();
  });
});

describe('AnimalEditPanel', () => {
  let host: HTMLElement;

  beforeEach(() => {
    host = document.createElement('div');
    host.hidden = true;
    document.body.replaceChildren(host);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  function hooks(overrides: Partial<EditorHooks> = {}): EditorHooks & { requests: EditRequest[]; reloads: number; premiumOpens: number } {
    const state = { requests: [] as EditRequest[], reloads: 0, premiumOpens: 0 };
    return {
      ...state,
      premium: true,
      load: () => Promise.resolve(sampleEditor),
      save: (request) => {
        state.requests.push(request);
        return Promise.resolve<SaveOutcome>({ status: 'saved' });
      },
      openPremium: () => {
        state.premiumOpens += 1;
      },
      reload: () => {
        state.reloads += 1;
      },
      get requests() {
        return state.requests;
      },
      get reloads() {
        return state.reloads;
      },
      get premiumOpens() {
        return state.premiumOpens;
      },
      ...overrides,
    };
  }

  const field = (id: string): HTMLInputElement => {
    const found = host.querySelector<HTMLInputElement>(`#${id}`);
    if (!found) throw new Error(id);
    return found;
  };

  function type(id: string, value: string): void {
    field(id).value = value;
    field(id).dispatchEvent(new Event('input'));
  }

  it('sends a free user to the Premium page instead of opening', async () => {
    const h = hooks({ premium: false });
    await new AnimalEditPanel(host, h).open(2, 'moose', 1, 0);
    expect(h.premiumOpens).toBe(1);
    expect(host.hidden).toBe(true);
  });

  it('shows the animal, the accepted ranges and the rules', async () => {
    await new AnimalEditPanel(host, hooks()).open(2, 'moose', 1, 0);
    expect(host.hidden).toBe(false);
    expect(host.textContent).toContain('grupo 1 · animal 1');
    expect(host.textContent).toContain('Aceito: de 90 a 660. Típico: 400.');
    expect(host.textContent).toContain('não quebrar o jogo');
    expect(field('edit-weight').value).toBe('560');
    expect(field('edit-go').checked).toBe(true);
    expect(host.querySelector('#edit-save')).toHaveProperty('disabled', true);
  });

  it('explains that the fur is coming and cannot be changed yet', async () => {
    await new AnimalEditPanel(host, hooks()).open(2, 'moose', 1, 0);
    expect(host.textContent).toContain('Pelagem');
    expect(host.querySelector('select')?.disabled).toBe(true);
    expect(host.textContent).toContain('Em breve');
  });

  it('turns on the save button only for valid changes and flags what is out of range', async () => {
    await new AnimalEditPanel(host, hooks()).open(2, 'moose', 1, 0);
    const save = host.querySelector<HTMLButtonElement>('#edit-save');
    type('edit-weight', '600');
    expect(save?.disabled).toBe(false);
    type('edit-weight', '9999');
    expect(save?.disabled).toBe(true);
    expect(host.querySelector('#edit-weight-hint')?.classList.contains('bad')).toBe(true);
    type('edit-weight', '560');
    expect(save?.disabled).toBe(true);
    expect(host.querySelector('#edit-weight-hint')?.classList.contains('bad')).toBe(false);
  });

  it('suggests the typical values when the sex is swapped', async () => {
    await new AnimalEditPanel(host, hooks()).open(2, 'moose', 1, 0);
    host.querySelector<HTMLElement>('[data-edit-sex="female"]')?.click();
    expect(field('edit-weight').value).toBe('320');
    expect(field('edit-score').value).toBe('90');
    expect(host.querySelector('[data-edit-sex="female"]')?.getAttribute('aria-pressed')).toBe('true');
    expect(host.textContent).toContain('Aceito: de 80 a 500');
    // Clicar no sexo que já está marcado não muda os valores digitados.
    type('edit-weight', '300');
    host.querySelector<HTMLElement>('[data-edit-sex="female"]')?.click();
    expect(field('edit-weight').value).toBe('300');
  });

  it('saves only what changed, closes and reloads the map', async () => {
    const h = hooks();
    await new AnimalEditPanel(host, h).open(2, 'moose', 1, 0);
    host.querySelector<HTMLElement>('[data-edit-sex="female"]')?.click();
    host.querySelector<HTMLElement>('#edit-save')?.click();
    await Promise.resolve();
    await Promise.resolve();
    expect(h.requests).toEqual([
      { reserve: 2, species: 'moose', changes: [{ group: 1, index: 0, male: false, weight: 320, score: 90 }] },
    ]);
    expect(h.reloads).toBe(1);
    expect(host.hidden).toBe(true);
  });

  it('can switch the Great One mark when the species allows it, and blocks it when not', async () => {
    const h = hooks();
    await new AnimalEditPanel(host, h).open(2, 'moose', 1, 0);
    field('edit-go').checked = false;
    field('edit-go').dispatchEvent(new Event('change'));
    host.querySelector<HTMLElement>('#edit-save')?.click();
    await Promise.resolve();
    expect(h.requests[0]?.changes[0]).toEqual({ group: 1, index: 0, greatOne: false });

    const blocked = hooks({ load: () => Promise.resolve({ ...sampleEditor, greatOneAllowed: false }) });
    await new AnimalEditPanel(host, blocked).open(2, 'moose', 1, 1);
    expect(field('edit-go').disabled).toBe(true);
    expect(host.textContent).toContain('não tem Great One');
  });

  it('shows the error from the app inside the panel and keeps it open', async () => {
    const h = hooks({ save: () => Promise.resolve({ status: 'error', message: 'Fora da faixa.' }) });
    await new AnimalEditPanel(host, h).open(2, 'moose', 1, 0);
    type('edit-weight', '600');
    host.querySelector<HTMLElement>('#edit-save')?.click();
    await Promise.resolve();
    await Promise.resolve();
    expect(host.querySelector('#edit-error')?.textContent).toBe('Fora da faixa.');
    expect(host.querySelector<HTMLElement>('#edit-error')?.hidden).toBe(false);
    expect(host.hidden).toBe(false);
  });

  it('keeps the panel open when the player gives up at the confirmation', async () => {
    const h = hooks({ save: () => Promise.resolve({ status: 'cancelled' }) });
    await new AnimalEditPanel(host, h).open(2, 'moose', 1, 0);
    type('edit-weight', '600');
    host.querySelector<HTMLElement>('#edit-save')?.click();
    await Promise.resolve();
    await Promise.resolve();
    expect(host.hidden).toBe(false);
    expect(h.reloads).toBe(0);
  });

  it('says so when the animal cannot be loaded and closes with cancel', async () => {
    await new AnimalEditPanel(host, hooks({ load: () => Promise.reject(new Error('x')) })).open(2, 'moose', 1, 0);
    expect(host.textContent).toContain('Não foi possível abrir este animal');
    await new AnimalEditPanel(host, hooks({ load: () => Promise.resolve({ ...sampleEditor, groups: [] }) })).open(2, 'moose', 1, 0);
    expect(host.textContent).toContain('Não foi possível abrir este animal');

    const panel = new AnimalEditPanel(host, hooks());
    await panel.open(2, 'moose', 1, 0);
    host.querySelector<HTMLElement>('#edit-cancel')?.click();
    expect(host.hidden).toBe(true);
    expect(host.innerHTML).toBe('');
  });

  it('the default hooks do nothing and never save', async () => {
    expect(NO_EDITOR.premium).toBe(false);
    NO_EDITOR.openPremium();
    NO_EDITOR.reload();
    expect(await NO_EDITOR.save({ reserve: 0, species: 'x', changes: [] })).toEqual({ status: 'cancelled' });
    await expect(NO_EDITOR.load(0, 'x')).rejects.toThrow();
  });
});

describe('edit buttons in the group tracker', () => {
  let root: HTMLElement;

  beforeEach(() => {
    root = document.createElement('div');
    document.body.replaceChildren(root);
  });

  function mountTracker(premium: boolean): { opened: string[] } {
    const opened: string[] = [];
    const view = new MapView(root, mapReserves, sampleMap, vi.fn(), {
      ...NO_EDITOR,
      premium,
      load: (reserve, species) => {
        opened.push(`${String(reserve)}:${species}`);
        return Promise.resolve(sampleEditor);
      },
      openPremium: () => {
        opened.push('premium');
      },
    });
    view.mount();
    const select = root.querySelector<HTMLSelectElement>('#map-grp');
    if (!select) throw new Error('grupo');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    return { opened };
  }

  it('shows a pencil on each animal for a Premium user and opens the editor', async () => {
    const { opened } = mountTracker(true);
    const chips = root.querySelectorAll<HTMLButtonElement>('#map-anim [data-animal]');
    expect(chips).toHaveLength(2);
    expect(chips[0]?.title).toContain('Editar este animal');
    chips[0]?.click();
    await Promise.resolve();
    await Promise.resolve();
    expect(opened).toEqual(['2:brown_bear']);
    expect(root.querySelector('#map-edit')?.textContent).toContain('grupo 1 · animal 1');
  });

  it('shows a lock for a free user and sends them to the Premium page', () => {
    const { opened } = mountTracker(false);
    const chip = root.querySelector<HTMLButtonElement>('#map-anim [data-animal="1"]');
    expect(chip?.title).toContain('Premium');
    chip?.click();
    expect(opened).toEqual(['premium']);
  });

  it('closes the editor when another group is picked', async () => {
    mountTracker(true);
    root.querySelector<HTMLButtonElement>('#map-anim [data-animal="0"]')?.click();
    await Promise.resolve();
    await Promise.resolve();
    expect(root.querySelector<HTMLElement>('#map-edit')?.hidden).toBe(false);
    const select = root.querySelector<HTMLSelectElement>('#map-grp');
    if (!select) throw new Error('grupo');
    select.value = '';
    select.dispatchEvent(new Event('change'));
    expect(root.querySelector<HTMLElement>('#map-edit')?.hidden).toBe(true);
  });
});

describe('group presets', () => {
  const group = sampleEditor.groups[0];
  if (!group) throw new Error('dados de teste');

  it('typical sends every animal to the median of its own sex', () => {
    expect(presetChanges(sampleEditor.limits, group, 'typical')).toEqual([
      { group: 1, index: 0, weight: 400, score: 150 },
      { group: 1, index: 1, weight: 320, score: 90 },
    ]);
  });

  it('best sends every animal to the biggest the game has, without the 10% slack', () => {
    const changes = presetChanges(sampleEditor.limits, group, 'best');
    expect(changes).toEqual([
      { group: 1, index: 0, weight: 600, score: 300 },
      { group: 1, index: 1, weight: 454, score: 181 },
    ]);
    for (const c of changes) {
      const animal = group.animals[c.index];
      if (!animal) throw new Error('dados de teste');
      const ok = validate(sampleEditor.limits, { ...formOf(animal), weight: c.weight ?? 0, score: c.score ?? 0 });
      expect(ok).toEqual({ weight: true, score: true });
    }
  });

  it('leaves out animals that already have the values, and never touches sex or Great One', () => {
    const done = { ...group, animals: [{ index: 0, male: true, weight: 400, score: 150, greatOne: true }] };
    expect(presetChanges(sampleEditor.limits, done, 'typical')).toEqual([]);
  });
});

describe('group panel', () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="host"></div>';
  });

  function hooks(overrides: Partial<EditorHooks> = {}): { hooks: EditorHooks; requests: EditRequest[] } {
    const requests: EditRequest[] = [];
    return {
      requests,
      hooks: {
        premium: true,
        load: () => Promise.resolve(sampleEditor),
        save: (request) => {
          requests.push(request);
          return Promise.resolve<SaveOutcome>({ status: 'saved' });
        },
        openPremium: vi.fn(),
        reload: vi.fn(),
        ...overrides,
      },
    };
  }

  it('applies the chosen preset to the whole group in one save', async () => {
    const host = document.querySelector<HTMLElement>('#host');
    if (!host) throw new Error('host');
    const reload = vi.fn();
    const { hooks: h, requests } = hooks({ reload });
    const panel = new AnimalEditPanel(host, h);
    await panel.openGroup(2, 'moose', 1);
    host.querySelector<HTMLButtonElement>('[data-batch="best"]')?.click();
    host.querySelector<HTMLButtonElement>('#batch-apply')?.click();
    await Promise.resolve();
    expect(requests).toHaveLength(1);
    expect(requests[0]?.changes).toHaveLength(2);
    expect(requests[0]?.changes[0]).toEqual({ group: 1, index: 0, weight: 600, score: 300 });
    expect(reload).toHaveBeenCalled();
  });

  it('sends free users to the Premium page and opens nothing', async () => {
    const host = document.querySelector<HTMLElement>('#host');
    if (!host) throw new Error('host');
    const openPremium = vi.fn();
    const { hooks: h } = hooks({ premium: false, openPremium });
    await new AnimalEditPanel(host, h).openGroup(2, 'moose', 1);
    expect(openPremium).toHaveBeenCalled();
    expect(host.hidden).toBe(false);
    expect(host.innerHTML).toBe('');
  });
});

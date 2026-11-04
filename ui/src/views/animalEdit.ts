import { type EditForm, type Preset, changeOf, formOf, limitsOf, presetChanges, suggestFor, validate } from '../editlogic';
import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { EditRequest, EditableAnimal, EditableGroup, SpeciesEditor } from '../types';

/** O que o painel precisa do app: carregar os dados da espécie, gravar e levar ao Premium. */
export interface EditorHooks {
  premium: boolean;
  load(reserve: number, species: string): Promise<SpeciesEditor>;
  /** Pede a confirmação e grava; diz se gravou, se o jogador desistiu ou o que deu errado. */
  save(request: EditRequest): Promise<SaveOutcome>;
  openPremium(): void;
  /** Recarrega a tela do mapa depois de gravar. */
  reload(): void;
}

export type SaveOutcome = { status: 'saved' | 'cancelled' } | { status: 'error'; message: string };

export const NO_EDITOR: EditorHooks = {
  premium: false,
  load: () => Promise.reject(new Error('editor indisponível')),
  save: () => Promise.resolve({ status: 'cancelled' }),
  openPremium: () => undefined,
  reload: () => undefined,
};

function numberField(id: string, label: string, value: number, hint: string): string {
  return `<label class="field" for="${id}"><span>${e(label)}</span>
    <input type="number" id="${id}" min="0" step="1" value="${String(Math.round(value))}" inputmode="numeric" />
    <small id="${id}-hint" class="muted">${e(hint)}</small></label>`;
}

/** Painel de edição de um animal, dentro do rastreador do mapa. */
export class AnimalEditPanel {
  private editor: SpeciesEditor | null = null;
  private animal: EditableAnimal | null = null;
  private group = 0;
  /** Grupo inteiro aberto para ajuste em lote (quando não há um animal só). */
  private batch: EditableGroup | null = null;
  private preset: Preset = 'typical';
  private form: EditForm = { male: true, weight: 0, score: 0, greatOne: false };

  constructor(
    private readonly host: HTMLElement,
    private readonly hooks: EditorHooks,
  ) {}

  /** Abre o painel para um animal; sem Premium, leva à tela do Premium. */
  async open(
    reserve: number,
    species: string,
    group: number,
    animalIndex: number,
  ): Promise<void> {
    if (!this.hooks.premium) {
      this.hooks.openPremium();
      return;
    }
    this.host.hidden = false;
    this.host.innerHTML = `<p class="muted">${e(t('edit.loading'))}</p>`;
    try {
      const editor = await this.hooks.load(reserve, species);
      const animal = editor.groups.find((g) => g.number === group)?.animals.find((a) => a.index === animalIndex);
      if (!animal) throw new Error('animal ausente');
      this.editor = editor;
      this.animal = animal;
      this.group = group;
      this.form = formOf(animal);
      this.render();
    } catch {
      this.host.innerHTML = `<p class="notice notice-error">${e(t('edit.loadError'))}</p>`;
    }
  }

  /** Abre o ajuste do grupo todo; sem Premium, leva à tela do Premium. */
  async openGroup(reserve: number, species: string, group: number): Promise<void> {
    if (!this.hooks.premium) {
      this.hooks.openPremium();
      return;
    }
    this.host.hidden = false;
    this.host.innerHTML = `<p class="muted">${e(t('edit.loading'))}</p>`;
    try {
      const editor = await this.hooks.load(reserve, species);
      const data = editor.groups.find((g) => g.number === group);
      if (!data) throw new Error('grupo ausente');
      this.editor = editor;
      this.animal = null;
      this.batch = data;
      this.group = group;
      this.preset = 'typical';
      this.renderBatch();
    } catch {
      this.host.innerHTML = `<p class="notice notice-error">${e(t('edit.loadError'))}</p>`;
    }
  }

  private renderBatch(): void {
    const { editor, batch } = this;
    if (!editor || !batch) return;
    const count = presetChanges(editor.limits, batch, this.preset).length;
    const choice = (preset: Preset, label: string, note: string): string =>
      `<button class="seg wide" type="button" data-batch="${preset}" aria-pressed="${String(this.preset === preset)}"><strong>${e(label)}</strong><small>${e(note)}</small></button>`;
    this.host.innerHTML = `<h3>${icon('settings', 16)} ${e(t('batch.title', { group: this.group }))}</h3>
      <p class="muted fine">${e(t('batch.rules'))}</p>
      <div class="field"><div class="segmented stack">${choice('typical', t('batch.typical'), t('batch.typicalNote'))}${choice('best', t('batch.best'), t('batch.bestNote'))}</div></div>
      <p id="batch-count" class="muted">${e(count === 0 ? t('batch.none') : t('batch.count', { count }))}</p>
      <p class="notice notice-error" id="edit-error" hidden></p>
      <div class="actions"><button class="primary" type="button" id="batch-apply"${count === 0 ? ' disabled' : ''}>${e(t('batch.apply'))}</button>
        <button type="button" id="edit-cancel">${e(t('edit.cancel'))}</button></div>`;
    for (const button of this.host.querySelectorAll<HTMLButtonElement>('[data-batch]')) {
      button.addEventListener('click', () => {
        this.preset = button.dataset['batch'] === 'best' ? 'best' : 'typical';
        this.renderBatch();
      });
    }
    this.host.querySelector('#edit-cancel')?.addEventListener('click', () => {
      this.close();
    });
    this.host.querySelector('#batch-apply')?.addEventListener('click', () => {
      void this.saveBatch();
    });
  }

  private async saveBatch(): Promise<void> {
    const { editor, batch } = this;
    if (!editor || !batch) return;
    const changes = presetChanges(editor.limits, batch, this.preset);
    if (changes.length === 0) return;
    const outcome = await this.hooks.save({ reserve: editor.reserve, species: editor.species, changes });
    if (outcome.status === 'saved') {
      this.close();
      this.hooks.reload();
    } else if (outcome.status === 'error') {
      const box = this.host.querySelector<HTMLElement>('#edit-error');
      if (box) {
        box.textContent = outcome.message;
        box.hidden = false;
      }
    }
  }

  close(): void {
    this.editor = null;
    this.animal = null;
    this.batch = null;
    this.host.hidden = true;
    this.host.innerHTML = '';
  }

  private render(): void {
    const { editor, animal } = this;
    if (!editor || !animal) return;
    const range = limitsOf(editor.limits, this.form.male);
    const sex = (male: boolean): string =>
      `<button class="seg" type="button" data-edit-sex="${male ? 'male' : 'female'}" aria-pressed="${String(this.form.male === male)}">${e(t(male ? 'edit.male' : 'edit.female'))}</button>`;
    const range1 = (min: number, max: number, typical: number): string =>
      t('edit.range', { min: formatCount(Math.round(min)), max: formatCount(Math.round(max)), typical: formatCount(Math.round(typical)) });
    this.host.innerHTML = `<h3>${icon('settings', 16)} ${e(t('edit.title', { group: this.group, animal: animal.index + 1 }))}</h3>
      <p class="muted fine">${e(t('edit.rules'))}</p>
      <div class="field"><span>${e(t('edit.sex'))}</span><div class="segmented">${sex(true)}${sex(false)}</div></div>
      ${numberField('edit-weight', t('edit.weight'), this.form.weight, range1(range.minWeight, range.maxWeight, range.typicalWeight))}
      ${numberField('edit-score', t('edit.score'), this.form.score, range1(range.minScore, range.maxScore, range.typicalScore))}
      <label class="switch"><input type="checkbox" id="edit-go"${this.form.greatOne ? ' checked' : ''}${editor.greatOneAllowed ? '' : ' disabled'} /><span class="track" aria-hidden="true"></span>
        <span class="switch-text">${e(t('edit.greatOne'))}${editor.greatOneAllowed ? '' : ` <small class="muted">${e(t('edit.greatOneNo'))}</small>`}</span></label>
      <div class="field"><span>${e(t('edit.fur'))}</span><select disabled aria-label="${e(t('edit.fur'))}"><option>${e(t('home.soon'))}</option></select>
        <small class="muted">${e(t('edit.furNote'))}</small></div>
      <p class="notice notice-error" id="edit-error" hidden></p>
      <div class="actions"><button class="primary" type="button" id="edit-save">${e(t('edit.save'))}</button>
        <button type="button" id="edit-cancel">${e(t('edit.cancel'))}</button></div>`;
    this.bind();
    this.refresh();
  }

  private input(id: string): HTMLInputElement {
    const found = this.host.querySelector<HTMLInputElement>(`#${id}`);
    if (!found) throw new Error(`#${id} ausente`);
    return found;
  }

  private bind(): void {
    for (const button of this.host.querySelectorAll<HTMLButtonElement>('[data-edit-sex]')) {
      button.addEventListener('click', () => {
        const male = button.dataset['editSex'] === 'male';
        if (male === this.form.male || !this.editor) return;
        // Ao trocar de sexo, o app sugere os valores típicos do sexo novo (o jogador ainda pode ajustar).
        this.form = { ...this.form, male, ...suggestFor(this.editor.limits, male) };
        this.render();
      });
    }
    const read = (): void => {
      this.form = {
        ...this.form,
        weight: this.input('edit-weight').valueAsNumber,
        score: this.input('edit-score').valueAsNumber,
        greatOne: this.input('edit-go').checked,
      };
      this.refresh();
    };
    for (const id of ['edit-weight', 'edit-score', 'edit-go']) this.input(id).addEventListener('input', read);
    this.input('edit-go').addEventListener('change', read);
    this.host.querySelector('#edit-cancel')?.addEventListener('click', () => {
      this.close();
    });
    this.host.querySelector('#edit-save')?.addEventListener('click', () => {
      void this.save();
    });
  }

  /** Marca em vermelho o que está fora das regras e liga ou desliga o botão de gravar. */
  private refresh(): void {
    const { editor, animal } = this;
    if (!editor || !animal) return;
    const ok = validate(editor.limits, this.form);
    this.host.querySelector('#edit-weight-hint')?.classList.toggle('bad', !ok.weight);
    this.host.querySelector('#edit-score-hint')?.classList.toggle('bad', !ok.score);
    const save = this.host.querySelector<HTMLButtonElement>('#edit-save');
    if (save) save.disabled = !(ok.weight && ok.score) || changeOf(this.group, animal, this.form) === null;
  }

  private async save(): Promise<void> {
    const { editor, animal } = this;
    if (!editor || !animal) return;
    const change = changeOf(this.group, animal, this.form);
    if (!change) return;
    const outcome = await this.hooks.save({ reserve: editor.reserve, species: editor.species, changes: [change] });
    if (outcome.status === 'saved') {
      this.close();
      this.hooks.reload();
    } else if (outcome.status === 'error') {
      const box = this.host.querySelector<HTMLElement>('#edit-error');
      if (box) {
        box.textContent = outcome.message;
        box.hidden = false;
      }
    }
  }
}

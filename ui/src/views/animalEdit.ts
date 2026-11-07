import { type EditForm, MEDALS, type Medal, changeOf, formOf, limitsOf, medalOf, scoreForMedal, suggestFor, validate } from '../editlogic';
import { escapeHtml as e } from '../escape';
import { formatCount } from '../format';
import { t } from '../i18n';
import { icon } from '../icons';
import type { EditRequest, EditableAnimal, SpeciesEditor } from '../types';

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

  close(): void {
    this.editor = null;
    this.animal = null;
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
      ${this.medalBlock()}
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

  /** Medalhas da espécie: cada uma mostra a pontuação mínima e, ao tocar, preenche a pontuação. */
  private medalBlock(): string {
    const bands = this.editor?.medals;
    if (!bands || !this.form.male) return '';
    const buttons = MEDALS.map(
      (medal) =>
        `<button class="medal-btn medal-${medal}" type="button" data-edit-medal="${medal}" aria-pressed="false" title="${e(t('edit.medalUse', { name: t(`edit.medal.${medal}`) }))}">
          ${icon('trophy', 18)}<strong>${e(t(`edit.medal.${medal}`))}</strong><small>${e(formatCount(Math.round(bands[medal])))}+</small></button>`,
    ).join('');
    return `<div class="field medals"><span>${e(t('edit.medals'))}</span>
      <div class="medal-row">${buttons}</div>
      <small class="muted" id="edit-medal-now"></small></div>`;
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
    for (const medal of this.host.querySelectorAll<HTMLButtonElement>('[data-edit-medal]')) {
      medal.addEventListener('click', () => {
        const bands = this.editor?.medals;
        if (!this.editor || !bands) return;
        const range = limitsOf(this.editor.limits, true);
        const kind = medal.dataset['editMedal'] as Medal;
        this.input('edit-score').value = String(scoreForMedal(bands, kind, range.maxScore));
        read();
      });
    }
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
    this.showMedal();
    const ok = validate(editor.limits, this.form);
    this.host.querySelector('#edit-weight-hint')?.classList.toggle('bad', !ok.weight);
    this.host.querySelector('#edit-score-hint')?.classList.toggle('bad', !ok.score);
    const save = this.host.querySelector<HTMLButtonElement>('#edit-save');
    if (save) save.disabled = !(ok.weight && ok.score) || changeOf(this.group, animal, this.form) === null;
  }

  /** Marca a medalha que a pontuação digitada dá. */
  private showMedal(): void {
    const bands = this.editor?.medals;
    if (!bands || !this.form.male) return;
    const now = medalOf(bands, this.form.score);
    for (const button of this.host.querySelectorAll<HTMLButtonElement>('[data-edit-medal]')) {
      button.setAttribute('aria-pressed', String(button.dataset['editMedal'] === now));
    }
    const label = this.host.querySelector('#edit-medal-now');
    if (label) label.textContent = now ? t('edit.medalNow', { name: t(`edit.medal.${now}`) }) : '';
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

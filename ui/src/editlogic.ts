import type { AnimalChange, EditableAnimal, EditableGroup, Limits, SexLimits } from './types';

export interface EditForm {
  male: boolean;
  weight: number;
  score: number;
  greatOne: boolean;
}

export interface Validation {
  weight: boolean;
  score: boolean;
}

/** Os limites que valem para o sexo escolhido. */
export function limitsOf(limits: SexLimits, male: boolean): Limits {
  return male ? limits.male : limits.female;
}

/** Peso e pontuação cabem no que o jogo tem para o sexo? (as mesmas regras que o app aplica ao gravar) */
export function validate(limits: SexLimits, form: EditForm): Validation {
  const range = limitsOf(limits, form.male);
  const inside = (value: number, min: number, max: number): boolean =>
    Number.isFinite(value) && value >= min && value <= max;
  return {
    weight: inside(form.weight, range.minWeight, range.maxWeight),
    score: inside(form.score, range.minScore, range.maxScore),
  };
}

/** Valores sugeridos ao trocar de sexo: os típicos do sexo novo. */
export function suggestFor(limits: SexLimits, male: boolean): Pick<EditForm, 'weight' | 'score'> {
  const range = limitsOf(limits, male);
  return { weight: Math.round(range.typicalWeight), score: Math.round(range.typicalScore) };
}

/** Formulário a partir do animal como ele é hoje. */
export function formOf(animal: EditableAnimal): EditForm {
  return { male: animal.male, weight: animal.weight, score: animal.score, greatOne: animal.greatOne };
}

/** Só o que o jogador mudou; `null` se nada mudou. Números são comparados como o jogo guarda (inteiro arredondado na tela). */
export function changeOf(
  group: number,
  animal: EditableAnimal,
  form: EditForm,
): AnimalChange | null {
  const change: AnimalChange = { group, index: animal.index };
  if (form.male !== animal.male) change.male = form.male;
  if (Math.abs(form.weight - animal.weight) > 0.05) change.weight = form.weight;
  if (Math.abs(form.score - animal.score) > 0.05) change.score = form.score;
  if (form.greatOne !== animal.greatOne) change.greatOne = form.greatOne;
  return Object.keys(change).length === 2 ? null : change;
}

/** Predefinições para o grupo todo. */
export type Preset = 'typical' | 'best';

/** Folga que o app aceita além do maior animal real (`MARGIN` em `animal_editor.rs`). */
const SLACK = 1.1;
/** Absorve o erro de ponto flutuante de `máx × 1,1 ÷ 1,1`. */
const EPSILON = 1e-3;

/**
 * Mudanças que levam todos os animais do grupo à predefinição: `typical` = a mediana de cada
 * sexo; `best` = o maior animal real do sexo na espécie (sem a folga de 10 % que o app aceita).
 * Animais que já estão nesses valores ficam de fora; sexo e Great One nunca mudam.
 */
export function presetChanges(
  limits: SexLimits,
  group: EditableGroup,
  preset: Preset,
): AnimalChange[] {
  const changes: AnimalChange[] = [];
  for (const animal of group.animals) {
    const range = limitsOf(limits, animal.male);
    const target =
      preset === 'typical'
        ? { weight: Math.round(range.typicalWeight), score: Math.round(range.typicalScore) }
        : {
            weight: Math.floor(range.maxWeight / SLACK + EPSILON),
            score: Math.floor(range.maxScore / SLACK + EPSILON),
          };
    const change = changeOf(group.number, animal, { ...formOf(animal), ...target });
    if (change) changes.push(change);
  }
  return changes;
}

import type { AnimalChange, EditableAnimal, Limits, MedalBands, SexLimits } from './types';

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

export type Medal = 'bronze' | 'silver' | 'gold' | 'diamond';

export const MEDALS: Medal[] = ['bronze', 'silver', 'gold', 'diamond'];

/** Medalha de uma pontuação de macho, pelas faixas da espécie. */
export function medalOf(bands: MedalBands, score: number): Medal | null {
  if (!Number.isFinite(score) || score < 0) return null;
  if (score >= bands.diamond) return 'diamond';
  if (score >= bands.gold) return 'gold';
  if (score >= bands.silver) return 'silver';
  return 'bronze';
}

/** Menor pontuação inteira que dá a medalha, sem passar do máximo que o app aceita. */
export function scoreForMedal(bands: MedalBands, medal: Medal, maxAccepted: number): number {
  const wanted = Math.ceil(bands[medal]);
  return Math.min(wanted, Math.floor(maxAccepted));
}

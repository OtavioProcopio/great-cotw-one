// Tradução dos textos "do jogo" usados no modo de demonstração (`?mock`), para os prints em inglês
// e espanhol saírem coerentes. No app de verdade esses nomes vêm do próprio jogo no idioma escolhido.
import { getLocale } from './i18n';

const TEXTS: Record<string, { en: string; es: string }> = {
  Alce: { en: 'Moose', es: 'Alce' },
  Alces: { en: 'Moose', es: 'Alces' },
  'Urso-pardo': { en: 'Brown Bear', es: 'Oso pardo' },
  Rena: { en: 'Reindeer', es: 'Reno' },
  Javali: { en: 'Wild Boar', es: 'Jabalí' },
  'Lince-euroasiático': { en: 'Eurasian Lynx', es: 'Lince euroasiático' },
  'Lobo-cinzento': { en: 'Gray Wolf', es: 'Lobo gris' },
  'Cervo-almiscarado': { en: 'Siberian Musk Deer', es: 'Ciervo almizclero' },
  'Tetraz-grande': { en: 'Western Capercaillie', es: 'Urogallo' },
  'A milha': { en: 'The Mile', es: 'La milla' },
  'Viaje 1 milha a pé.': { en: 'Travel 1 mile on foot.', es: 'Recorre 1 milla a pie.' },
  Maratona: { en: 'Marathon', es: 'Maratón' },
  'Viaje 42 quilômetros a pé.': { en: 'Travel 42 kilometers on foot.', es: 'Recorre 42 kilómetros a pie.' },
  'Primeiro troféu': { en: 'First Trophy', es: 'Primer trofeo' },
  'Pendure o seu primeiro troféu no salão.': { en: 'Hang your first trophy in the lodge.', es: 'Cuelga tu primer trofeo en el salón.' },
  'Tiro certeiro': { en: 'Sharpshooter', es: 'Tiro certero' },
  'Abata um animal a mais de 300 metros.': { en: 'Harvest an animal from over 300 meters.', es: 'Abate un animal a más de 300 metros.' },
  Fotógrafo: { en: 'Photographer', es: 'Fotógrafo' },
  'Fotografe 10 espécies diferentes.': { en: 'Photograph 10 different species.', es: 'Fotografía 10 especies distintas.' },
  'Caçador de lendas': { en: 'Legend Hunter', es: 'Cazador de leyendas' },
  'Abata um animal Great One.': { en: 'Harvest a Great One.', es: 'Abate un Great One.' },
  Colecionador: { en: 'Collector', es: 'Coleccionista' },
  'Tenha 50 troféus no salão.': { en: 'Have 50 trophies in the lodge.', es: 'Ten 50 trofeos en el salón.' },
  'Conquista secreta': { en: 'Secret achievement', es: 'Logro secreto' },
  Explorador: { en: 'Explorer', es: 'Explorador' },
  'Descubra todas as zonas de uma reserva.': { en: 'Discover every zone in a reserve.', es: 'Descubre todas las zonas de una reserva.' },
  'Mestre dos rifles': { en: 'Rifle Master', es: 'Maestro de rifles' },
  'Chegue ao nível máximo de rifles.': { en: 'Reach the maximum rifle level.', es: 'Llega al nivel máximo de rifles.' },
  'Crie pressão de caça perto do Lago Zhizni.': { en: 'Create hunting pressure near Lake Zhizni.', es: 'Crea presión de caza cerca del lago Zhizni.' },
  'Fotografe um alce na margem do rio.': { en: 'Photograph a moose on the riverbank.', es: 'Fotografía un alce en la orilla del río.' },
  'Abata e recolha um lince nas Terras de Lesnye.': { en: 'Harvest a lynx in the Lesnye Lands.', es: 'Abate y recoge un lince en las Tierras de Lesnye.' },
  'Encontre as pegadas de um urso-pardo.': { en: 'Find the tracks of a brown bear.', es: 'Encuentra las huellas de un oso pardo.' },
  'Vá até a cabana abandonada.': { en: 'Go to the abandoned cabin.', es: 'Ve a la cabaña abandonada.' },
  'Identifique os rastros de uma rena.': { en: 'Identify the tracks of a reindeer.', es: 'Identifica el rastro de un reno.' },
  'Abata um javali a mais de 150 metros.': { en: 'Harvest a wild boar from over 150 meters.', es: 'Abate un jabalí a más de 150 metros.' },
  'Fotografe três espécies diferentes.': { en: 'Photograph three different species.', es: 'Fotografía tres especies distintas.' },
};

/** Troca, em qualquer profundidade, os textos de exemplo pelo idioma atual do app. */
export function localize<T>(value: T): T {
  const locale = getLocale();
  if (locale === 'pt-BR') return value;
  const walk = (v: unknown): unknown => {
    if (typeof v === 'string') return TEXTS[v]?.[locale] ?? v;
    if (Array.isArray(v)) return v.map(walk);
    if (v && typeof v === 'object') return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, walk(x)]));
    return v;
  };
  return walk(value) as T;
}

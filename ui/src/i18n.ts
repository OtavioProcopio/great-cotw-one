// Idiomas da interface: português do Brasil (referência), inglês e espanhol.
// Para acrescentar um idioma: um arquivo em `locales/` com as mesmas chaves e uma linha em `LOCALES`.

import { en } from './locales/en';
import { es } from './locales/es';
import { type MessageKey, ptBR } from './locales/pt-BR';

export type { MessageKey } from './locales/pt-BR';

export type Locale = 'pt-BR' | 'en' | 'es';

/** Idiomas oferecidos, com o nome de cada um escrito no próprio idioma. */
export const LOCALES: readonly { code: Locale; name: string }[] = [
  { code: 'pt-BR', name: 'Português (Brasil)' },
  { code: 'en', name: 'English' },
  { code: 'es', name: 'Español' },
];

const DICTIONARIES: Record<Locale, Record<MessageKey, string>> = { 'pt-BR': ptBR, en, es };

const STORAGE_KEY = 'great-cotw-one.locale';

let current: Locale = 'pt-BR';

export function getLocale(): Locale {
  return current;
}

export function setLocale(locale: Locale): void {
  current = locale;
  if (typeof document !== 'undefined') document.documentElement.lang = locale;
}

/** `pt`, `pt-PT`, `en-US`, `es-MX`… → idioma que o app oferece; desconhecido → `null`. */
export function matchLocale(tag: string): Locale | null {
  const language = tag.toLowerCase().split(/[-_]/)[0];
  if (language === 'pt') return 'pt-BR';
  if (language === 'en') return 'en';
  return language === 'es' ? 'es' : null;
}

/** O idioma do jogador: o que ele escolheu antes, senão o do sistema; sem nenhum dos dois, inglês. */
export function detectLocale(saved: string | null, system: readonly string[]): Locale {
  const chosen = saved === null ? null : matchLocale(saved);
  if (chosen) return chosen;
  for (const tag of system) {
    const found = matchLocale(tag);
    if (found) return found;
  }
  return 'en';
}

/** Idioma guardado no navegador (nunca falha: sem acesso ao armazenamento, devolve `null`). */
export function savedLocale(): string | null {
  try {
    return window.localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

export function saveLocale(locale: Locale): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Sem armazenamento, a escolha vale só até fechar o app.
  }
}

/** Texto traduzido com `{marcadores}` substituídos. */
export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  const template = DICTIONARIES[current][key];
  return template.replace(/\{(\w+)\}/g, (whole, name: string) => {
    const value = params[name];
    return value === undefined ? whole : String(value);
  });
}

/** Mensagem para um código de erro dos comandos; sem tradução, usa o texto original. */
export function errorText(code: string, message: string): string {
  const key = `error.${code}`;
  return key in ptBR ? t(key as MessageKey) : t('error.generic', { message });
}

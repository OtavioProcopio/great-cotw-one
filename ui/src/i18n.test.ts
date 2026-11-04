import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  LOCALES,
  type Locale,
  type MessageKey,
  detectLocale,
  errorText,
  getLocale,
  matchLocale,
  saveLocale,
  savedLocale,
  setLocale,
  t,
} from './i18n';
import { en } from './locales/en';
import { es } from './locales/es';
import { ptBR } from './locales/pt-BR';

describe('t', () => {
  it('replaces placeholders', () => {
    expect(t('game.tabFiles', { count: 144 })).toBe('144 índices (.tab)');
  });

  it('keeps an unfilled placeholder visible instead of hiding the bug', () => {
    expect(t('home.save.profile')).toBe('Perfil Steam {id}');
  });

  it('returns messages without placeholders as written', () => {
    expect(t('nav.history')).toBe('Backups');
  });
});

describe('errorText', () => {
  it('translates known error codes', () => {
    expect(errorText('gameRunning', 'x')).toContain('Feche o theHunter');
  });

  it('falls back to a generic message with the original text', () => {
    expect(errorText('codigoNovo', 'detalhe técnico')).toBe('Algo deu errado: detalhe técnico');
  });
});

const DICTIONARIES: [Locale, Record<MessageKey, string>][] = [
  ['pt-BR', ptBR],
  ['en', en],
  ['es', es],
];
const keys = Object.keys(ptBR) as MessageKey[];
const placeholders = (text: string): string[] => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1] ?? '').sort();

describe('dictionaries', () => {
  it('have every key, with no empty text', () => {
    for (const [locale, dictionary] of DICTIONARIES) {
      expect(Object.keys(dictionary).sort(), locale).toEqual([...keys].sort());
      for (const key of keys) expect(dictionary[key].trim(), `${locale} ${key}`).not.toBe('');
    }
  });

  it('use the same {placeholders} as Portuguese in every language', () => {
    for (const [locale, dictionary] of DICTIONARIES) {
      for (const key of keys) {
        expect(placeholders(dictionary[key]), `${locale} ${key}`).toEqual(placeholders(ptBR[key]));
      }
    }
  });

  it('keep the same line breaks in the long confirmations', () => {
    for (const [locale, dictionary] of DICTIONARIES) {
      for (const key of keys) {
        expect(dictionary[key].split('\n').length, `${locale} ${key}`).toBe(ptBR[key].split('\n').length);
      }
    }
  });

  it('are actually translated: most texts differ from Portuguese', () => {
    for (const [locale, dictionary] of DICTIONARIES.slice(1)) {
      const same = keys.filter((k) => dictionary[k] === ptBR[k]);
      // Nomes próprios, unidades e palavras iguais em português e espanhol (Great One, Premium, kg…) são esperados.
      expect(same.length / keys.length, locale).toBeLessThan(0.15);
    }
  });

  it('offer exactly the languages that have a dictionary', () => {
    expect(LOCALES.map((l) => l.code)).toEqual(['pt-BR', 'en', 'es']);
  });
});

describe('locale', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('switches every text, the page language and the number formats', () => {
    setLocale('en');
    expect(getLocale()).toBe('en');
    expect(t('nav.home')).toBe('Home');
    expect(document.documentElement.lang).toBe('en');
    setLocale('es');
    expect(t('nav.home')).toBe('Inicio');
    expect(t('game.tabFiles', { count: 144 })).toBe('144 índices (.tab)');
    setLocale('pt-BR');
    expect(t('nav.home')).toBe('Início');
  });

  it('translates the error messages too', () => {
    setLocale('en');
    expect(errorText('gameRunning', 'x')).toContain('Close theHunter');
    setLocale('es');
    expect(errorText('gameRunning', 'x')).toContain('Cierra theHunter');
    expect(errorText('codigoNovo', 'detalle')).toBe('Algo salió mal: detalle');
  });

  it('matches the tags of the browser and the system', () => {
    expect(matchLocale('pt')).toBe('pt-BR');
    expect(matchLocale('pt-PT')).toBe('pt-BR');
    expect(matchLocale('EN-us')).toBe('en');
    expect(matchLocale('es_MX')).toBe('es');
    expect(matchLocale('fr')).toBeNull();
    expect(matchLocale('')).toBeNull();
  });

  it('prefers the saved choice, then the system, then English', () => {
    expect(detectLocale('es', ['pt-BR'])).toBe('es');
    expect(detectLocale(null, ['fr-FR', 'es-AR', 'en'])).toBe('es');
    expect(detectLocale('xx', ['pt-BR'])).toBe('pt-BR');
    expect(detectLocale(null, ['fr', 'de'])).toBe('en');
    expect(detectLocale(null, [])).toBe('en');
  });

  it('remembers the choice and survives a browser without storage', () => {
    window.localStorage.clear();
    expect(savedLocale()).toBeNull();
    saveLocale('es');
    expect(savedLocale()).toBe('es');
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('bloqueado');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('bloqueado');
    });
    expect(savedLocale()).toBeNull();
    expect(() => {
      saveLocale('en');
    }).not.toThrow();
  });
});

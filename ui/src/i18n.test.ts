import { describe, expect, it } from 'vitest';

import { errorText, t } from './i18n';

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

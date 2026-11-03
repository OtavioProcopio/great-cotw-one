import { describe, expect, it } from 'vitest';

import { foundDiagnostics, historyItems, zonesPreview } from '../test-data';
import type { Diagnostics } from '../types';
import { renderDiagnostics } from './diagnostics';
import { defaultRewriteFile, renderHome } from './home';
import { renderHistory } from './history';
import { renderUnlock } from './unlock';

function html(markup: string): HTMLElement {
  const root = document.createElement('div');
  root.innerHTML = markup;
  return root;
}

describe('renderDiagnostics', () => {
  it('shows game, save, reserves and file states', () => {
    const root = html(renderDiagnostics(foundDiagnostics));
    const text = root.textContent;
    expect(text).toContain('Instalação encontrada');
    expect(text).toContain('144 índices (.tab)');
    expect(root.querySelector('dl')?.textContent).toContain('Versão (build)26051119');
    expect(text).toContain('Conta Steam 111');
    expect(text).toContain('Medved-Taiga');
    expect(text).toContain('Reserva 99');
    expect(text).toContain('1 reconhecidos · 2 não reconhecidos');
    expect(text).toContain('Animais da reserva 2');
    expect(text).toContain('Não foi possível ler: só leitura');
    expect(root.querySelectorAll('tbody tr')).toHaveLength(3);
  });

  it('escapes file names coming from the disk', () => {
    const root = html(renderDiagnostics(foundDiagnostics));
    expect(root.querySelector('script')).toBeNull();
    expect(root.textContent).toContain('<script>alert(1)</script>');
  });

  it('warns about limited mode when the game is missing', () => {
    const missing: Diagnostics = {
      ...foundDiagnostics,
      limitedMode: true,
      game: { ...foundDiagnostics.game, found: false, root: null, archives: null, buildId: null },
    };
    const text = html(renderDiagnostics(missing)).textContent;
    expect(text).toContain('Jogo não encontrado');
    expect(text).toContain('Modo limitado');
  });

  it('lists the accounts when several saves exist', () => {
    const choose: Diagnostics = {
      ...foundDiagnostics,
      save: {
        status: 'choose',
        account: null,
        accounts: [
          { steamId: '111', path: 'C:\\a', lastModified: 1_793_600_000 },
          { steamId: '222', path: 'C:\\b', lastModified: null },
        ],
        searched: [],
        manual: false,
      },
    };
    const root = html(renderDiagnostics(choose));
    expect(root.textContent).toContain('2 contas Steam');
    const buttons = root.querySelectorAll<HTMLElement>('[data-action="use-account"]');
    expect([...buttons].map((b) => b.dataset['path'])).toEqual(['C:\\a', 'C:\\b']);
    expect(root.querySelector('table')).toBeNull();
  });

  it('explains where it looked when no save is found', () => {
    const none: Diagnostics = {
      ...foundDiagnostics,
      save: { status: 'notFound', account: null, accounts: [], searched: ['D:\\Docs\\Saves'], manual: false },
    };
    const text = html(renderDiagnostics(none)).textContent;
    expect(text).toContain('Save não encontrado');
    expect(text).toContain('D:\\Docs\\Saves');
  });

  it('offers to go back to the automatic search after a manual choice', () => {
    const manual: Diagnostics = {
      ...foundDiagnostics,
      game: { ...foundDiagnostics.game, manual: true },
      save: { ...foundDiagnostics.save, manual: true },
    };
    const root = html(renderDiagnostics(manual));
    expect(root.querySelector('[data-action="reset-choices"]')).not.toBeNull();
    expect(root.textContent).toContain('Pasta indicada manualmente');
  });

  it('handles an empty save folder', () => {
    const empty: Diagnostics = { ...foundDiagnostics, reserves: [], files: [] };
    const text = html(renderDiagnostics(empty)).textContent;
    expect(text).toContain('Nenhuma reserva encontrada');
    expect(text).toContain('Nenhum arquivo para mostrar');
  });
});

describe('renderHome', () => {
  it('says everything is ready in plain words, without paths or technical terms', () => {
    const text = html(renderHome(foundDiagnostics)).textContent;
    expect(text).toContain('O jogo foi encontrado');
    expect(text).toContain('Seu progresso foi encontrado: 2 reservas');
    expect(text).not.toContain('C:\\');
    expect(text).not.toContain('.tab');
  });

  it('offers the map only when the game is installed', () => {
    expect(html(renderHome(foundDiagnostics)).querySelector('[data-action="show-map"]')).not.toBeNull();
    const missing: Diagnostics = {
      ...foundDiagnostics,
      game: { ...foundDiagnostics.game, found: false, root: null, archives: null, buildId: null },
    };
    expect(html(renderHome(missing)).querySelector('[data-action="show-map"]')).toBeNull();
  });

  it('offers to unlock the map only when the game is installed', () => {
    const root = html(renderHome(foundDiagnostics));
    expect(root.querySelector('[data-action="show-unlock"]')).not.toBeNull();
    const missing: Diagnostics = {
      ...foundDiagnostics,
      game: { ...foundDiagnostics.game, found: false, root: null, archives: null, buildId: null },
    };
    expect(html(renderHome(missing)).querySelector('[data-action="show-unlock"]')).toBeNull();
  });

  it('offers the test, the backups and the upcoming features', () => {
    const root = html(renderHome(foundDiagnostics));
    expect(root.querySelector('[data-action="rewrite-unchanged"]')).not.toBeNull();
    expect(root.querySelector('[data-action="show-history"]')).not.toBeNull();
    expect(root.querySelectorAll('.tile-soon')).toHaveLength(2);
    expect(root.textContent).toContain('Em breve');
  });

  it('hides the test when no file is readable', () => {
    const files = foundDiagnostics.files.map((f) => ({ ...f, state: 'unreadable' as const }));
    const root = html(renderHome({ ...foundDiagnostics, files }));
    expect(root.querySelector('[data-action="rewrite-unchanged"]')).toBeNull();
  });

  it('asks to locate the game when it is missing', () => {
    const missing: Diagnostics = {
      ...foundDiagnostics,
      limitedMode: true,
      game: { ...foundDiagnostics.game, found: false, root: null, archives: null, buildId: null },
    };
    const root = html(renderHome(missing));
    expect(root.textContent).toContain('Não encontramos o jogo');
    expect(root.querySelector('[data-action="choose-game-folder"]')).not.toBeNull();
  });

  it('lets the player pick their Steam profile', () => {
    const choose: Diagnostics = {
      ...foundDiagnostics,
      save: {
        status: 'choose',
        account: null,
        accounts: [
          { steamId: '111', path: 'C:\\a', lastModified: 1_793_600_000 },
          { steamId: '222', path: 'C:\\b', lastModified: null },
        ],
        searched: [],
        manual: false,
      },
    };
    const root = html(renderHome(choose));
    const buttons = root.querySelectorAll<HTMLElement>('[data-action="use-account"]');
    expect([...buttons].map((b) => b.dataset['path'])).toEqual(['C:\\a', 'C:\\b']);
    expect(root.querySelector('[data-action="rewrite-unchanged"]')).toBeNull();
  });

  it('asks to locate the progress when no save is found', () => {
    const none: Diagnostics = {
      ...foundDiagnostics,
      save: { status: 'notFound', account: null, accounts: [], searched: [], manual: false },
    };
    const root = html(renderHome(none));
    expect(root.textContent).toContain('Não encontramos o seu progresso');
    expect(root.querySelector('[data-action="choose-save-folder"]')).not.toBeNull();
    expect(root.querySelector('.tiles')).toBeNull();
  });
});

describe('defaultRewriteFile', () => {
  it('prefers the reserve 2 population, then any population, then the first readable file', () => {
    const files = foundDiagnostics.files.map((f) => ({ ...f, state: 'ok' as const }));
    expect(defaultRewriteFile(files)).toBe('animal_population_2');
    const others = files.filter((f) => f.name !== 'animal_population_2');
    expect(defaultRewriteFile(others)).toBe('settings_adf');
    const pop = [{ name: 'animal_population_5', size: 1, kind: 'population' as const, reserve: 5, state: 'ok' as const }, ...others];
    expect(defaultRewriteFile(pop)).toBe('animal_population_5');
    expect(defaultRewriteFile([])).toBeUndefined();
  });
});

describe('renderUnlock', () => {
  it('shows a waiting message while the progress is analysed', () => {
    const root = html(renderUnlock(null));
    expect(root.textContent).toContain('Analisando o seu progresso');
    expect(root.querySelector('[data-action="unlock-run"]')).toBeNull();
  });

  it('lets the player pick only the reserves that have something to unlock', () => {
    const root = html(renderUnlock(zonesPreview));
    const picks = [...root.querySelectorAll<HTMLInputElement>('.reserve-pick')];
    expect(picks.map((p) => p.value)).toEqual(['2', '13']);
    expect(picks.every((p) => p.checked)).toBe(true);
    expect(root.textContent).toContain('3.511 lugares novos no mapa');
    expect(root.textContent).toContain('89 com horário aproximado');
    expect(root.textContent).toContain('Serão liberados 7.330 lugares em 2 reservas');
    expect(root.querySelector('[data-action="unlock-run"]')).not.toBeNull();
  });

  it('explains the reserves that cannot be unlocked in plain words', () => {
    const text = html(renderUnlock(zonesPreview)).textContent;
    expect(text).toContain('Tudo já está liberado');
    expect(text).toContain('Jogue um pouco nesta reserva');
    expect(text).toContain('Sem dados desta reserva');
  });

  it('offers no button when there is nothing to unlock', () => {
    const none = { total: 0, reserves: zonesPreview.reserves.filter((r) => r.state !== 'ready') };
    const root = html(renderUnlock(none));
    expect(root.textContent).toContain('Não há nada para liberar agora');
    expect(root.querySelector('[data-action="unlock-run"]')).toBeNull();
  });
});

describe('renderHistory', () => {
  it('lists entries with dates, files and a restore button', () => {
    const root = html(renderHistory(historyItems));
    expect(root.textContent).toContain('Liberar zonas');
    expect(root.textContent).toContain('02/11/2026 15:30:00');
    expect(root.textContent).toContain('found_need_zones_adf');
    expect(root.textContent).toContain('Pasta inteira');
    const restore = root.querySelectorAll<HTMLElement>('[data-action="restore"]');
    expect(restore).toHaveLength(1);
    expect(restore[0]?.dataset['backup']).toBe('C:\\Backups\\2026-11-02_15-30-00');
  });

  it('warns instead of offering restore when the backup is gone', () => {
    const root = html(renderHistory(historyItems));
    expect(root.textContent).toContain('não existe mais');
  });

  it('shows a friendly message when empty', () => {
    expect(html(renderHistory([])).textContent).toContain('Ainda não há nada');
  });
});

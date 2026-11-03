import { beforeEach, describe, expect, it } from 'vitest';

import { Controller, describeError } from './controller';
import { commandError, fakeApi, foundDiagnostics } from './test-data';

let root: HTMLElement;

beforeEach(() => {
  root = document.createElement('div');
  document.body.replaceChildren(root);
});

function click(action: string, data: Record<string, string> = {}): void {
  const button = document.createElement('button');
  button.dataset['action'] = action;
  Object.assign(button.dataset, data);
  root.appendChild(button);
  button.click();
}

/** Deixa as promessas pendentes terminarem. */
async function settle(): Promise<void> {
  for (let i = 0; i < 10; i += 1) await Promise.resolve();
}

describe('Controller', () => {
  it('starts on the home screen', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    expect(root.textContent).toContain('O jogo foi encontrado');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Início');
    expect(api.calls).toEqual(['diagnostics', 'mapReserves']);
  });

  it('shows the reserves the app recognised and opens the one the player taps', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    expect(root.textContent).toContain('Suas reservas');
    expect([...root.querySelectorAll('.reserve-gallery .reserve-name')].map((n) => n.textContent)).toEqual([
      'Medved-Taiga',
      'Hirschfelden',
    ]);
    expect(root.querySelectorAll('.reserve-gallery img')).toHaveLength(1);
    root.querySelector<HTMLElement>('[data-action="open-reserve"][data-reserve="0"]')?.click();
    await settle();
    expect(api.calls).toContain('reserveMap:0');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Mapa');
  });

  it('still opens the home screen when the reserves cannot be listed', async () => {
    const api = fakeApi({ mapReserves: () => Promise.reject(new Error('sem jogo')) });
    await new Controller(root, api).start();
    expect(root.textContent).toContain('O jogo foi encontrado');
    expect(root.querySelector('.reserve-gallery')).toBeNull();
  });

  it('switches between home, backups and technical details', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('show-history');
    await settle();
    expect(root.textContent).toContain('Liberar zonas');
    click('show-diagnostics');
    await settle();
    expect(root.textContent).toContain('Instalação encontrada');
    click('show-home');
    await settle();
    expect(root.textContent).toContain('Bem-vindo, caçador!');
  });

  it('refreshes the current screen', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('refresh');
    await settle();
    expect(api.calls.filter((c) => c === 'diagnostics')).toHaveLength(2);
  });

  it('applies the chosen account and reloads', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('use-account', { path: 'C:\\Saves\\222' });
    await settle();
    expect(api.calls).toContain('selectSaveDir:C:\\Saves\\222');
    expect(api.calls.filter((c) => c === 'diagnostics')).toHaveLength(2);
  });

  it('uses the folder picker for the save and the game', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('choose-save-folder');
    await settle();
    click('choose-game-folder');
    await settle();
    expect(api.calls).toContain('selectSaveDir:C:\\Escolhida');
    expect(api.calls).toContain('selectGameDir:C:\\Escolhida');
  });

  it('does nothing when the folder picker is cancelled', async () => {
    const api = fakeApi({ pickFolder: () => Promise.resolve(null) });
    await new Controller(root, api).start();
    click('choose-save-folder');
    await settle();
    expect(api.calls.some((c) => c.startsWith('selectSaveDir'))).toBe(false);
  });

  it('shows a translated error when the chosen folder is invalid', async () => {
    const api = fakeApi({
      selectSaveDir: () => commandError('invalidSaveDir'),
    });
    await new Controller(root, api).start();
    click('choose-save-folder');
    await settle();
    expect(root.querySelector('[role="alert"]')?.textContent).toBe(
      'Essa pasta não contém arquivos de save do jogo.',
    );
  });

  it('resets manual choices', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('reset-choices');
    await settle();
    expect(api.calls).toContain('resetChoices');
  });

  it('restores a backup only after the user confirms', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('show-history');
    await settle();
    click('restore', { backup: 'C:\\Backups\\x', at: '2026-11-02_15-30-00' });
    await settle();
    expect(api.calls).toContain('restore:C:\\Backups\\x');
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Seu progresso voltou');
  });

  it('does not restore when the user declines', async () => {
    const api = fakeApi({ confirm: () => Promise.resolve(false) });
    await new Controller(root, api).start();
    click('show-history');
    await settle();
    click('restore', { backup: 'C:\\Backups\\x', at: 'a' });
    await settle();
    expect(api.calls.some((c) => c.startsWith('restore'))).toBe(false);
  });

  it('explains that the game must be closed when restoring fails', async () => {
    const api = fakeApi({
      restore: () => commandError('gameRunning'),
    });
    await new Controller(root, api).start();
    click('show-history');
    await settle();
    click('restore', { backup: 'C:\\Backups\\x', at: 'a' });
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('Feche o theHunter');
  });

  it('rewrites the selected file unchanged after confirmation and explains the next step', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('rewrite-unchanged');
    await settle();
    expect(api.calls).toContain('rewriteUnchanged:animal_population_2');
    const notice = root.querySelector('.notice-ok')?.textContent ?? '';
    expect(notice).toContain('Teste feito');
    expect(notice).toContain('abra o jogo');
    expect(notice).not.toContain('C:\\');
    // Recarrega o diagnóstico depois da gravação.
    expect(api.calls.filter((c) => c === 'diagnostics')).toHaveLength(2);
  });

  it('does not rewrite when the user declines the confirmation', async () => {
    const api = fakeApi({ confirm: () => Promise.resolve(false) });
    await new Controller(root, api).start();
    click('rewrite-unchanged');
    await settle();
    expect(api.calls.some((c) => c.startsWith('rewriteUnchanged'))).toBe(false);
  });

  it('tells the user to close the game when the rewrite is blocked', async () => {
    const api = fakeApi({ rewriteUnchanged: () => commandError('gameRunning') });
    await new Controller(root, api).start();
    click('rewrite-unchanged');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('Feche o theHunter');
  });

  it('ignores the rewrite action when there is no file to choose', async () => {
    const api = fakeApi({
      diagnostics: () => Promise.resolve({ ...foundDiagnostics, files: [] }),
    });
    await new Controller(root, api).start();
    click('rewrite-unchanged');
    await settle();
    expect(api.calls.some((c) => c.startsWith('rewriteUnchanged'))).toBe(false);
  });

  it('shows an error when loading fails', async () => {
    const api = fakeApi({ diagnostics: () => Promise.reject(new Error('sem conexão com o app')) });
    await new Controller(root, api).start();
    expect(root.querySelector('[role="alert"]')?.textContent).toContain('sem conexão com o app');
  });

  it('ignores clicks outside any action', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    root.click();
    click('acao-desconhecida');
    await settle();
    expect(api.calls).toEqual(['diagnostics', 'mapReserves']);
  });
});

describe('Controller: liberar o mapa', () => {
  async function openUnlock(api = fakeApi()): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click('show-unlock');
    await settle();
    return api;
  }

  it('analyses the progress when the screen opens and highlights Liberar mapa', async () => {
    const api = await openUnlock();
    expect(api.calls).toContain('zonesPreview');
    expect(root.textContent).toContain('3.511 lugares novos no mapa');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Liberar mapa');
  });

  it('unlocks only the checked reserves after confirmation', async () => {
    const api = await openUnlock();
    const second = root.querySelector<HTMLInputElement>('.reserve-pick[value="13"]');
    if (second) second.checked = false;
    click('unlock-run');
    await settle();
    expect(api.calls).toContain('unlockZones:2');
    expect(root.querySelector('.notice-ok')?.textContent).toContain('3.511 lugares liberados em 1 reservas');
  });

  it('does nothing when no reserve is checked or the user declines', async () => {
    const api = await openUnlock(fakeApi({ confirm: () => Promise.resolve(false) }));
    click('unlock-run');
    await settle();
    expect(api.calls.some((c) => c.startsWith('unlockZones'))).toBe(false);

    const other = await openUnlock();
    for (const box of root.querySelectorAll<HTMLInputElement>('.reserve-pick')) box.checked = false;
    click('unlock-run');
    await settle();
    expect(other.calls.some((c) => c.startsWith('unlockZones'))).toBe(false);
  });

  it('tells the player to close the game when the unlock is blocked', async () => {
    await openUnlock(fakeApi({ unlockZones: () => commandError('gameRunning') }));
    click('unlock-run');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('Feche o theHunter');
  });

  it('explains when the game is not installed', async () => {
    await new Controller(root, fakeApi({ zonesPreview: () => commandError('gameNotFound') })).start();
    click('show-unlock');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('Não encontramos o jogo instalado');
  });

  it('goes back home', async () => {
    await openUnlock();
    click('show-home');
    await settle();
    expect(root.textContent).toContain('Bem-vindo, caçador!');
  });
});

describe('Controller: mapa', () => {
  async function openMap(api = fakeApi()): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click('show-map');
    await settle();
    return api;
  }

  it('loads the reserves and the first reserve map, and keeps Mapa highlighted', async () => {
    const api = await openMap();
    expect(api.calls).toContain('mapReserves');
    expect(api.calls).toContain('reserveMap:2');
    expect(root.querySelector('#map-canvas')).not.toBeNull();
    expect(root.querySelector('main.wide')).not.toBeNull();
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Mapa');
  });

  it('loads another reserve when the player picks it', async () => {
    const api = await openMap();
    root.querySelector<HTMLElement>('#map-reserve [data-reserve="0"]')?.click();
    await settle();
    expect(api.calls).toContain('reserveMap:0');
  });

  it('opens the unlock screen from the map', async () => {
    await openMap();
    click('show-unlock');
    await settle();
    expect(root.textContent).toContain('Liberar o mapa');
  });

  it('explains when no reserve has data', async () => {
    await openMap(fakeApi({ mapReserves: () => Promise.resolve([]) }));
    expect(root.textContent).toContain('Ainda não há nenhuma reserva com dados');
    expect(root.querySelector('#map-canvas')).toBeNull();
  });

  it('explains when the game is not installed', async () => {
    await openMap(fakeApi({ mapReserves: () => commandError('gameNotFound') }));
    expect(root.querySelector('.notice-error')?.textContent).toContain('Não encontramos o jogo instalado');
  });
});

describe('Controller: animais', () => {
  async function open(api = fakeApi()): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click('show-animals');
    await settle();
    return api;
  }

  it('opens the wiki on the first reserve and highlights Animais', async () => {
    const api = await open();
    expect(api.calls).toContain('animals');
    expect(root.querySelectorAll('.species-card')).toHaveLength(2);
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Animais');
  });

  it('opens a species sheet, jumps to another reserve and closes it', async () => {
    const api = await open();
    const before = api.calls.length;
    click('pick-species', { species: 'moose' });
    expect(root.querySelector('.species-detail')?.textContent).toContain('Alce');
    click('open-animal', { reserve: '8', species: 'moose' });
    expect(root.querySelector('.species-detail')?.textContent).toContain('Cuatro Colinas');
    click('pick-species', { species: '' });
    expect(root.querySelector('.species-detail')).toBeNull();
    click('pick-animal-reserve', { reserve: '2' });
    expect(root.querySelectorAll('.species-card')).toHaveLength(2);
    expect(api.calls).toHaveLength(before);
  });

  it('filters the species by name after the screen is drawn', async () => {
    await open();
    const search = root.querySelector<HTMLInputElement>('#animal-search');
    if (!search) throw new Error('busca ausente');
    search.value = 'urso';
    search.dispatchEvent(new Event('input'));
    const shown = [...root.querySelectorAll<HTMLElement>('#species-grid li')].filter((li) => !li.hidden);
    expect(shown).toHaveLength(1);
  });

  it('shows a plain message when the animals cannot be read', async () => {
    await open(fakeApi({ animals: () => commandError('map') }));
    expect(root.querySelector('[role="alert"]')).not.toBeNull();
  });
});

describe('Controller: perfil e missões', () => {
  async function open(action: string, api = fakeApi()): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click(action);
    await settle();
    return api;
  }

  it('opens the profile and highlights Perfil', async () => {
    const api = await open('show-profile');
    expect(api.calls).toContain('profile');
    expect(root.textContent).toContain('Nível 60');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Perfil');
  });

  it('filters the achievements after the screen is drawn', async () => {
    await open('show-profile');
    root.querySelector<HTMLElement>('[data-ach-filter="done"]')?.click();
    const shown = [...root.querySelectorAll<HTMLElement>('.ach')].filter((card) => !card.hidden);
    expect(shown).toHaveLength(1);
  });

  it('opens the missions with the active ones first and highlights Missões', async () => {
    const api = await open('show-missions');
    expect(api.calls).toContain('missions');
    expect(api.calls).toContain('mapReserves');
    expect(root.querySelectorAll('.mission')).toHaveLength(2);
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Missões');
    expect(root.querySelectorAll('.reserve-card img')).toHaveLength(1);
  });

  it('switches the reserve and the filter without asking the app again', async () => {
    const api = await open('show-missions');
    const before = api.calls.length;
    click('mission-filter', { filter: 'all' });
    expect(root.querySelectorAll('.mission')).toHaveLength(4);
    click('mission-filter', { filter: 'invalido' });
    expect(root.querySelectorAll('.mission')).toHaveLength(2);
    click('pick-mission-reserve', { reserve: '0' });
    expect(root.querySelector('.reserve-summary')?.textContent).toContain('Hirschfelden');
    expect(api.calls).toHaveLength(before);
  });

  it('still shows the missions when the covers cannot be loaded', async () => {
    const api = fakeApi({ mapReserves: () => Promise.reject(new Error('sem jogo')) });
    await open('show-missions', api);
    expect(root.querySelectorAll('.mission')).toHaveLength(2);
    expect(root.querySelector('.reserve-card img')).toBeNull();
  });

  it('shows a plain message when the progress cannot be read', async () => {
    const api = fakeApi({ missions: () => commandError('career') });
    await open('show-missions', api);
    expect(root.querySelector('[role="alert"]')?.textContent).toContain('Não foi possível ler o seu perfil');
  });
});

describe('describeError', () => {
  it('handles command errors, Error objects and plain values', () => {
    expect(describeError({ code: 'noSaveSelected', message: 'x' })).toContain('Selecione primeiro');
    expect(describeError(new Error('falhou'))).toBe('Algo deu errado: falhou');
    expect(describeError('texto')).toBe('Algo deu errado: texto');
  });
});

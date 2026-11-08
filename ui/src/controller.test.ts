import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { Controller, describeError } from './controller';
import type { Api } from './api';
import type { EditRequest, PurchaseStatus } from './types';
import { ACTIVE_LICENSE, FREE_LICENSE, commandError, fakeApi, foundDiagnostics } from './test-data';

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
    expect(api.calls).toEqual(['license', 'checkPurchase', 'diagnostics', 'mapReserves']);
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
    expect(api.calls).toEqual(['license', 'checkPurchase', 'diagnostics', 'mapReserves']);
  });
});

describe('Controller: liberar o mapa', () => {
  const premium = (): Partial<Api> => ({ license: () => Promise.resolve(ACTIVE_LICENSE) });

  async function openUnlock(api = fakeApi(premium())): Promise<ReturnType<typeof fakeApi>> {
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

  it('shows a free user everything but does not write: the button only explains Premium', async () => {
    const api = await openUnlock(fakeApi());
    expect(root.querySelector('.premium-banner')).not.toBeNull();
    expect(root.querySelector('.tab[data-premium="true"]')?.textContent).toBe('Liberar mapa');
    click('premium-info');
    await settle();
    expect(root.textContent).toContain('Escolha o seu plano');
    click('unlock-run');
    await settle();
    expect(api.calls.some((c) => c.startsWith('unlockZones'))).toBe(false);
  });

  it('does not mark Liberar mapa as Premium when the license is active', async () => {
    await openUnlock();
    expect(root.querySelector('.tab[data-premium="true"]')).toBeNull();
  });

  it('unlocks only the checked reserves after confirmation', async () => {
    const api = await openUnlock();
    click('toggle-reserve', { reserve: '13' });
    expect(root.textContent).toContain('Serão liberados 3.511 lugares em 1 reservas');
    click('unlock-run');
    await settle();
    expect(api.requests).toEqual([{ reserves: [2], species: null, needs: null }]);
    expect(root.querySelector('.notice-ok')?.textContent).toContain('3.511 lugares liberados em 1 reservas');
  });

  it('unlocks by need', async () => {
    const api = await openUnlock();
    click('unlock-mode', { mode: 'need' });
    click('toggle-need', { need: '2' });
    expect(root.textContent).toContain('Serão liberados 2.540 lugares em 2 reservas');
    click('unlock-run');
    await settle();
    expect(api.requests).toEqual([{ reserves: [2, 13], species: null, needs: [2] }]);
  });

  it('unlocks by animal and can go back to everything', async () => {
    const api = await openUnlock();
    click('unlock-mode', { mode: 'animal' });
    click('toggle-species', { species: 'brown_bear' });
    expect(root.textContent).toContain('Serão liberados 111 lugares em 1 reservas');
    click('toggle-species', { species: 'brown_bear' });
    expect(root.textContent).toContain('Nada escolhido ainda');
    click('toggle-species', { species: 'brown_bear' });
    click('unlock-run');
    await settle();
    expect(api.requests).toEqual([{ reserves: [2, 13], species: ['brown_bear'], needs: null }]);
    click('unlock-mode', { mode: 'qualquer-coisa' });
    expect(root.textContent).toContain('Tudo');
    expect(root.querySelector('[data-mode="all"]')?.getAttribute('aria-pressed')).toBe('true');
  });

  it('does nothing when nothing is chosen or the user declines', async () => {
    const declined = await openUnlock(fakeApi({ ...premium(), confirm: () => Promise.resolve(false) }));
    click('unlock-run');
    await settle();
    expect(declined.requests).toEqual([]);

    const other = await openUnlock();
    click('unlock-mode', { mode: 'animal' });
    click('unlock-run');
    await settle();
    expect(other.requests).toEqual([]);
  });

  it('tells the player to close the game when the unlock is blocked', async () => {
    await openUnlock(fakeApi({ ...premium(), unlockZones: () => commandError('gameRunning') }));
    click('unlock-run');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('Feche o theHunter');
  });

  it('explains that the function is Premium if the app refuses it', async () => {
    await openUnlock(fakeApi({ ...premium(), unlockZones: () => commandError('premiumRequired') }));
    click('unlock-run');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('do Premium');
  });

  it('keeps working when the license cannot be read', async () => {
    const api = fakeApi({ license: () => Promise.reject(new Error('sem licença')) });
    await new Controller(root, api).start();
    expect(root.textContent).toContain('Bem-vindo, caçador!');
    expect(root.querySelector('.tab[data-premium="true"]')).not.toBeNull();
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

describe('Controller: idioma', () => {
  it('tells the app the language at the start', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    expect(api.languages).toEqual(['pt-BR']);
  });

  it('switches the whole screen to English and tells the app', async () => {
    window.localStorage.clear();
    const api = fakeApi();
    await new Controller(root, api).start();
    click('show-diagnostics');
    await settle();
    click('set-language', { locale: 'en' });
    await settle();
    expect(api.languages).toEqual(['pt-BR', 'en']);
    expect(root.textContent).toContain('Settings');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Settings');
    expect(root.querySelector('[data-locale="en"]')?.getAttribute('aria-pressed')).toBe('true');
    expect(window.localStorage.getItem('great-cotw-one.locale')).toBe('en');
    expect(document.documentElement.lang).toBe('en');
  });

  it('refreshes the content that comes from the app, in the new language', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('show-animals');
    await settle();
    const before = api.calls.filter((c) => c === 'animals').length;
    click('set-language', { locale: 'es' });
    await settle();
    expect(api.calls.filter((c) => c === 'animals').length).toBe(before + 1);
    expect(root.textContent).toContain('Animales');
  });

  it('ignores the language that is already chosen and unknown ones', async () => {
    const api = fakeApi();
    await new Controller(root, api).start();
    click('set-language', { locale: 'pt-BR' });
    click('set-language', { locale: 'fr' });
    click('set-language');
    await settle();
    expect(api.languages).toEqual(['pt-BR']);
  });

  it('keeps going if the app cannot be told the language', async () => {
    const api = fakeApi({ setLanguage: () => Promise.reject(new Error('sem app')) });
    await new Controller(root, api).start();
    click('set-language', { locale: 'en' });
    await settle();
    expect(root.textContent).toContain('Welcome, hunter!');
  });
});

describe('Controller: licença', () => {
  async function openSettings(api = fakeApi()): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click('show-premium');
    await settle();
    return api;
  }

  function paste(text: string): void {
    const field = root.querySelector<HTMLTextAreaElement>('#license-key');
    if (!field) throw new Error('campo ausente');
    field.value = text;
  }

  it('shows the license card on the Premium page and highlights Premium', async () => {
    await openSettings();
    expect(root.textContent).toContain('Licença Premium');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Premium');
  });

  it('explains that the store is not open yet when a plan button is pressed', async () => {
    const api = await openSettings(fakeApi({ startPurchase: () => commandError('storeClosed') }));
    click('buy', { plan: 'yearly' });
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('loja ainda não abriu');
    expect(root.querySelector('.purchase-waiting')).toBeNull();
    expect(api.calls.some((c) => c.startsWith('activateLicense'))).toBe(false);
  });

  it('takes a free user from the unlock screen to the Premium page', async () => {
    await new Controller(root, fakeApi()).start();
    click('show-unlock');
    await settle();
    click('show-premium');
    await settle();
    expect(root.textContent).toContain('Great COTW One Premium');
  });

  it('sends a free user who tries to save to the Premium page', async () => {
    await new Controller(root, fakeApi()).start();
    click('show-unlock');
    await settle();
    click('premium-info');
    await settle();
    expect(root.textContent).toContain('Escolha o seu plano');
  });

  it('activates a pasted key and unlocks Premium', async () => {
    let active = false;
    const keys: string[] = [];
    await openSettings(
      fakeApi({
        license: () => Promise.resolve(active ? ACTIVE_LICENSE : FREE_LICENSE),
        activateLicense: (key) => {
          active = true;
          keys.push(key);
          return Promise.resolve(ACTIVE_LICENSE);
        },
      }),
    );
    paste('  GCO1.abc.def \n');
    click('activate-license');
    await settle();
    expect(keys).toEqual(['GCO1.abc.def']);
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Premium ativado');
    expect(root.querySelector('.tab[data-premium="true"]')).toBeNull();
  });

  it('asks for a key when the field is empty', async () => {
    const api = await openSettings();
    click('activate-license');
    await settle();
    expect(api.calls.some((c) => c.startsWith('activateLicense'))).toBe(false);
    expect(root.querySelector('.notice-error')?.textContent).toContain('Cole primeiro a chave');
  });

  it('explains in plain words when the key is refused', async () => {
    await openSettings(fakeApi({ activateLicense: () => commandError('licenseSignature') }));
    paste('GCO1.x.y');
    click('activate-license');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('não é válida');
  });

  it('removes the key after confirmation and goes back to free', async () => {
    let active = true;
    await openSettings(
      fakeApi({
        license: () => Promise.resolve(active ? ACTIVE_LICENSE : FREE_LICENSE),
        removeLicense: () => {
          active = false;
          return Promise.resolve(FREE_LICENSE);
        },
      }),
    );
    click('remove-license');
    await settle();
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Chave removida');
    expect(root.querySelector('.tab[data-premium="true"]')).not.toBeNull();
  });

  it('keeps the key when the player declines to remove it', async () => {
    const api = await openSettings(fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE), confirm: () => Promise.resolve(false) }));
    click('remove-license');
    await settle();
    expect(api.calls).not.toContain('removeLicense');
  });

  it('shows a plain message when removing fails', async () => {
    await openSettings(fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE), removeLicense: () => commandError('licenseSave') }));
    click('remove-license');
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('guardar a chave');
  });
});

describe('Controller: editar animais no mapa', () => {
  const premium = (): Partial<Api> => ({ license: () => Promise.resolve(ACTIVE_LICENSE) });

  async function openGroup(api = fakeApi(premium())): Promise<ReturnType<typeof fakeApi>> {
    await new Controller(root, api).start();
    click('show-map');
    await settle();
    const select = root.querySelector<HTMLSelectElement>('#map-grp');
    if (!select) throw new Error('grupo');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    root.querySelector<HTMLElement>('#map-anim [data-animal="0"]')?.click();
    await settle();
    return api;
  }

  function weigh(value: string): void {
    const field = root.querySelector<HTMLInputElement>('#edit-weight');
    if (!field) throw new Error('peso');
    field.value = value;
    field.dispatchEvent(new Event('input'));
  }

  it('confirms, saves the edit with a backup and reloads the map with a notice', async () => {
    const requests: EditRequest[] = [];
    const api = await openGroup(
      fakeApi({
        ...premium(),
        editAnimals: (request) => {
          requests.push(request);
          return Promise.resolve({ animals: 1, backup: 'backup-x' });
        },
      }),
    );
    weigh('600');
    root.querySelector<HTMLElement>('#edit-save')?.click();
    await settle();
    expect(requests).toEqual([{ reserve: 2, species: 'moose', changes: [{ group: 1, index: 0, weight: 600 }] }]);
    expect(api.calls.filter((c) => c.startsWith('reserveMap')).length).toBeGreaterThan(1);
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Animal editado');
  });

  it('does not save when the player declines the confirmation', async () => {
    const requests: EditRequest[] = [];
    await openGroup(
      fakeApi({
        ...premium(),
        confirm: () => Promise.resolve(false),
        editAnimals: (request) => {
          requests.push(request);
          return Promise.resolve({ animals: 1, backup: '' });
        },
      }),
    );
    weigh('600');
    root.querySelector<HTMLElement>('#edit-save')?.click();
    await settle();
    expect(requests).toEqual([]);
    expect(root.querySelector('#map-edit')?.hasAttribute('hidden')).toBe(false);
  });

  it('shows the reason inside the panel when the app refuses the edit', async () => {
    await openGroup(fakeApi({ ...premium(), editAnimals: () => commandError('editorOutOfRange') }));
    weigh('600');
    root.querySelector<HTMLElement>('#edit-save')?.click();
    await settle();
    expect(root.querySelector('#edit-error')?.textContent).toContain('fora do que o jogo tem');
  });

  it('tells the player to close the game when it is open', async () => {
    await openGroup(fakeApi({ ...premium(), editAnimals: () => commandError('gameRunning') }));
    weigh('600');
    root.querySelector<HTMLElement>('#edit-save')?.click();
    await settle();
    expect(root.querySelector('#edit-error')?.textContent).toContain('Feche o theHunter');
  });

  it('sends a free user to the Premium page when they press edit', async () => {
    await new Controller(root, fakeApi()).start();
    click('show-map');
    await settle();
    const select = root.querySelector<HTMLSelectElement>('#map-grp');
    if (!select) throw new Error('grupo');
    select.value = '1';
    select.dispatchEvent(new Event('change'));
    root.querySelector<HTMLElement>('#map-anim [data-animal="0"]')?.click();
    await settle();
    expect(root.textContent).toContain('Escolha o seu plano');
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

  it('opens the equipment page, switches tabs, opens a weapon and fills the icons', async () => {
    const api = fakeApi({
      equipmentIcons: (ids) => {
        api.calls.push(`equipmentIcons:${ids.join(',')}`);
        return Promise.resolve({ equipment_weapon_ba_rifle_270_01: 'data:image/png;base64,AAAA' });
      },
    });
    await new Controller(root, api).start();
    click('show-equipment');
    await settle();
    expect(api.calls).toContain('equipment');
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Equipamentos');
    expect(api.calls.some((c) => c.startsWith('equipmentIcons:'))).toBe(true);
    expect(root.querySelector('img[src^="data:image/png"]')).not.toBeNull();
    click('equip-kind', { kind: 'ammo' });
    expect(root.textContent).toContain('.270 Soft-Point Bullet');
    click('equip-own', { own: 'missing' });
    expect(root.textContent).not.toContain('Soft-Point');
    click('equip-kind', { kind: 'weapon' });
    click('equip-open', { id: 'equipment_weapon_ba_rifle_270_01' });
    expect(root.textContent).toContain('Munições que servem');
    click('equip-close');
    click('equip-tab', { tab: 'ranking' });
    click('equip-rank', { rank: 'accuracy' });
    expect(root.querySelectorAll('.eq-rank li')).toHaveLength(1);
    click('equip-tab', { tab: 'loadouts' });
    expect(root.textContent).toContain('Alces');
  });

  function input(selector: string): HTMLInputElement {
    const found = root.querySelector<HTMLInputElement>(selector);
    if (!found) throw new Error(`sem ${selector}`);
    return found;
  }

  async function openEquipment(api: Api): Promise<void> {
    await new Controller(root, api).start();
    click('show-equipment');
    await settle();
  }

  it('sends free users to the Premium page instead of writing equipment', async () => {
    const api = fakeApi();
    await openEquipment(api);
    click('equip-give', { id: 'equipment_weapon_longbow_01' });
    await settle();
    expect(api.calls.some((c) => c.startsWith('editEquipment'))).toBe(false);
    expect(root.querySelector('.tab[aria-current="page"]')?.textContent).toBe('Premium');
  });

  it('gives an item after the player confirms, then reloads the equipment', async () => {
    const api = fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE) });
    await openEquipment(api);
    click('equip-give', { id: 'equipment_weapon_longbow_01' });
    await settle();
    expect(api.calls).toContain('editEquipment:[{"action":"give","id":"equipment_weapon_longbow_01"}]');
    expect(api.calls.filter((c) => c === 'equipment')).toHaveLength(2);
    expect(root.textContent).toContain('Pronto! Abra o jogo');
  });

  it('does nothing when the player declines the confirmation', async () => {
    const api = fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE), confirm: () => Promise.resolve(false) });
    await openEquipment(api);
    click('equip-unlock', { id: 'equipment_ammo_270_pt_01' });
    await settle();
    expect(api.calls.some((c) => c.startsWith('editEquipment'))).toBe(false);
  });

  it('sets the ammo amount and the cash from the typed values and rejects bad ones', async () => {
    const api = fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE) });
    await openEquipment(api);
    click('equip-kind', { kind: 'ammo' });
    const amount = root.querySelector<HTMLInputElement>('[data-eq-amount="equipment_ammo_270_sp_01"]');
    if (!amount) throw new Error('sem campo de quantidade');
    amount.value = '0';
    click('equip-set-amount', { id: 'equipment_ammo_270_sp_01' });
    await settle();
    expect(root.textContent).toContain('quantidade inteira de 1 a 999');
    input('[data-eq-amount="equipment_ammo_270_sp_01"]').value = '500';
    click('equip-set-amount', { id: 'equipment_ammo_270_sp_01' });
    await settle();
    expect(api.calls).toContain('editEquipment:[{"action":"setAmount","id":"equipment_ammo_270_sp_01","amount":500}]');
    const cash = root.querySelector<HTMLInputElement>('#eq-cash');
    if (!cash) throw new Error('sem campo de dinheiro');
    cash.value = '99999999';
    click('equip-set-cash');
    await settle();
    expect(root.textContent).toContain('valor inteiro de 0 a 10.000.000');
    input('#eq-cash').value = '50000';
    click('equip-set-cash');
    await settle();
    expect(api.calls).toContain('editEquipment:[{"action":"setCash","cash":50000}]');
  });

  it('shows the error when the write is refused', async () => {
    const api = fakeApi({
      license: () => Promise.resolve(ACTIVE_LICENSE),
      editEquipment: () => commandError('gameRunning'),
    });
    await openEquipment(api);
    click('equip-give', { id: 'equipment_weapon_longbow_01' });
    await settle();
    expect(root.querySelector('.notice-error')).not.toBeNull();
  });

  it('picks a reserve and an animal in the gear advisor and shows the suggestions', async () => {
    const api = fakeApi();
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    expect(api.calls).toContain('animals');
    expect(root.textContent).toContain('Escolha onde vai caçar');
    click('adv-reserve', { reserve: '2' });
    const species = root.querySelector<HTMLElement>('[data-action="adv-species"]');
    if (!species) throw new Error('sem animais');
    click('adv-species', { species: species.dataset['species'] ?? '' });
    await settle();
    expect(api.calls.some((c) => c.startsWith('gearAdvice:'))).toBe(true);
    expect(root.textContent).toContain('Vale comprar');
  });

  it('builds the kit for a whole reserve and remembers the weight limit the player types', async () => {
    const api = fakeApi();
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    click('adv-mode', { mode: 'reserve' });
    click('adv-reserve', { reserve: '2' });
    await settle();
    expect(api.calls).toContain('reserveKit:2');
    expect(root.textContent).toContain('Armas para levar');
    const limit = root.querySelector<HTMLInputElement>('#kit-limit');
    if (!limit) throw new Error('sem campo de limite');
    limit.value = '30';
    limit.dispatchEvent(new Event('input'));
    expect(root.querySelector('#kit-limit-note')?.textContent).toContain('sobram');
    expect(localStorage.getItem('gco.weightLimit')).toBe('30');
    limit.value = '';
    limit.dispatchEvent(new Event('input'));
    expect(localStorage.getItem('gco.weightLimit')).toBeNull();
    click('adv-mode', { mode: 'animal' });
    expect(root.querySelector('[data-action="adv-mode"][aria-pressed="true"]')?.getAttribute('data-mode')).toBe('animal');
  });

  it('shows an error when the kit cannot be read', async () => {
    const api = fakeApi({ reserveKit: () => commandError('noSaveSelected') });
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    click('adv-mode', { mode: 'reserve' });
    click('adv-reserve', { reserve: '2' });
    await settle();
    expect(root.querySelector('.notice-error')).not.toBeNull();
  });

  it('gives the suggested set after the player confirms and refreshes the suggestions', async () => {
    const api = fakeApi({ license: () => Promise.resolve(ACTIVE_LICENSE) });
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    click('adv-reserve', { reserve: '2' });
    click('adv-species', { species: 'moose' });
    await settle();
    click('adv-give', { weapon: 'equipment_weapon_longbow_01', ammo: 'equipment_ammo_270_pt_01' });
    await settle();
    expect(api.calls).toContain(
      'editEquipment:[{"action":"give","id":"equipment_weapon_longbow_01"},{"action":"give","id":"equipment_ammo_270_pt_01"}]',
    );
    expect(api.calls.filter((c) => c === 'gearAdvice:moose')).toHaveLength(2);
  });

  it('shows an error when the suggestions cannot be read', async () => {
    const api = fakeApi({ gearAdvice: () => commandError('noSaveSelected') });
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    click('adv-reserve', { reserve: '2' });
    click('adv-species', { species: 'moose' });
    await settle();
    expect(root.querySelector('.notice-error')).not.toBeNull();
  });

  it('keeps the advisor usable when the animals cannot be loaded', async () => {
    const api = fakeApi({ animals: () => commandError('gameNotFound') });
    await openEquipment(api);
    click('equip-tab', { tab: 'advisor' });
    await settle();
    expect(root.querySelector('.notice-error')).not.toBeNull();
  });

  it('keeps the equipment page working when the icons cannot be read', async () => {
    const api = fakeApi({ equipmentIcons: () => Promise.reject(new Error('sem jogo')) });
    await new Controller(root, api).start();
    click('show-equipment');
    await settle();
    expect(root.textContent).toContain('.270 Bolt-Action Rifle');
    expect(root.querySelector('img[src^="data:image/png"]')).toBeNull();
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

describe('Controller: compra pelo app', () => {
  const waiting: PurchaseStatus = { state: 'waiting', plan: 'yearly', license: FREE_LICENSE };
  const paid: PurchaseStatus = { state: 'activated', plan: 'yearly', license: ACTIVE_LICENSE };

  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function purchaseApi(statuses: PurchaseStatus[]): { api: ReturnType<typeof fakeApi>; started: [string, string][]; cancelled: number[] } {
    const started: [string, string][] = [];
    const cancelled: number[] = [];
    let step = 0;
    const api = fakeApi({
      startPurchase: (plan, token) => {
        started.push([plan, token]);
        return Promise.resolve();
      },
      checkPurchase: () => {
        const status = statuses[Math.min(step, statuses.length - 1)];
        step += 1;
        return Promise.resolve(status ?? { state: 'none', plan: null, license: FREE_LICENSE });
      },
      cancelPurchase: () => {
        cancelled.push(1);
        return Promise.resolve();
      },
    });
    return { api, started, cancelled };
  }

  async function openPremium(api: Api): Promise<void> {
    await new Controller(root, api).start();
    click('show-premium');
    await settle();
  }

  it('opens the payment, waits, and turns Premium on by itself when it is confirmed', async () => {
    // 1ª chamada: a abertura do app (nada em andamento); depois, esperando; por fim, pago.
    const none: PurchaseStatus = { state: 'none', plan: null, license: FREE_LICENSE };
    const { api, started } = purchaseApi([none, waiting, waiting, paid]);
    await openPremium(api);

    click('buy', { plan: 'yearly' });
    await settle();
    expect(started).toHaveLength(1);
    expect(started[0]?.[0]).toBe('yearly');
    expect(started[0]?.[1]).toMatch(/^[A-Za-z0-9_-]{24}$/);
    expect(root.querySelector('.purchase-waiting')?.textContent).toContain('Esperando a confirmação');
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Abrimos o pagamento');
    expect(root.querySelector<HTMLButtonElement>('[data-action="buy"]')?.disabled).toBe(true);

    // Apertar de novo enquanto espera não abre outro pagamento.
    click('buy', { plan: 'monthly' });
    await settle();
    expect(started).toHaveLength(1);

    await vi.advanceTimersByTimeAsync(4000);
    expect(root.querySelector('.purchase-waiting')).not.toBeNull();

    await vi.advanceTimersByTimeAsync(8000);
    await settle();
    expect(root.querySelector('.purchase-waiting')).toBeNull();
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Pagamento confirmado');
    expect(root.textContent).toContain('Premium');
    const calls = api.calls.filter((c) => c === 'checkPurchase').length;
    await vi.advanceTimersByTimeAsync(20_000);
    expect(api.calls.filter((c) => c === 'checkPurchase').length).toBe(calls);
  });

  it('keeps waiting when the internet drops for a moment', async () => {
    let answers = 0;
    const api = fakeApi({
      checkPurchase: () => {
        answers += 1;
        if (answers === 1) return Promise.resolve({ state: 'none', plan: null, license: FREE_LICENSE });
        return answers === 2 ? commandError('storeOffline') : Promise.resolve(paid);
      },
    });
    await openPremium(api);
    click('buy', { plan: 'monthly' });
    await settle();
    await vi.advanceTimersByTimeAsync(4000);
    expect(root.querySelector('.purchase-waiting')).not.toBeNull();
    expect(root.querySelector('.notice-error')).toBeNull();
    await vi.advanceTimersByTimeAsync(4000);
    await settle();
    expect(root.querySelector('.notice-ok')?.textContent).toContain('Pagamento confirmado');
  });

  it('lets the player stop waiting', async () => {
    const none: PurchaseStatus = { state: 'none', plan: null, license: FREE_LICENSE };
    const { api, cancelled } = purchaseApi([none, waiting]);
    await openPremium(api);
    click('buy', { plan: 'yearly' });
    await settle();
    click('cancel-purchase');
    await settle();
    expect(cancelled).toHaveLength(1);
    expect(root.querySelector('.purchase-waiting')).toBeNull();
    expect(root.querySelector<HTMLButtonElement>('[data-action="buy"]')?.disabled).toBe(false);
    const calls = api.calls.filter((c) => c === 'checkPurchase').length;
    await vi.advanceTimersByTimeAsync(20_000);
    expect(api.calls.filter((c) => c === 'checkPurchase').length).toBe(calls);
  });

  it('picks the purchase back up when the app is opened again', async () => {
    const { api } = purchaseApi([waiting]);
    await openPremium(api);
    expect(root.querySelector('.purchase-waiting')).not.toBeNull();
  });

  it('shows a clear message when the store cannot be reached to start', async () => {
    const api = fakeApi({ startPurchase: () => commandError('storeOffline') });
    await openPremium(api);
    click('buy', { plan: 'yearly' });
    await settle();
    expect(root.querySelector('.notice-error')?.textContent).toContain('loja');
    expect(root.querySelector('.purchase-waiting')).toBeNull();
  });
});

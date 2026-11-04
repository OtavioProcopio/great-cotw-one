import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';

import { isCommandError, tauriApi } from './api';

interface Call {
  command: string;
  args: unknown;
}

/** Instala um mock do IPC que registra as chamadas e responde com `replies[comando]`. */
function mock(replies: Record<string, unknown> = {}): Call[] {
  const calls: Call[] = [];
  mockIPC((command, args) => {
    calls.push({ command, args });
    return replies[command] ?? null;
  });
  return calls;
}

afterEach(() => {
  clearMocks();
});

describe('tauriApi', () => {
  // Os nomes abaixo precisam ser exatamente os comandos registrados em `src-tauri/src/lib.rs`.
  it('calls the registered command names with the expected arguments', async () => {
    const calls = mock();
    await tauriApi.diagnostics();
    await tauriApi.selectSaveDir('C:\\Saves\\1');
    await tauriApi.selectGameDir('D:\\Jogo');
    await tauriApi.resetChoices();
    await tauriApi.history();
    await tauriApi.rewriteUnchanged('animal_population_2');
    await tauriApi.mapReserves();
    await tauriApi.reserveMap(2);
    await tauriApi.zonesPreview();
    await tauriApi.unlockZones({ reserves: [2, 13], species: null, needs: [2] });
    await tauriApi.license();
    await tauriApi.activateLicense('GCO1.a.b');
    await tauriApi.removeLicense();
    await tauriApi.restore('C:\\Backups\\1');
    expect(calls).toEqual([
      { command: 'diagnostics', args: {} },
      { command: 'select_save_dir', args: { path: 'C:\\Saves\\1' } },
      { command: 'select_game_dir', args: { path: 'D:\\Jogo' } },
      { command: 'reset_choices', args: {} },
      { command: 'history', args: {} },
      { command: 'rewrite_unchanged', args: { file: 'animal_population_2' } },
      { command: 'map_reserves', args: {} },
      { command: 'reserve_map', args: { reserve: 2 } },
      { command: 'zones_preview', args: {} },
      { command: 'unlock_zones', args: { request: { reserves: [2, 13], species: null, needs: [2] } } },
      { command: 'license', args: {} },
      { command: 'activate_license', args: { key: 'GCO1.a.b' } },
      { command: 'remove_license', args: {} },
      { command: 'restore_backup', args: { backup: 'C:\\Backups\\1' } },
    ]);
  });

  it('returns the chosen folder, or null when the dialog is cancelled', async () => {
    mock({ 'plugin:dialog|open': 'C:\\Pasta' });
    expect(await tauriApi.pickFolder('Escolha')).toBe('C:\\Pasta');
    mock({ 'plugin:dialog|open': null });
    expect(await tauriApi.pickFolder('Escolha')).toBeNull();
  });

  it('asks for confirmation through the dialog plugin', async () => {
    // `ask` usa o comando `message` e devolve o rótulo do botão apertado.
    const calls = mock({ 'plugin:dialog|message': 'Gravar' });
    expect(await tauriApi.confirm('Gravar?', 'Gravar')).toBe(true);
    expect(calls[0]?.command).toBe('plugin:dialog|message');
    expect(JSON.stringify(calls[0]?.args)).toContain('"OkCancelCustom":["Gravar","Cancelar"]');
    mock({ 'plugin:dialog|message': 'Cancelar' });
    expect(await tauriApi.confirm('Gravar?', 'Gravar')).toBe(false);
  });
});

describe('isCommandError', () => {
  it('accepts objects with string code and message only', () => {
    expect(isCommandError({ code: 'a', message: 'b' })).toBe(true);
    expect(isCommandError({ code: 'a' })).toBe(false);
    expect(isCommandError({ code: 1, message: 'b' })).toBe(false);
    expect(isCommandError(null)).toBe(false);
    expect(isCommandError('texto')).toBe(false);
  });
});

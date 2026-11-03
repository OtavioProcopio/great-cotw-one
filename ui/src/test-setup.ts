// O jsdom não desenha em canvas: devolve um contexto falso que aceita qualquer chamada.
import { vi } from 'vitest';

export const drawCalls: string[] = [];

function fakeContext(): CanvasRenderingContext2D {
  return new Proxy({} as CanvasRenderingContext2D, {
    get: (_target, name: string) => () => {
      drawCalls.push(name);
    },
    set: () => true,
  });
}

vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => fakeContext());

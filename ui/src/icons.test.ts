import { describe, expect, it } from 'vitest';

import { icon, iconPath } from './icons';

describe('icons', () => {
  it('draws a decorative svg with the requested size', () => {
    const markup = icon('map', 32);
    expect(markup).toContain('width="32"');
    expect(markup).toContain('aria-hidden="true"');
    expect(markup).toContain(`d="${iconPath('map')}"`);
  });

  it('uses 20 pixels by default and keeps the text content empty', () => {
    const root = document.createElement('div');
    root.innerHTML = icon('leaf');
    expect(root.querySelector('svg')?.getAttribute('width')).toBe('20');
    expect(root.textContent).toBe('');
  });
});

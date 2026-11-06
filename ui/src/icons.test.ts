import { describe, expect, it } from 'vitest';

import { animalIcon, icon } from './icons';

describe('animalIcon', () => {
  it('picks an icon from the class the game gives to the animal', () => {
    expect(animalIcon('deer')).toBe('deer');
    expect(animalIcon('duck')).toBe('bird');
    expect(animalIcon('wolf')).toBe('canid');
    expect(animalIcon('lynx')).toBe('cat');
    expect(animalIcon('goat antelope')).toBe('horn');
    expect(animalIcon('crocodilian')).toBe('croc');
  });

  it('falls back to the paw for unknown classes', () => {
    expect(animalIcon('')).toBe('paw');
    expect(animalIcon('mammal')).toBe('paw');
  });

  it('draws every animal icon', () => {
    for (const group of ['deer', 'bird', 'canid', 'cat', 'bear', 'pig', 'bovine', 'sheep', 'hare', 'crocodilian']) {
      expect(icon(animalIcon(group))).toContain('<path d="M');
    }
  });
});

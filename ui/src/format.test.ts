import { describe, expect, it } from 'vitest';

import { escapeHtml } from './escape';
import { formatBytes, formatEpoch, formatStamp } from './format';

describe('formatBytes', () => {
  it('uses the right unit and pt-BR decimals', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(1536)).toBe('1,5 KB');
    expect(formatBytes(436_671)).toBe('426,4 KB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5,0 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3,0 GB');
    expect(formatBytes(5000 * 1024 ** 3)).toContain('GB');
  });
});

describe('formatStamp', () => {
  it('turns a backup folder stamp into a readable date', () => {
    expect(formatStamp('2026-11-02_15-30-00')).toBe('02/11/2026 15:30:00');
  });

  it('keeps unknown text unchanged', () => {
    expect(formatStamp('meu-backup')).toBe('meu-backup');
  });
});

describe('formatEpoch', () => {
  it('formats in pt-BR for a fixed time zone', () => {
    expect(formatEpoch(0, 'UTC')).toMatch(/01\/01\/1970,? 00:00/);
  });

  it('works without an explicit time zone', () => {
    expect(formatEpoch(1_793_600_000)).toMatch(/\d{2}\/\d{2}\/\d{4}/);
  });
});

describe('escapeHtml', () => {
  it('escapes every HTML-significant character', () => {
    expect(escapeHtml(`<a href="x" onclick='y'>&</a>`)).toBe(
      '&lt;a href=&quot;x&quot; onclick=&#39;y&#39;&gt;&amp;&lt;/a&gt;',
    );
    expect(escapeHtml('texto simples')).toBe('texto simples');
  });
});

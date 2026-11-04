import { describe, expect, it } from 'vitest';

import { sampleMissions } from '../test-data';
import type { Mission, ReserveMissions } from '../types';
import { quickWins, renderMissions } from './missions';

function mission(id: number, objectives: [number, number][], state: Mission['state'] = 'active'): Mission {
  return {
    id,
    text: `Missão ${String(id)}`,
    state,
    main: false,
    objectives: objectives.map(([done, required]) => ({ done, required })),
  };
}

function reserve(id: number, missions: Mission[]): ReserveMissions {
  return { id, name: `Reserva ${String(id)}`, total: missions.length, done: 0, active: missions.length, mainTotal: 0, mainDone: 0, missions };
}

describe('quick wins', () => {
  it('lists started active missions, fewest objectives left first, across reserves', () => {
    const list = [
      reserve(1, [mission(10, [[1, 2], [0, 1], [0, 1]]), mission(11, [[1, 1], [1, 3]])]),
      reserve(2, [mission(20, [[4, 5]]), mission(21, [[2, 2], [0, 2]])]),
    ];
    expect(quickWins(list).map((w) => w.mission.id)).toEqual([20, 11, 21, 10]);
  });

  it('ignores missions with no progress, already complete, locked, done or without objectives', () => {
    const list = [
      reserve(1, [
        mission(1, [[0, 3]]),
        mission(2, [[3, 3]]),
        mission(3, [[1, 2]], 'locked'),
        mission(4, [[1, 2]], 'done'),
        mission(5, []),
      ]),
    ];
    expect(quickWins(list)).toEqual([]);
  });

  it('keeps only the requested number', () => {
    const list = [reserve(1, Array.from({ length: 8 }, (_, i) => mission(i, [[1, 2]])))];
    expect(quickWins(list, 3)).toHaveLength(3);
  });

  it('shows the card and the button that opens the reserve', () => {
    const html = renderMissions(sampleMissions, new Map(), 2, 'active');
    expect(html).toContain('class="quick-win"');
    expect(html).toContain('Fotografe um alce.');
    expect(html).toContain('data-reserve="2"');
  });

  it('shows no card when nothing is close', () => {
    expect(renderMissions([reserve(1, [mission(1, [[0, 1]])])], new Map(), 1, 'all')).not.toContain('quick-win');
  });
});

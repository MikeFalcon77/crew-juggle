import { describe, it, expect } from 'vitest';
import { cropLayout } from './crop';
import { celebrationFor } from './celebration';
import type { Day, Occurrence } from './types';
const item: Occurrence = {
  id: '2026-11-10/a',
  revision: 1,
  status: 'pending',
  completed_by: null,
  completed_at: null,
  routine: {
    id: 'a',
    revision: 1,
    person_id: 'p-zed',
    assignee: 'Zed',
    name: 'Do the dishes',
    note: '',
    group: '',
    time: '08:00',
    days: [],
    start_date: '2026-11-10',
    end_date: null,
    enabled: true,
    prerequisite: null,
  },
};
const day: Day = {
  date: '2026-11-10',
  today: '2026-11-10',
  first_date: '2026-11-10',
  timezone: 'Europe/Berlin',
  scheduled_times: {},
  server_now: '2026-11-10T05:00:00Z',
  people: [],
  items: [item],
};
describe('Saved-action reactions and crop bounds', () => {
  it('prioritizes day over period, and ignores undo or previously completed records', () => {
    const done = { ...item, status: 'done' as const };
    expect(celebrationFor(day, done, 'op')?.level).toBe('day');
    expect(celebrationFor({ ...day, items: [done] }, done, 'op')).toBeNull();
    expect(celebrationFor(day, item, 'op')).toBeNull();
    const evening = {
      ...item,
      id: 'evening',
      routine: { ...item.routine, time: '20:00' },
    };
    expect(celebrationFor({ ...day, items: [item, evening] }, done, 'op')?.level).toBe('period');
    const morning = { ...item, id: 'morning' };
    expect(celebrationFor({ ...day, items: [item, morning, evening] }, done, 'op')?.level).toBe(
      'item',
    );
  });
  it('ignores cancelled tasks and another person’s unfinished tasks', () => {
    const other = {
      ...item,
      id: 'other',
      routine: { ...item.routine, person_id: 'mochi' },
    };
    const stopped = { ...item, id: 'stopped', status: 'cancelled' as const };
    expect(
      celebrationFor({ ...day, items: [item, other, stopped] }, { ...item, status: 'done' }, 'op')
        ?.level,
    ).toBe('day');
  });
  it('centres landscape/portrait images, covers the square and clamps dragged edges', () => {
    expect(cropLayout(1000, 500, 1, 999, -999)).toEqual({ width: 512, height: 256, x: 128, y: 0 });
    expect(cropLayout(500, 1000, 1, -999, 999)).toEqual({ width: 256, height: 512, x: 0, y: 128 });
    expect(cropLayout(256, 256, 2, 999, 999)).toEqual({ width: 512, height: 512, x: 128, y: 128 });
  });
});

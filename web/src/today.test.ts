import { describe, expect, it } from 'vitest';
import { todayCards } from './today';
import { flatDay } from './testFixtures';

describe('Today cards', () => {
  it('flags overdue work, celebrates finished people and points to what is next', () => {
    const cards = todayCards({ day: flatDay(), today: '2026-11-10' });
    const zed = cards.find((c) => c.personId === 'p-zed')!;
    expect(zed.group).toBe('attention');
    expect(zed.title).toBe('1 overdue');
    expect(zed.action?.routine.id).toBe('dishes');
    const mochi = cards.find((c) => c.personId === 'mochi')!;
    expect(mochi.group).toBe('good');
    const pip = cards.find((c) => c.personId === 'p-pip')!;
    expect(pip.group).toBe('next');
    expect(pip.action?.routine.id).toBe('trash');
  });

  it('adds the shopping list and today’s scanned receipts', () => {
    const cards = todayCards({
      today: '2026-11-10',
      shopping: {
        list: [
          { name: 'Cat food', done: false, author: 'Pip' },
          { name: 'Dish soap', done: true, author: 'Pip' },
        ],
        receipts: [
          {
            id: 'r1',
            merchant: 'Corner Market',
            status: 'saved',
            registeredBy: 'Zed',
            date: '2026-11-10',
          },
          {
            id: 'r2',
            merchant: 'Old Shop',
            status: 'saved',
            registeredBy: 'Zed',
            date: '2026-11-01',
          },
        ],
      },
    });
    expect(cards.map((c) => c.title)).toEqual(['1 thing to buy', 'Receipt from Corner Market']);
  });

  it('ignores a day that is not today', () => {
    expect(todayCards({ day: flatDay(), today: '2026-11-11' })).toEqual([]);
  });
});

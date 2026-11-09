import { afterEach, describe, expect, it } from 'vitest';
import { completedLabel, daysLabel, scheduledLabel, setCrewZone } from './time';

describe('crew time', () => {
  afterEach(() => setCrewZone('UTC'));

  it('describes weekdays the way people say them', () => {
    expect(daysLabel([])).toBe('Every day');
    expect(daysLabel([1, 2, 3, 4, 5, 6, 7])).toBe('Every day');
    expect(daysLabel([5, 4, 3, 2, 1])).toBe('Weekdays');
    expect(daysLabel([6, 7])).toBe('Weekends');
    expect(daysLabel([2])).toBe('Tue');
    expect(daysLabel([5, 1, 3])).toBe('Mon, Wed, Fri');
  });

  it('adds your local clock only when you are somewhere else', () => {
    setCrewZone('Europe/Berlin');
    const at = '2026-11-10T07:00:00Z';
    expect(scheduledLabel('08:00', at, 'Europe/Berlin', '2026-11-10')).toBe('08:00');
    expect(scheduledLabel('08:00', at, 'America/New_York', '2026-11-10')).toBe(
      '08:00 · 02:00 New York',
    );
    expect(scheduledLabel('08:00', at, 'America/Los_Angeles', '2026-11-10')).toBe(
      '08:00 · 23:00 Los Angeles (Nov 9)',
    );
    expect(completedLabel('2026-11-10T06:55:00Z', 'Europe/Berlin')).toBe('07:55');
  });
});

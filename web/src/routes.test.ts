import { expect, it } from 'vitest';
import { parseRoute, routeUrl } from './routes';

it('falls back to sensible tabs and keeps invite codes', () => {
  expect(parseRoute(new URL('https://crew.test/'))).toMatchObject({ page: 'today' });
  expect(parseRoute(new URL('https://crew.test/routines'))).toMatchObject({
    page: 'routines',
    tab: 'today',
  });
  expect(parseRoute(new URL('https://crew.test/routines?date=2026-11-01'))).toMatchObject({
    tab: 'history',
    date: '2026-11-01',
  });
  expect(parseRoute(new URL('https://crew.test/shopping?tab=nope'))).toMatchObject({ tab: 'list' });
  expect(parseRoute(new URL('https://crew.test/join?code=K7QX-M2PA'))).toMatchObject({
    page: 'join',
    code: 'K7QX-M2PA',
  });
  expect(parseRoute(new URL('https://crew.test/nowhere'))).toMatchObject({ page: 'today' });
});

it('round-trips a route through its URL', () => {
  const route = { page: 'routines' as const, tab: 'history', person: 'mochi', date: '2026-11-09' };
  expect(parseRoute(new URL(routeUrl(route), 'https://crew.test'))).toMatchObject(route);
});

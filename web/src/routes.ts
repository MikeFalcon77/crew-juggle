import { useEffect, useState } from 'react';
export type Page = 'today' | 'routines' | 'shopping' | 'crew' | 'settings' | 'join';
export type AppRoute = {
  page: Page;
  tab?: string;
  section?: string;
  person?: string;
  date?: string;
  record?: string;
  code?: string;
};
const PAGES: Page[] = ['today', 'routines', 'shopping', 'crew', 'settings', 'join'];
export function parseRoute(url = new URL(location.href)): AppRoute {
  const q = url.searchParams;
  const fields = {
    tab: q.get('tab') || undefined,
    section: q.get('section') || undefined,
    person: q.get('person') || undefined,
    date: q.get('date') || undefined,
    record: q.get('record') || undefined,
    code: q.get('code') || undefined,
  };
  const page = url.pathname.slice(1) as Page;
  const result: AppRoute = { ...fields, page: PAGES.includes(page) ? page : 'today' };
  if (result.page === 'routines' && !['today', 'schedule', 'history'].includes(result.tab || ''))
    result.tab = result.date ? 'history' : 'today';
  if (result.page === 'shopping' && !['list', 'history', 'analytics'].includes(result.tab || ''))
    result.tab = 'list';
  if (
    result.page === 'settings' &&
    !['notifications', 'time', 'about'].includes(result.section || '')
  )
    result.section = 'notifications';
  return result;
}
export function routeUrl(route: AppRoute) {
  const q = new URLSearchParams();
  for (const key of ['tab', 'section', 'person', 'date', 'record', 'code'] as const) {
    if (route[key]) q.set(key, route[key]!);
  }
  return `/${route.page}${q.size ? '?' + q.toString() : ''}`;
}
export function useAppRoute() {
  const [route, setRoute] = useState(() => parseRoute());
  useEffect(() => {
    const pop = () => setRoute(parseRoute());
    window.addEventListener('popstate', pop);
    return () => window.removeEventListener('popstate', pop);
  }, []);
  const navigate = (next: AppRoute, replace = false) => {
    history[replace ? 'replaceState' : 'pushState']({}, '', routeUrl(next));
    setRoute(next);
  };
  return { route, navigate };
}

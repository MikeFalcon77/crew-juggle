const CACHE = 'crew-juggle-__BUILD__';
const SHELL = __SHELL__;
// API calls of every gear are never cached: they go straight to the server.
const API_PREFIXES = [
  '/juggle-crews/',
  '/juggle-routines/',
  '/juggle-shopping/',
  '/docs',
  '/openapi.json',
];
self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(SHELL)));
  self.skipWaiting();
});
self.addEventListener('activate', (event) =>
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith('crew-juggle-') && key !== CACHE)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  ),
);
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (
    event.request.method !== 'GET' ||
    url.origin !== self.location.origin ||
    API_PREFIXES.some((prefix) => url.pathname.startsWith(prefix)) ||
    url.pathname.startsWith('/__/')
  )
    return;
  if (event.request.mode === 'navigate') {
    event.respondWith(fetch(event.request).catch(() => caches.match('/index.html')));
    return;
  }
  if (SHELL.includes(url.pathname))
    event.respondWith(caches.match(event.request).then((cached) => cached || fetch(event.request)));
});
self.addEventListener('push', (event) => {
  let data;
  try {
    data = event.data.json();
  } catch {
    return;
  }
  if (!data || typeof data !== 'object') return;
  event.waitUntil(
    self.registration
      .showNotification(data.title || 'crew-juggle', {
        body: data.body,
        tag: data.tag,
        icon: '/icons/crew-juggle-192.png',
        badge: '/icons/notification-badge.png',
        data: { url: notificationTarget(data.url, data.tag) },
      })
      .then(() => self.clients.matchAll({ type: 'window', includeUncontrolled: true }))
      .then((clients) =>
        clients.forEach((client) =>
          client.postMessage({
            type: data.kind === 'shopping' ? 'shopping-update' : 'routines-reminder',
            tag: data.tag,
          }),
        ),
      ),
  );
});
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then(async (clients) => {
      const client = clients.find((c) => new URL(c.url).origin === self.location.origin);
      const target = notificationTarget(event.notification.data?.url, event.notification.tag);
      if (client) {
        await client.navigate(target);
        return client.focus();
      }
      return self.clients.openWindow(target);
    }),
  );
});
function notificationTarget(value, tag = '') {
  try {
    const url = new URL(typeof value === 'string' ? value : '/today', self.location.origin);
    if (url.origin !== self.location.origin || !['/today', '/shopping'].includes(url.pathname))
      return '/today';
    if (url.pathname === '/today' && typeof tag === 'string' && tag.startsWith('routines/')) {
      const date = /^routines\/(\d{4}-\d{2}-\d{2})$/.exec(tag)?.[1];
      return '/routines?tab=today' + (date ? '&date=' + date : '');
    }
    return url.pathname + url.search;
  } catch {
    return '/today';
  }
}

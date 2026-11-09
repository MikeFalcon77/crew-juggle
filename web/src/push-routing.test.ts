import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { resolve } from 'node:path';
const source = readFileSync(resolve('public/sw.js'), 'utf8');
function worker() {
  const handlers: Record<string, (event: any) => void> = {};
  const postMessage = vi.fn();
  const showNotification = vi.fn().mockResolvedValue(undefined);
  const openWindow = vi.fn().mockResolvedValue(undefined);
  const navigate = vi.fn().mockResolvedValue(undefined),
    focus = vi.fn().mockResolvedValue(undefined);
  vm.runInNewContext(source, {
    URL,
    __SHELL__: [],
    self: {
      location: { origin: 'https://crew.test' },
      addEventListener: (name: string, handler: any) => (handlers[name] = handler),
      registration: { showNotification },
      clients: {
        matchAll: async () => [{ postMessage, url: 'https://crew.test/today', navigate, focus }],
        openWindow,
      },
    },
  });
  return { handlers, postMessage, showNotification, openWindow, navigate, focus };
}
describe('push routing', () => {
  it('opens today’s routines from a reminder', async () => {
    const w = worker();
    let pending: Promise<void> = Promise.resolve();
    w.handlers.push({
      data: { json: () => ({ kind: 'routines', tag: 'routines/2026-11-10', url: '/today' }) },
      waitUntil: (p: Promise<void>) => (pending = p),
    });
    await pending;
    expect(w.showNotification.mock.calls[0][1].data.url).toBe(
      '/routines?tab=today&date=2026-11-10',
    );
    expect(w.postMessage.mock.calls[0][0].type).toBe('routines-reminder');
    w.handlers.notificationclick({
      notification: { tag: 'routines/2026-11-10', data: { url: '/today' }, close: vi.fn() },
      waitUntil: (p: Promise<void>) => (pending = p),
    });
    await pending;
    expect(w.navigate).toHaveBeenCalledWith('/routines?tab=today&date=2026-11-10');
    expect(w.focus).toHaveBeenCalled();
  });
  it('sends receipt news to shopping', async () => {
    const w = worker();
    let pending: Promise<void> = Promise.resolve();
    w.handlers.push({
      data: {
        json: () => ({
          kind: 'shopping',
          title: 'crew-juggle · shopping',
          url: '/shopping?tab=history',
        }),
      },
      waitUntil: (p: Promise<void>) => (pending = p),
    });
    await pending;
    expect(w.showNotification.mock.calls[0][1].data.url).toBe('/shopping?tab=history');
    expect(w.postMessage.mock.calls[0][0].type).toBe('shopping-update');
  });
  it('never sends a notification click outside the app', async () => {
    const w = worker();
    let pending: Promise<void> = Promise.resolve();
    w.handlers.push({
      data: { json: () => ({ url: 'https://other.test/steal' }) },
      waitUntil: (p: Promise<void>) => (pending = p),
    });
    await pending;
    expect(w.showNotification.mock.calls[0][1].data.url).toBe('/today');
    expect(w.showNotification).toHaveBeenCalledWith(
      'crew-juggle',
      expect.objectContaining({
        icon: '/icons/crew-juggle-192.png',
        badge: '/icons/notification-badge.png',
      }),
    );
  });
});

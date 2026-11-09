import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const source = readFileSync('public/sw.js', 'utf8');

describe('PWA updates', () => {
  it('removes outdated app shells while preserving other caches', async () => {
    const handlers: Record<string, (event: any) => void> = {};
    const remove = vi.fn().mockResolvedValue(true);
    const claim = vi.fn().mockResolvedValue(undefined);
    vm.runInNewContext(source.replace('__BUILD__', 'current'), {
      __SHELL__: [],
      caches: {
        keys: async () => ['crew-juggle-old', 'crew-juggle-current', 'other-app'],
        delete: remove,
      },
      self: {
        addEventListener: (name: string, handler: any) => (handlers[name] = handler),
        clients: { claim },
      },
    });
    let pending = Promise.resolve();
    handlers.activate({ waitUntil: (p: Promise<void>) => (pending = p) });
    await pending;
    expect(remove.mock.calls.map(([key]) => key)).toEqual(['crew-juggle-old']);
    expect(claim).toHaveBeenCalledOnce();
  });

  it('never caches any gear API', () => {
    const handlers: Record<string, (event: any) => void> = {};
    vm.runInNewContext(source, {
      URL,
      __SHELL__: [],
      self: {
        location: { origin: 'https://crew.test' },
        addEventListener: (name: string, handler: any) => (handlers[name] = handler),
      },
    });
    for (const path of [
      '/juggle-routines/v1/day',
      '/juggle-crews/v1/me',
      '/juggle-shopping/v1/state',
      '/openapi.json',
    ]) {
      const respondWith = vi.fn();
      handlers.fetch({
        request: { method: 'GET', url: `https://crew.test${path}`, mode: 'navigate' },
        respondWith,
      });
      expect(respondWith).not.toHaveBeenCalled();
    }
  });

  it('ships valid PNGs matching every advertised mobile icon size', () => {
    const manifest = JSON.parse(readFileSync('public/manifest.webmanifest', 'utf8'));
    const icons = [
      ...manifest.icons,
      { src: '/icons/apple-touch-icon.png', sizes: '180x180' },
      { src: '/icons/notification-badge.png', sizes: '96x96' },
      { src: '/icons/favicon-32.png', sizes: '32x32' },
    ];
    for (const icon of icons) {
      const png = readFileSync(`public${icon.src}`);
      expect(png.subarray(0, 8).toString('hex')).toBe('89504e470d0a1a0a');
      expect(`${png.readUInt32BE(16)}x${png.readUInt32BE(20)}`).toBe(icon.sizes);
    }
  });
});

import { afterEach, describe, expect, it, vi } from 'vitest';
import { registerPush } from './push';
const api = vi.hoisted(() => vi.fn());
vi.mock('./api', () => ({ api }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
  localStorage.clear();
});
function fixture(byte: number) {
  const old = {
    options: { applicationServerKey: new Uint8Array([byte]).buffer },
    unsubscribe: vi.fn().mockResolvedValue(true),
    toJSON: () => ({ endpoint: 'https://push.example/old' }),
  };
  const next = { toJSON: () => ({ endpoint: 'https://push.example/new' }) };
  const subscribe = vi.fn().mockResolvedValue(next);
  const worker = { pushManager: { getSubscription: vi.fn().mockResolvedValue(old), subscribe } };
  vi.stubGlobal('PushManager', class {});
  vi.stubGlobal('Notification', { requestPermission: vi.fn().mockResolvedValue('granted') });
  vi.stubGlobal('navigator', {
    serviceWorker: { register: vi.fn().mockResolvedValue(worker), ready: Promise.resolve(worker) },
  });
  api.mockResolvedValue({ id: 'device' });
  return { old, subscribe };
}
describe('VAPID rotation', () => {
  it('replaces an old-key browser subscription before registering with the server', async () => {
    const { old, subscribe } = fixture(1);
    await registerPush('Ag');
    expect(old.unsubscribe).toHaveBeenCalledTimes(1);
    expect(subscribe).toHaveBeenCalledWith({
      userVisibleOnly: true,
      applicationServerKey: new Uint8Array([2]),
    });
    expect(api).toHaveBeenCalledWith('/push/subscriptions', 'POST', {
      endpoint: 'https://push.example/new',
    });
  });
  it('reuses a browser subscription already bound to the current key', async () => {
    const { old, subscribe } = fixture(2);
    await registerPush('Ag');
    expect(old.unsubscribe).not.toHaveBeenCalled();
    expect(subscribe).not.toHaveBeenCalled();
    expect(api).toHaveBeenCalledWith('/push/subscriptions', 'POST', {
      endpoint: 'https://push.example/old',
    });
  });
});

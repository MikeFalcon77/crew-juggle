import { api } from './api';
export function pushSupported() {
  return 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
}
export async function registerPush(key: string) {
  if (!pushSupported())
    throw new Error(
      'Push is not available here. On iPhone, add the app to your Home Screen first.',
    );
  const permission = await Notification.requestPermission();
  if (permission !== 'granted')
    throw new Error('Notifications are blocked. Allow them in your browser settings.');
  const worker = await navigator.serviceWorker.register('/sw.js');
  await navigator.serviceWorker.ready;
  const binary = atob(key.replace(/-/g, '+').replace(/_/g, '/'));
  const applicationServerKey = Uint8Array.from(binary, (c) => c.charCodeAt(0));
  let subscription = await worker.pushManager.getSubscription();
  const currentKey = subscription?.options.applicationServerKey;
  if (
    subscription &&
    (!currentKey ||
      new Uint8Array(currentKey).length !== applicationServerKey.length ||
      new Uint8Array(currentKey).some((value, index) => value !== applicationServerKey[index]))
  ) {
    if (!(await subscription.unsubscribe()))
      throw new Error(
        "Couldn't replace the old subscription. Turn notifications off and try again.",
      );
    subscription = null;
  }
  subscription ??= await worker.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey,
  });
  const { id } = await api<{ id: string }>('/push/subscriptions', 'POST', subscription.toJSON());
  localStorage.setItem('crew-juggle-push-id', id);
  return id;
}
export async function disablePush() {
  const id = localStorage.getItem('crew-juggle-push-id');
  let failure: unknown;
  try {
    if (id) await api(`/push/subscriptions/${id}`, 'DELETE');
  } catch (error) {
    failure = error;
  }
  try {
    const worker = await navigator.serviceWorker.getRegistration();
    await (await worker?.pushManager.getSubscription())?.unsubscribe();
  } finally {
    localStorage.removeItem('crew-juggle-push-id');
  }
  if (failure) throw failure;
}
let audio: AudioContext | null = null;
export function ring() {
  audio ??= new AudioContext();
  void audio.resume();
  const oscillator = audio.createOscillator();
  const gain = audio.createGain();
  oscillator.type = 'sine';
  oscillator.frequency.value = 880;
  gain.gain.setValueAtTime(0.0001, audio.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.12, audio.currentTime + 0.015);
  gain.gain.exponentialRampToValueAtTime(0.0001, audio.currentTime + 0.7);
  oscillator.connect(gain);
  gain.connect(audio.destination);
  oscillator.start();
  oscillator.stop(audio.currentTime + 0.7);
}

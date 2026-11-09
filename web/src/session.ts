/**
 * Who is signed in. A deployment signs in with Google (Firebase); a local
 * demo signs in by picking a character. The server decides which one via
 * /juggle-crews/v1/config, and only accepts demo tokens while it runs in demo
 * mode, so this file cannot turn a real deployment into a demo.
 */
import * as firebase from './firebase';

export type SignIn = 'google' | 'demo';
export type DemoUser = { key: string; name: string; crew: string; tagline: string };
export type SessionConfig = { sign_in: SignIn; demo_users: DemoUser[] };

const DEMO_KEY = 'crew-juggle-demo-token';
let mode: SignIn = 'google';
const listeners = new Set<(signedIn: boolean, uid?: string) => void>();

function storage(): Storage | null {
  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}
function demoToken() {
  return storage()?.getItem(DEMO_KEY) ?? null;
}
function demoUid(token: string | null) {
  const key = token?.split('.')[1];
  return key ? `demo-${key}` : undefined;
}

export async function loadConfig(): Promise<SessionConfig> {
  try {
    const response = await fetch('/juggle-crews/v1/config', { cache: 'no-store' });
    const config = (await response.json()) as SessionConfig;
    mode = config.sign_in;
    return config;
  } catch {
    mode = 'google';
    return { sign_in: 'google', demo_users: [] };
  }
}

export function signInMode() {
  return mode;
}

export function googleConfigured() {
  return firebase.configured;
}

export function watch(callback: (signedIn: boolean, uid?: string) => void) {
  if (mode === 'demo') {
    listeners.add(callback);
    const token = demoToken();
    callback(Boolean(token), demoUid(token));
    return () => listeners.delete(callback);
  }
  return firebase.watchUser(callback);
}

export async function signInAsDemo(user: string) {
  const response = await fetch('/juggle-crews/v1/demo/sessions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user }),
  });
  if (!response.ok) throw new Error('Demo sign-in is not available on this server');
  const { token } = (await response.json()) as { token: string };
  storage()?.setItem(DEMO_KEY, token);
  listeners.forEach((l) => l(true, demoUid(token)));
}

export async function signInWithGoogle() {
  await firebase.login();
}

export async function signOut() {
  if (mode === 'demo') {
    storage()?.removeItem(DEMO_KEY);
    listeners.forEach((l) => l(false));
    return;
  }
  await firebase.logout();
}

/** Bearer token for API calls, or null when signed out. */
export async function token(force = false) {
  if (mode === 'demo') {
    const value = demoToken();
    // A restarted demo server forgets its tokens: go back to the picker.
    if (force && value) {
      storage()?.removeItem(DEMO_KEY);
      listeners.forEach((l) => l(false));
      return null;
    }
    return value;
  }
  return firebase.token(force);
}

/** Account id of whoever is signed in on this device, or '' when nobody is. */
export function currentUserId() {
  if (mode === 'demo') return demoUid(demoToken()) || '';
  return firebase.currentUserId();
}

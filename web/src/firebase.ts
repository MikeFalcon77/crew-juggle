import { initializeApp } from 'firebase/app';
import {
  getAuth,
  GoogleAuthProvider,
  signInWithPopup,
  signOut,
  onAuthStateChanged,
} from 'firebase/auth';
const config = {
  apiKey: import.meta.env.VITE_FIREBASE_API_KEY,
  authDomain: import.meta.env.VITE_FIREBASE_AUTH_DOMAIN,
  projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID,
};
export const configured = Boolean(config.apiKey && config.authDomain && config.projectId);
const auth = configured ? getAuth(initializeApp(config)) : null;
export function currentUserId() {
  return auth?.currentUser?.uid || '';
}
export function watchUser(callback: (loggedIn: boolean, uid?: string) => void) {
  if (!auth) {
    callback(false);
    return () => {};
  }
  return onAuthStateChanged(auth, (user) => callback(Boolean(user), user?.uid));
}
export async function login() {
  if (!auth) throw new Error('Google sign-in is not configured');
  await signInWithPopup(auth, new GoogleAuthProvider());
}
export async function logout() {
  if (auth) await signOut(auth);
}
export async function token(force = false) {
  return auth?.currentUser?.getIdToken(force) ?? null;
}

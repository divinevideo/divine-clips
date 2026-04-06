import { writable, derived } from 'svelte/store';

export const authToken = writable<string | null>(null);
export const isAuthenticated = derived(authToken, ($token) => !!$token);

export function login(token: string) {
  localStorage.setItem('clipcrate_token', token);
  authToken.set(token);
}

export function logout() {
  localStorage.removeItem('clipcrate_token');
  authToken.set(null);
}

export function initAuth() {
  if (typeof window !== 'undefined') {
    const token = localStorage.getItem('clipcrate_token');
    if (token) authToken.set(token);
  }
}

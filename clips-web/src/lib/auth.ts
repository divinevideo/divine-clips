import { login } from './stores/auth';

// MVP: prompt user for their npub or generate a demo token
// In production: redirect to Keycast OAuth flow
export async function loginWithKeycast(): Promise<void> {
  const npub = prompt('Enter your npub (or any identifier for MVP):');
  if (npub && npub.trim()) {
    login(npub.trim());
  }
}

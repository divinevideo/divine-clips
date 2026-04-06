import { createDivineClient } from '@divinevideo/login';
import { login, logout as storeLogout } from './stores/auth';

const DIVINE_AUTH_SERVER = 'https://login.divine.video';
const CLIENT_ID = 'clips.divine.video';

function getRedirectUri(): string {
  if (typeof window === 'undefined') return '';
  return `${window.location.origin}/auth/callback`;
}

function getClient() {
  return createDivineClient({
    serverUrl: DIVINE_AUTH_SERVER,
    clientId: CLIENT_ID,
    redirectUri: getRedirectUri(),
    storage: typeof localStorage !== 'undefined' ? localStorage : undefined,
  });
}

/**
 * Initiate DiVine login — redirects to login.divine.video OAuth
 */
export async function loginWithKeycast(): Promise<void> {
  const client = getClient();
  const { url, pkce } = await client.oauth.getAuthorizationUrl();

  // Store PKCE verifier for callback
  localStorage.setItem('divine_pkce_verifier', pkce.verifier);

  // Redirect to DiVine login
  window.location.href = url;
}

/**
 * Handle OAuth callback — call this from /auth/callback page
 */
export async function handleAuthCallback(callbackUrl: string): Promise<boolean> {
  const client = getClient();

  try {
    const result = client.oauth.parseCallback(callbackUrl);

    if ('error' in result) {
      console.error('Auth callback error:', result.error);
      return false;
    }

    const verifier = localStorage.getItem('divine_pkce_verifier');
    localStorage.removeItem('divine_pkce_verifier');

    const tokens = await client.oauth.exchangeCode(result.code, verifier ?? undefined);

    // Store the access token for API calls
    if (!tokens.access_token) throw new Error('No access token received');
    login(tokens.access_token);

    return true;
  } catch (err) {
    console.error('Auth exchange failed:', err);
    return false;
  }
}

/**
 * Restore session from stored tokens (auto-refresh if needed)
 */
export async function restoreSession(): Promise<boolean> {
  const client = getClient();

  try {
    const session = await client.oauth.getSessionWithRefresh();
    if (session?.accessToken) {
      login(session.accessToken);
      return true;
    }
  } catch {
    // Session expired or invalid
  }

  return false;
}

/**
 * Logout — clear session from DiVine client and local store
 */
export function logout(): void {
  const client = getClient();
  client.oauth.logout();
  storeLogout();
}

/**
 * Get an RPC client for Nostr signing (if needed)
 */
export async function getSigningClient() {
  const client = getClient();
  const session = await client.oauth.getSessionWithRefresh();
  if (!session) throw new Error('Not authenticated');
  // createRpc expects TokenResponse-like object; map from stored credentials
  return client.createRpc({
    bunker_url: session.bunkerUrl || '',
    access_token: session.accessToken,
    refresh_token: session.refreshToken || '',
    token_type: 'Bearer',
    expires_in: 86400,
    scope: '',
  } as any);
}

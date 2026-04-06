// MVP: Use SSE from clipcrate. Future: WebSocket to funnelcake relay.
export function connectToFeed(apiBase: string, onEvent: (data: any) => void): EventSource | null {
  if (typeof window === 'undefined') return null;
  const es = new EventSource(`${apiBase}/api/feed/live`);
  es.addEventListener('campaign', (e) => {
    try { onEvent(JSON.parse((e as MessageEvent).data)); } catch {}
  });
  return es;
}

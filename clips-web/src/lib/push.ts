const API_BASE = import.meta.env.VITE_API_URL || 'https://api.clips.divine.video';

export async function registerPushNotifications(token: string): Promise<boolean> {
  if (!('serviceWorker' in navigator) || !('PushManager' in window)) return false;

  try {
    const reg = await navigator.serviceWorker.register('/sw.js');

    // Get VAPID public key from server
    const res = await fetch(`${API_BASE}/api/notifications/vapid-key`);
    const { vapid_public_key } = await res.json();

    const subscription = await reg.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: vapid_public_key,
    });

    const subJson = subscription.toJSON();
    await fetch(`${API_BASE}/api/notifications/subscribe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({
        endpoint: subJson.endpoint,
        keys: { p256dh: subJson.keys?.p256dh, auth: subJson.keys?.auth },
      }),
    });

    return true;
  } catch (err) {
    console.error('Push registration failed:', err);
    return false;
  }
}

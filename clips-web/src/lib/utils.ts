export function formatSats(sats: number): string {
  return sats.toLocaleString() + ' sats';
}

export function formatViews(views: number): string {
  if (views >= 1_000_000) return (views / 1_000_000).toFixed(1) + 'M';
  if (views >= 1_000) return (views / 1_000).toFixed(1) + 'K';
  return views.toString();
}

export function timeAgo(date: string): string {
  const seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return Math.floor(seconds / 60) + 'm ago';
  if (seconds < 86400) return Math.floor(seconds / 3600) + 'h ago';
  return Math.floor(seconds / 86400) + 'd ago';
}

export const PLATFORMS = ['tiktok', 'instagram', 'youtube', 'x'] as const;
export type Platform = typeof PLATFORMS[number];

export function platformLabel(p: string): string {
  const labels: Record<string, string> = {
    tiktok: 'TikTok', instagram: 'Instagram',
    youtube: 'YouTube', x: 'X'
  };
  return labels[p] || p;
}

export function budgetPercent(campaign: { budget_total_sats: number; budget_remaining_sats: number }): number {
  if (campaign.budget_total_sats === 0) return 0;
  return Math.round((campaign.budget_remaining_sats / campaign.budget_total_sats) * 100);
}

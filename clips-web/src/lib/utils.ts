// Bitcoin price in USD — updated periodically, fallback to reasonable estimate
// In production, fetch from an API. For now, a reasonable default.
let btcPriceUsd = 100_000; // $100K per BTC
const SATS_PER_BTC = 100_000_000;

export function setBtcPrice(price: number) {
  btcPriceUsd = price;
}

export function satsToUsd(sats: number): number {
  return (sats / SATS_PER_BTC) * btcPriceUsd;
}

export function usdToSats(usd: number): number {
  return Math.round((usd / btcPriceUsd) * SATS_PER_BTC);
}

/**
 * Format money amount — shows USD primary, sats secondary
 * e.g., "$3.00 (3,000 sats)" or "$0.15 (150 sats)"
 */
export function formatMoney(sats: number): string {
  const usd = satsToUsd(sats);
  if (usd >= 1) {
    return `$${usd.toFixed(2)}`;
  }
  if (usd >= 0.01) {
    return `$${usd.toFixed(2)}`;
  }
  return `$${usd.toFixed(4)}`;
}

/**
 * Format with both USD and sats
 */
export function formatMoneyFull(sats: number): string {
  return `${formatMoney(sats)} (${sats.toLocaleString()} sats)`;
}

/**
 * Format sats only (for contexts where sats are appropriate)
 */
export function formatSats(sats: number): string {
  return sats.toLocaleString() + ' sats';
}

/**
 * Format CPM rate — show as "per 1,000 views" in USD
 * e.g., "$3.00 / 1K views"
 */
export function formatCpm(cpmSats: number): string {
  return `${formatMoney(cpmSats)} / 1K views`;
}

/**
 * Format CPM with sats detail
 * e.g., "$3.00 / 1K views (3,000 sats)"
 */
export function formatCpmFull(cpmSats: number): string {
  return `${formatMoney(cpmSats)} / 1K views (${cpmSats.toLocaleString()} sats)`;
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

/**
 * Supported input currencies for campaign creation
 */
export const CURRENCIES = ['USD', 'EUR', 'GBP', 'BTC', 'sats'] as const;
export type Currency = typeof CURRENCIES[number];

const CURRENCY_TO_USD: Record<string, number> = {
  USD: 1,
  EUR: 1.08,
  GBP: 1.27,
};

/**
 * Convert any supported currency amount to sats
 */
export function currencyToSats(amount: number, currency: Currency): number {
  if (currency === 'sats') return Math.round(amount);
  if (currency === 'BTC') return Math.round(amount * SATS_PER_BTC);
  const usdAmount = amount * (CURRENCY_TO_USD[currency] || 1);
  return usdToSats(usdAmount);
}

/**
 * Format a currency input for display
 */
export function formatCurrencyInput(amount: number, currency: Currency): string {
  if (currency === 'sats') return `${amount.toLocaleString()} sats`;
  if (currency === 'BTC') return `${amount} BTC`;
  const symbol = currency === 'EUR' ? '€' : currency === 'GBP' ? '£' : '$';
  return `${symbol}${amount.toFixed(2)}`;
}

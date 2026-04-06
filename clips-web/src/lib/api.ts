import type { Campaign, Submission, DashboardData, TransactionRecord, CreateCampaignRequest, CreateSubmissionRequest, AnalyticsOverview, SubmissionAnalytics } from './types';

const API_BASE = import.meta.env.VITE_API_URL || 'https://api.clips.divine.video';

function getToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('clipcrate_token');
}

async function apiFetch<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(opts.headers as Record<string, string> || {}) };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  const res = await fetch(`${API_BASE}${path}`, { ...opts, headers });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || `API error ${res.status}`);
  }
  return res.json();
}

export const api = {
  campaigns: {
    list: (limit = 20, offset = 0) => apiFetch<Campaign[]>(`/api/campaigns?limit=${limit}&offset=${offset}`),
    get: (id: string) => apiFetch<Campaign>(`/api/campaigns/${id}`),
    create: (data: CreateCampaignRequest) => apiFetch<Campaign>('/api/campaigns', { method: 'POST', body: JSON.stringify(data) }),
    update: (id: string, status: string) => apiFetch<Campaign>(`/api/campaigns/${id}`, { method: 'PATCH', body: JSON.stringify({ status }) }),
    fund: (id: string, amount_sats: number) =>
      apiFetch<{ invoice: string; quote_id: string; amount_sats: number }>(`/api/campaigns/${id}/fund`, { method: 'POST', body: JSON.stringify({ amount_sats }) }),
    checkFunding: (id: string, quoteId: string) =>
      apiFetch<{ paid: boolean; amount_credited_sats: number }>(`/api/campaigns/${id}/fund/${quoteId}`),
  },
  submissions: {
    list: (limit = 20, offset = 0) => apiFetch<Submission[]>(`/api/submissions?limit=${limit}&offset=${offset}`),
    get: (id: string) => apiFetch<Submission>(`/api/submissions/${id}`),
    create: (data: CreateSubmissionRequest) => apiFetch<Submission>('/api/submissions', { method: 'POST', body: JSON.stringify(data) }),
  },
  dashboard: () => apiFetch<DashboardData>('/api/dashboard'),
  leaderboard: (metric = 'earnings', period = 'week', limit = 50) =>
    apiFetch<{ pubkey: string; trust_level: number; value: number }[]>(
      `/api/leaderboard?metric=${metric}&period=${period}&limit=${limit}`
    ),
  socialProof: () =>
    apiFetch<{ clippers_this_week: number; sats_earned_this_week: number }>(
      '/api/stats/social-proof'
    ),
  wallet: {
    balance: () => apiFetch<{ balance_sats: number }>('/api/wallet/balance'),
    withdraw: (invoice: string, amount_sats: number) => apiFetch<{ status: string; amount_sats: number }>('/api/wallet/withdraw', { method: 'POST', body: JSON.stringify({ invoice, amount_sats }) }),
    history: () => apiFetch<{ transactions: TransactionRecord[] }>('/api/wallet/history'),
  },
  analytics: {
    overview: () => apiFetch<AnalyticsOverview>('/api/analytics/overview'),
    submission: (id: string) => apiFetch<SubmissionAnalytics>(`/api/analytics/submission/${id}`),
  },
};

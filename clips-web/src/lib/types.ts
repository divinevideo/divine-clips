export interface Campaign {
  id: string;
  creator_pubkey: string;
  title: string;
  budget_total_sats: number;
  budget_remaining_sats: number;
  cpm_sats: number;
  status: string;
  target_platforms: string[];
  content_refs: string[];
  guidelines: string | null;
  expires_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface Submission {
  id: string;
  campaign_id: string;
  clipper_pubkey: string;
  external_url: string;
  platform: string;
  status: string;
  total_verified_views: number;
  total_paid_sats: number;
  submitted_at: string;
  last_verified_at: string | null;
}

export interface DashboardData {
  trust_level: number;
  total_verified_views: number;
  total_earned_sats: number;
  balance_sats: number;
  active_submissions: number;
  weekly_views_used: number;
  weekly_views_limit: number;
}

export interface TransactionRecord {
  amount_sats: number;
  transaction_type: string;
  created_at: string;
}

export interface CreateCampaignRequest {
  title: string;
  budget_sats: number;
  cpm_sats: number;
  target_platforms: string[];
  content_refs: string[];
  guidelines?: string;
  expires_at?: string;
}

export interface CreateSubmissionRequest {
  campaign_id: string;
  external_url: string;
  platform: string;
}

export interface DailyViews {
  date: string;
  views: number;
}

export interface PlatformStats {
  platform: string;
  views: number;
  earned_sats: number;
}

export interface AnalyticsOverview {
  daily_views: DailyViews[];
  by_platform: PlatformStats[];
}

export interface ViewSnapshot {
  timestamp: number;
  view_count: number;
}

export interface PayoutPoint {
  timestamp: string;
  amount_sats: number;
  cumulative_sats: number;
}

export interface SubmissionAnalytics {
  snapshots: ViewSnapshot[];
  payouts: PayoutPoint[];
}

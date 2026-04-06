export interface ViewCountResult {
  viewCount: number;
  source: string;
}

export interface Submission {
  id: string;
  campaign_id: string;
  clipper_pubkey: string;
  external_url: string;
  platform: string;
  total_verified_views: number;
}

export interface VerificationResult {
  submission_id: string;
  view_count: number;
  source: string;
  fraud_score: number;
}

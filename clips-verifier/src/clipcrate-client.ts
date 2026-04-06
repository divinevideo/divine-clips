import type { Submission, VerificationResult } from './platforms/types';

export class ClipcrateClient {
  private readonly baseUrl: string;
  private readonly apiToken: string;

  constructor(baseUrl: string, apiToken: string) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.apiToken = apiToken;
  }

  private headers(): HeadersInit {
    return {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.apiToken}`,
    };
  }

  /**
   * Fetches submissions that are pending view-count verification.
   */
  async getPendingSubmissions(): Promise<Submission[]> {
    const url = `${this.baseUrl}/internal/submissions/pending-verification`;
    const response = await fetch(url, {
      method: 'GET',
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new Error(
        `Failed to fetch pending submissions: ${response.status} ${response.statusText}`,
      );
    }

    const data = await response.json() as { submissions: Submission[] };
    return data.submissions ?? [];
  }

  /**
   * Posts a verification result back to clipcrate for a given submission.
   */
  async postVerificationResult(result: VerificationResult): Promise<void> {
    const url = `${this.baseUrl}/internal/submissions/${result.submission_id}/verify`;
    const response = await fetch(url, {
      method: 'POST',
      headers: this.headers(),
      body: JSON.stringify({
        view_count: result.view_count,
        source: result.source,
        fraud_score: result.fraud_score,
      }),
    });

    if (!response.ok) {
      throw new Error(
        `Failed to post verification result for ${result.submission_id}: ${response.status} ${response.statusText}`,
      );
    }
  }
}

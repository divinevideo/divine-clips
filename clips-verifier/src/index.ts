import { ClipcrateClient } from './clipcrate-client';
import { calculateFraudScore } from './fraud';
import { getPhylloViews } from './phyllo';
import { getYouTubeViews } from './platforms/youtube';
import { getTikTokViews, isTikTokUrl } from './platforms/tiktok';
import type { Submission, VerificationResult, ViewCountResult } from './platforms/types';

export interface Env {
  CLIPCRATE_API_URL: string;
  CLIPCRATE_API_TOKEN: string;
  YOUTUBE_API_KEY: string;
}

/**
 * Poll a single submission across available platform APIs.
 * Falls back to Phyllo if no platform-specific result is obtained.
 */
async function pollViewCount(
  submission: Submission,
  env: Env,
): Promise<ViewCountResult | null> {
  const { external_url, platform } = submission;
  let result: ViewCountResult | null = null;

  if (platform === 'youtube' || external_url.includes('youtube.com') || external_url.includes('youtu.be')) {
    result = await getYouTubeViews(external_url, env.YOUTUBE_API_KEY);
  } else if (platform === 'tiktok' || isTikTokUrl(external_url)) {
    result = await getTikTokViews(external_url);
  }

  if (!result) {
    // Fall back to Phyllo for unsupported or failed platforms
    result = await getPhylloViews(external_url, platform);
  }

  return result;
}

/**
 * Scheduled handler — runs on cron trigger (every 6 hours).
 */
async function handleScheduled(env: Env): Promise<void> {
  const client = new ClipcrateClient(env.CLIPCRATE_API_URL, env.CLIPCRATE_API_TOKEN);

  let submissions: Submission[];
  try {
    submissions = await client.getPendingSubmissions();
  } catch (err) {
    console.error('[clips-verifier] Failed to fetch pending submissions:', err);
    return;
  }

  console.log(`[clips-verifier] Processing ${submissions.length} pending submission(s)`);

  for (const submission of submissions) {
    try {
      const viewResult = await pollViewCount(submission, env);

      if (!viewResult) {
        console.warn(`[clips-verifier] No view count obtainable for submission ${submission.id}`);
        continue;
      }

      const fraudScore = calculateFraudScore(
        viewResult.viewCount,
        submission.total_verified_views,
        6, // interval matches cron schedule
      );

      const result: VerificationResult = {
        submission_id: submission.id,
        view_count: viewResult.viewCount,
        source: viewResult.source,
        fraud_score: fraudScore,
      };

      await client.postVerificationResult(result);
      console.log(
        `[clips-verifier] Verified submission ${submission.id}: ${viewResult.viewCount} views, fraud_score=${fraudScore}`,
      );
    } catch (err) {
      console.error(`[clips-verifier] Error processing submission ${submission.id}:`, err);
    }
  }
}

export default {
  async scheduled(_controller: ScheduledController, env: Env, _ctx: ExecutionContext): Promise<void> {
    await handleScheduled(env);
  },
} satisfies ExportedHandler<Env>;

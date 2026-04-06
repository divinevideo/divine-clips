<script lang="ts">
  import type { PageData } from './$types';
  import { budgetPercent, platformLabel, timeAgo, formatSats, formatMoney, formatCpm, formatMoneyFull } from '$lib/utils';

  let { data }: { data: PageData } = $props();

  const platformColors: Record<string, string> = {
    tiktok: 'bg-cyan-900 text-cyan-300 border border-cyan-700',
    instagram: 'bg-pink-900 text-pink-300 border border-pink-700',
    youtube: 'bg-red-900 text-red-300 border border-red-700',
    x: 'bg-gray-700 text-gray-300 border border-gray-500',
  };

  function budgetBarColor(pct: number): string {
    if (pct > 50) return 'bg-green-500';
    if (pct > 20) return 'bg-yellow-400';
    return 'bg-red-500';
  }
</script>

<svelte:head>
  <title>Campaigns — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white px-4 py-10">
  <div class="max-w-6xl mx-auto">
    <h1 class="text-3xl font-bold mb-2">Active Campaigns</h1>
    <p class="text-gray-400 mb-8">Browse campaigns and start earning sats by clipping content.</p>

    {#if data.campaigns.length === 0}
      <div class="flex flex-col items-center justify-center py-24 text-center">
        <div class="text-5xl mb-4">🎬</div>
        <h2 class="text-xl font-semibold text-gray-300 mb-2">No active campaigns yet</h2>
        <p class="text-gray-500 max-w-sm">Check back soon — creators are setting up campaigns to reward clippers like you.</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each data.campaigns as campaign (campaign.id)}
          {@const pct = budgetPercent(campaign)}
          <a
            href="/campaigns/{campaign.id}"
            class="block bg-gray-900 rounded-xl border border-gray-800 hover:bg-gray-800 hover:border-gray-700 transition-all duration-200 shadow-lg p-5 no-underline"
          >
            <h2 class="text-lg font-bold text-white mb-1 leading-tight">{campaign.title}</h2>

            <p class="text-sm text-gray-400 mb-3">
              <span class="text-yellow-400 font-semibold">{formatCpm(campaign.cpm_sats)}</span>
              <span class="text-gray-500"> / 1K views</span>
            </p>

            <!-- Budget progress bar -->
            <div class="mb-3">
              <div class="flex justify-between text-xs text-gray-400 mb-1">
                <span>Budget remaining</span>
                <span>{pct}%</span>
              </div>
              <div class="w-full bg-gray-700 rounded-full h-2">
                <div
                  class="h-2 rounded-full transition-all {budgetBarColor(pct)}"
                  style="width: {pct}%"
                ></div>
              </div>
              <p class="text-xs text-gray-500 mt-1">{formatMoneyFull(campaign.budget_remaining_sats)} remaining</p>
            </div>

            <!-- Platform badges -->
            {#if campaign.target_platforms.length > 0}
              <div class="flex flex-wrap gap-1.5 mb-3">
                {#each campaign.target_platforms as platform}
                  <span class="text-xs px-2 py-0.5 rounded-full font-medium {platformColors[platform] || 'bg-gray-700 text-gray-300 border border-gray-600'}">
                    {platformLabel(platform)}
                  </span>
                {/each}
              </div>
            {/if}

            <p class="text-xs text-gray-500">Created {timeAgo(campaign.created_at)}</p>
          </a>
        {/each}
      </div>
    {/if}
  </div>
</div>

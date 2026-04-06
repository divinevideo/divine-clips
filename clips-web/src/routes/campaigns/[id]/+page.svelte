<script lang="ts">
  import type { PageData } from './$types';
  import { budgetPercent, platformLabel, timeAgo, formatSats } from '$lib/utils';

  let { data }: { data: PageData } = $props();

  const campaign = $derived(data.campaign);

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

  function truncatePubkey(pk: string): string {
    if (pk.length <= 16) return pk;
    return pk.slice(0, 8) + '...' + pk.slice(-8);
  }

  const pct = $derived(budgetPercent(data.campaign));
</script>

<svelte:head>
  <title>{campaign.title} — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white px-4 py-10">
  <div class="max-w-3xl mx-auto">

    <!-- Back link -->
    <a href="/campaigns" class="text-sm text-purple-400 hover:text-purple-300 mb-6 inline-block transition-colors">
      ← Back to campaigns
    </a>

    <!-- Title -->
    <h1 class="text-3xl font-bold mb-2">{campaign.title}</h1>

    <!-- Creator pubkey -->
    <p class="text-sm text-gray-500 mb-6 font-mono">
      Creator: <span class="text-gray-400">{truncatePubkey(campaign.creator_pubkey)}</span>
    </p>

    <!-- Stats row -->
    <div class="grid grid-cols-2 sm:grid-cols-3 gap-4 mb-6">
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-4">
        <p class="text-xs text-gray-500 uppercase tracking-wide mb-1">CPM Rate</p>
        <p class="text-lg font-bold text-yellow-400">{campaign.cpm_sats.toLocaleString()} sats</p>
        <p class="text-xs text-gray-500">per 1K views</p>
      </div>

      <div class="bg-gray-900 rounded-xl border border-gray-800 p-4">
        <p class="text-xs text-gray-500 uppercase tracking-wide mb-1">Budget Remaining</p>
        <p class="text-lg font-bold text-white">{formatSats(campaign.budget_remaining_sats)}</p>
        <p class="text-xs text-gray-500">of {formatSats(campaign.budget_total_sats)}</p>
      </div>

      <div class="bg-gray-900 rounded-xl border border-gray-800 p-4">
        <p class="text-xs text-gray-500 uppercase tracking-wide mb-1">Status</p>
        <p class="text-lg font-bold capitalize {campaign.status === 'active' ? 'text-green-400' : 'text-gray-400'}">{campaign.status}</p>
        <p class="text-xs text-gray-500">Created {timeAgo(campaign.created_at)}</p>
      </div>

      {#if campaign.expires_at}
        <div class="bg-gray-900 rounded-xl border border-gray-800 p-4">
          <p class="text-xs text-gray-500 uppercase tracking-wide mb-1">Expires</p>
          <p class="text-base font-semibold text-white">{new Date(campaign.expires_at).toLocaleDateString()}</p>
        </div>
      {/if}
    </div>

    <!-- Budget progress bar -->
    <div class="bg-gray-900 rounded-xl border border-gray-800 p-4 mb-6">
      <div class="flex justify-between text-sm text-gray-400 mb-2">
        <span>Budget progress</span>
        <span class="font-medium">{pct}% remaining</span>
      </div>
      <div class="w-full bg-gray-700 rounded-full h-3">
        <div
          class="h-3 rounded-full transition-all {budgetBarColor(pct)}"
          style="width: {pct}%"
        ></div>
      </div>
    </div>

    <!-- Target platforms -->
    {#if campaign.target_platforms.length > 0}
      <div class="mb-6">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-2">Target Platforms</h2>
        <div class="flex flex-wrap gap-2">
          {#each campaign.target_platforms as platform}
            <span class="px-3 py-1 rounded-full text-sm font-medium {platformColors[platform] || 'bg-gray-700 text-gray-300 border border-gray-600'}">
              {platformLabel(platform)}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Guidelines -->
    {#if campaign.guidelines}
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-5 mb-6">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3">Guidelines</h2>
        <p class="text-gray-200 whitespace-pre-wrap leading-relaxed">{campaign.guidelines}</p>
      </div>
    {/if}

    <!-- Content refs -->
    {#if campaign.content_refs.length > 0}
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-5 mb-8">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3">Source Content</h2>
        <ul class="space-y-2">
          {#each campaign.content_refs as ref, i}
            <li class="flex items-center gap-3">
              <span class="text-xs text-gray-500 w-5 shrink-0">{i + 1}.</span>
              <a
                href={ref}
                target="_blank"
                rel="noopener noreferrer"
                class="text-purple-400 hover:text-purple-300 text-sm font-mono truncate transition-colors"
                title={ref}
              >
                {ref.length > 60 ? ref.slice(0, 60) + '...' : ref}
              </a>
              <a
                href={ref}
                target="_blank"
                rel="noopener noreferrer"
                class="ml-auto shrink-0 text-xs text-gray-500 hover:text-gray-300 border border-gray-700 hover:border-gray-500 px-2 py-0.5 rounded transition-colors"
              >
                Download
              </a>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <!-- CTA -->
    <a
      href="/submit?campaign={campaign.id}"
      class="block w-full text-center bg-purple-600 hover:bg-purple-500 text-white font-bold text-lg py-4 rounded-xl transition-colors shadow-lg shadow-purple-900/30"
    >
      Claim &amp; Clip
    </a>

  </div>
</div>

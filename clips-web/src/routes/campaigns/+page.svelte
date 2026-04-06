<script lang="ts">
  import { onMount } from 'svelte';
  import type { PageData } from './$types';
  import { budgetPercent, platformLabel, timeAgo, formatSats, formatMoney, formatCpm, formatMoneyFull } from '$lib/utils';

  let { data }: { data: PageData } = $props();

  interface PopularVideo {
    id: string;
    title: string;
    thumbnail?: string;
    author_pubkey: string;
    author_name?: string;
    created_at: string;
    views?: number;
  }

  let popularVideos = $state<PopularVideo[]>([]);
  let loadingPopular = $state(false);

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

  onMount(async () => {
    // If no campaigns, load popular DiVine videos as open clips
    if (data.campaigns.length === 0) {
      loadingPopular = true;
      try {
        // Funnelcake API at relay.divine.video
        const res = await fetch('https://relay.divine.video/api/videos?sort=popular&limit=24');
        if (res.ok) {
          const result = await res.json();
          // Response is Vec<TrendingVideo> — array directly
          const videos = Array.isArray(result) ? result : (result.videos || []);
          popularVideos = videos.map((v: any) => ({
            id: v.id || v.d_tag,
            title: v.title || v.content || 'Untitled',
            thumbnail: v.thumbnail,
            author_pubkey: v.pubkey || '',
            author_name: v.author_name || '',
            created_at: v.created_at || v.published_at || '',
            views: v.views || v.loops || 0,
          }));
        }
      } catch {
        // Relay API not available — show placeholder content
      } finally {
        loadingPopular = false;
      }
    }
  });

  function truncatePubkey(pubkey: string): string {
    if (pubkey.length <= 12) return pubkey;
    return pubkey.slice(0, 8) + '...' + pubkey.slice(-4);
  }
</script>

<svelte:head>
  <title>Campaigns — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white px-4 py-10">
  <div class="max-w-6xl mx-auto">

    {#if data.campaigns.length > 0}
      <!-- Active Campaigns -->
      <h1 class="text-3xl font-bold mb-2">Active Campaigns</h1>
      <p class="text-gray-400 mb-8">Creators are paying for clips. Pick a campaign, share the content, earn for every view.</p>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mb-16">
        {#each data.campaigns as campaign (campaign.id)}
          {@const pct = budgetPercent(campaign)}
          <a
            href="/campaigns/{campaign.id}"
            class="block bg-gray-900 rounded-xl border border-gray-800 hover:bg-gray-800 hover:border-gray-700 transition-all duration-200 shadow-lg p-5 no-underline"
          >
            <h2 class="text-lg font-bold text-white mb-1 leading-tight">{campaign.title}</h2>

            <p class="text-sm text-gray-400 mb-3">
              <span class="text-yellow-400 font-semibold">{formatCpm(campaign.cpm_sats)}</span>
            </p>

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

    <!-- Popular DiVine Videos — Always Clippable -->
    <div>
      <div class="flex items-baseline justify-between mb-2">
        <h2 class="text-2xl font-bold text-white">
          {#if data.campaigns.length > 0}
            Or clip any DiVine video
          {:else}
            Popular DiVine Videos
          {/if}
        </h2>
        <span class="text-yellow-400 text-sm font-medium">$1.00 / 1K views</span>
      </div>
      <p class="text-gray-400 mb-6">
        Every DiVine video is clippable at the base rate. Pick one you love, share it on your platforms, and earn for the views you bring.
      </p>

      {#if loadingPopular}
        <div class="text-center py-12">
          <div class="animate-spin w-8 h-8 border-2 border-purple-600 border-t-transparent rounded-full mx-auto mb-3"></div>
          <p class="text-gray-500">Loading popular videos...</p>
        </div>
      {:else if popularVideos.length > 0}
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
          {#each popularVideos as video (video.id)}
            <a
              href="/submit?video={video.id}"
              class="group relative bg-gray-900 rounded-xl border border-gray-800 hover:border-purple-700 transition-all overflow-hidden"
            >
              {#if video.thumbnail}
                <img
                  src={video.thumbnail}
                  alt={video.title}
                  class="w-full aspect-square object-cover group-hover:scale-105 transition-transform duration-300"
                />
              {:else}
                <div class="w-full aspect-square bg-gray-800 flex items-center justify-center">
                  <div class="text-center">
                    <span class="text-4xl">&#x25B6;</span>
                    <p class="text-gray-600 text-xs mt-1">6s loop</p>
                  </div>
                </div>
              {/if}

              <!-- Overlay -->
              <div class="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3">
                <p class="text-white text-sm font-medium truncate">{video.title}</p>
                {#if video.author_name}
                  <p class="text-gray-400 text-xs">by {video.author_name}</p>
                {:else if video.author_pubkey}
                  <p class="text-gray-400 text-xs">by {truncatePubkey(video.author_pubkey)}</p>
                {/if}
              </div>

              <!-- Clip badge -->
              <div class="absolute top-2 right-2 bg-purple-600 text-white text-xs font-bold px-2 py-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity">
                Clip it
              </div>
            </a>
          {/each}
        </div>
      {:else}
        <!-- No API available — show call to action -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <div class="bg-gray-900 rounded-xl border border-gray-800 p-6 text-center">
            <div class="text-4xl mb-3">&#x1F3AC;</div>
            <h3 class="text-white font-bold mb-2">Browse divine.video</h3>
            <p class="text-gray-400 text-sm mb-4">Find 6-second loops you love on the DiVine app or website.</p>
            <a href="https://divine.video" class="text-purple-400 hover:underline text-sm">Go to divine.video &rarr;</a>
          </div>
          <div class="bg-gray-900 rounded-xl border border-gray-800 p-6 text-center">
            <div class="text-4xl mb-3">&#x1F4F1;</div>
            <h3 class="text-white font-bold mb-2">Share on your platforms</h3>
            <p class="text-gray-400 text-sm mb-4">Post loops on TikTok, Reels, Shorts, or X — use your own style.</p>
          </div>
          <div class="bg-gray-900 rounded-xl border border-gray-800 p-6 text-center">
            <div class="text-4xl mb-3">&#x26A1;</div>
            <h3 class="text-white font-bold mb-2">Submit & earn</h3>
            <p class="text-gray-400 text-sm mb-4">Paste your clip link here and earn $1.00 per 1,000 verified views.</p>
            <a href="/submit" class="text-purple-400 hover:underline text-sm">Submit a clip &rarr;</a>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

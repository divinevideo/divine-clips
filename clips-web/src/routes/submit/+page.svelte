<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { PLATFORMS, platformLabel } from '$lib/utils';
  import type { Campaign } from '$lib/types';

  let campaigns = $state<Campaign[]>([]);
  let loadingCampaigns = $state(true);

  let selectedCampaignId = $state('');
  let selectedPlatform = $state('');
  let url = $state('');

  let submitting = $state(false);
  let success = $state(false);
  let errorMsg = $state('');
  let validationError = $state('');

  // Pre-fill from URL params
  let prefilledVideoId = $state('');
  let prefilledVideoTitle = $state('');

  const platformDomains: Record<string, string[]> = {
    tiktok: ['tiktok.com', 'vm.tiktok.com'],
    instagram: ['instagram.com', 'instagr.am'],
    youtube: ['youtube.com', 'youtu.be'],
    x: ['x.com', 'twitter.com', 't.co'],
  };

  const platformColors: Record<string, string> = {
    tiktok: 'border-cyan-600 bg-cyan-900/40 text-cyan-300',
    instagram: 'border-pink-600 bg-pink-900/40 text-pink-300',
    youtube: 'border-red-600 bg-red-900/40 text-red-300',
    x: 'border-gray-500 bg-gray-700/40 text-gray-300',
  };

  onMount(async () => {
    // Pre-fill campaign and video from URL query params
    const campaignParam = $page.url.searchParams.get('campaign');
    const videoParam = $page.url.searchParams.get('video');

    if (videoParam) {
      prefilledVideoId = videoParam;
      // Try to fetch video title from relay API
      try {
        const res = await fetch(`https://relay.divine.video/api/videos/${videoParam}`);
        if (res.ok) {
          const v = await res.json();
          prefilledVideoTitle = v.title || v.content || videoParam;
        } else {
          prefilledVideoTitle = videoParam;
        }
      } catch {
        prefilledVideoTitle = videoParam;
      }
    }

    try {
      campaigns = await api.campaigns.list();
      if (campaignParam && campaigns.some(c => c.id === campaignParam)) {
        selectedCampaignId = campaignParam;
      }
    } catch {
      // silently fail, campaigns will be empty
    }
    loadingCampaigns = false;
  });

  function validateUrl(): boolean {
    validationError = '';
    if (!url.trim()) {
      validationError = 'Please enter a URL.';
      return false;
    }
    let parsed: URL;
    try {
      parsed = new URL(url.trim());
    } catch {
      validationError = 'Please enter a valid URL (include https://).';
      return false;
    }
    if (!selectedPlatform) {
      validationError = 'Please select a platform.';
      return false;
    }
    const allowed = platformDomains[selectedPlatform] || [];
    const hostname = parsed.hostname.replace(/^www\./, '');
    if (!allowed.some(d => hostname === d || hostname.endsWith('.' + d))) {
      validationError = `URL doesn't look like a ${platformLabel(selectedPlatform)} link. Expected domains: ${allowed.join(', ')}`;
      return false;
    }
    return true;
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    errorMsg = '';
    success = false;

    if (!selectedCampaignId) {
      validationError = 'Please select a campaign.';
      return;
    }
    if (!validateUrl()) return;

    submitting = true;
    try {
      await api.submissions.create({
        campaign_id: selectedCampaignId,
        external_url: url.trim(),
        platform: selectedPlatform,
      });
      success = true;
      url = '';
    } catch (err: unknown) {
      errorMsg = err instanceof Error ? err.message : 'Something went wrong. Please try again.';
    } finally {
      submitting = false;
    }
  }

  function handleUrlInput() {
    // Auto-detect platform from URL
    if (!url) return;
    let parsed: URL;
    try {
      parsed = new URL(url);
    } catch {
      return;
    }
    const hostname = parsed.hostname.replace(/^www\./, '');
    for (const [platform, domains] of Object.entries(platformDomains)) {
      if (domains.some(d => hostname === d || hostname.endsWith('.' + d))) {
        selectedPlatform = platform;
        break;
      }
    }
  }
</script>

<svelte:head>
  <title>Submit Clip — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white flex items-start justify-center px-4 py-12">
  <div class="w-full max-w-lg">

    <a href="/campaigns" class="text-sm text-purple-400 hover:text-purple-300 mb-6 inline-block transition-colors">
      ← Back to campaigns
    </a>

    <h1 class="text-2xl font-bold mb-1">Submit a Clip</h1>
    <p class="text-gray-400 text-sm mb-4">Paste the link to your published clip and start earning sats.</p>

    {#if prefilledVideoId}
      <div class="bg-purple-600/10 border border-purple-500/30 rounded-xl px-4 py-3 mb-6 flex items-center gap-3">
        <span class="text-purple-400 text-lg">🎬</span>
        <div>
          <p class="text-xs text-gray-500 uppercase tracking-wider mb-0.5">Clipping</p>
          <p class="text-white text-sm font-medium truncate">{prefilledVideoTitle || prefilledVideoId}</p>
        </div>
      </div>
    {/if}

    {#if success}
      <div class="bg-green-900/40 border border-green-700 rounded-xl p-6 text-center">
        <div class="text-4xl mb-3">✓</div>
        <h2 class="text-lg font-semibold text-green-300 mb-1">Clip submitted!</h2>
        <p class="text-green-400 text-sm">We'll start tracking views and paying out sats as they roll in.</p>
        <button
          class="mt-5 text-sm text-purple-400 hover:text-purple-300 underline transition-colors"
          onclick={() => { success = false; }}
        >
          Submit another clip
        </button>
      </div>
    {:else}
      <form onsubmit={handleSubmit} class="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-6">

        <!-- Campaign selector -->
        <div>
          <label for="campaign" class="block text-sm font-medium text-gray-300 mb-2">Campaign</label>
          {#if loadingCampaigns}
            <div class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-gray-500 text-sm">
              Loading campaigns...
            </div>
          {:else}
            <select
              id="campaign"
              bind:value={selectedCampaignId}
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent"
            >
              <option value="">Select a campaign…</option>
              {#each campaigns as c}
                <option value={c.id}>{c.title}</option>
              {/each}
            </select>
          {/if}
        </div>

        <!-- Platform selector -->
        <div>
          <p class="block text-sm font-medium text-gray-300 mb-2">Platform</p>
          <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
            {#each PLATFORMS as platform}
              <button
                type="button"
                onclick={() => { selectedPlatform = platform; }}
                class="border rounded-lg px-3 py-2 text-sm font-medium transition-all
                  {selectedPlatform === platform
                    ? (platformColors[platform] || 'border-purple-600 bg-purple-900/40 text-purple-300')
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600 hover:text-gray-300'}"
              >
                {platformLabel(platform)}
              </button>
            {/each}
          </div>
        </div>

        <!-- URL input -->
        <div>
          <label for="clip-url" class="block text-sm font-medium text-gray-300 mb-2">Clip URL</label>
          <input
            id="clip-url"
            type="url"
            bind:value={url}
            oninput={handleUrlInput}
            placeholder="https://www.tiktok.com/@user/video/..."
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent"
          />
          {#if selectedPlatform}
            <p class="text-xs text-gray-500 mt-1">
              Expected: {platformDomains[selectedPlatform]?.join(', ')}
            </p>
          {/if}
        </div>

        <!-- Validation error -->
        {#if validationError}
          <div class="bg-red-900/40 border border-red-700 rounded-lg px-4 py-2.5 text-red-300 text-sm">
            {validationError}
          </div>
        {/if}

        <!-- API error -->
        {#if errorMsg}
          <div class="bg-red-900/40 border border-red-700 rounded-lg px-4 py-2.5 text-red-300 text-sm">
            {errorMsg}
          </div>
        {/if}

        <!-- Submit button -->
        <button
          type="submit"
          disabled={submitting}
          class="w-full bg-purple-600 hover:bg-purple-500 disabled:bg-purple-900 disabled:text-purple-400 text-white font-bold py-3 rounded-xl transition-colors shadow-lg shadow-purple-900/30 text-base"
        >
          {submitting ? 'Submitting…' : 'Submit Clip'}
        </button>

      </form>
    {/if}
  </div>
</div>

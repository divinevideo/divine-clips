<script lang="ts">
  import type { PageData } from './$types';
  import { budgetPercent, platformLabel, timeAgo, formatSats } from '$lib/utils';
  import { isAuthenticated } from '$lib/stores/auth';
  import { api } from '$lib/api';
  import { invalidateAll } from '$app/navigation';
  import VideoCard from '$lib/components/VideoCard.svelte';

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

  // Funding modal state
  let showFundModal = $state(false);
  let fundAmountSats = $state(10000);
  let invoice = $state('');
  let quoteId = $state('');
  let fundingStatus = $state<'idle' | 'loading' | 'invoice' | 'polling' | 'success' | 'error'>('idle');
  let fundingError = $state('');
  let amountCredited = $state(0);
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let copied = $state(false);

  function openFundModal() {
    showFundModal = true;
    fundingStatus = 'idle';
    invoice = '';
    quoteId = '';
    fundingError = '';
    amountCredited = 0;
    copied = false;
  }

  function closeFundModal() {
    showFundModal = false;
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function getInvoice() {
    fundingStatus = 'loading';
    fundingError = '';
    try {
      const result = await api.campaigns.fund(campaign.id, fundAmountSats);
      invoice = result.invoice;
      quoteId = result.quote_id;
      fundingStatus = 'invoice';
      startPolling();
    } catch (err: unknown) {
      fundingError = err instanceof Error ? err.message : 'Failed to get invoice';
      fundingStatus = 'error';
    }
  }

  function startPolling() {
    fundingStatus = 'polling';
    pollTimer = setInterval(async () => {
      try {
        const result = await api.campaigns.checkFunding(campaign.id, quoteId);
        if (result.paid) {
          fundingStatus = 'success';
          amountCredited = result.amount_credited_sats;
          if (pollTimer) {
            clearInterval(pollTimer);
            pollTimer = null;
          }
          await invalidateAll();
        }
      } catch {
        // Silently retry on poll errors
      }
    }, 3000);
  }

  async function copyInvoice() {
    try {
      await navigator.clipboard.writeText(invoice);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    } catch {
      // Fallback: select the text
    }
  }
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
      {@const firstRef = campaign.content_refs[0]}
      {@const videoUrl = firstRef.startsWith('http') ? firstRef : `https://media.divine.video/${firstRef}`}
      <div class="mb-8">
        <h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">Source Content</h2>

        <!-- First content ref rendered as a video player -->
        <div class="max-w-xs mx-auto mb-6">
          <a href="/submit?campaign={campaign.id}&video={firstRef}">
            <VideoCard
              src={videoUrl}
              title={campaign.title}
              subtitle="Tap to clip this video"
            />
          </a>
        </div>

        <!-- Remaining refs as download links -->
        {#if campaign.content_refs.length > 1}
          <div class="bg-gray-900 rounded-xl border border-gray-800 p-5">
            <p class="text-xs text-gray-500 mb-3">Additional source files:</p>
            <ul class="space-y-2">
              {#each campaign.content_refs.slice(1) as ref, i}
                <li class="flex items-center gap-3">
                  <span class="text-xs text-gray-500 w-5 shrink-0">{i + 2}.</span>
                  <a
                    href={ref.startsWith('http') ? ref : `https://media.divine.video/${ref}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-purple-400 hover:text-purple-300 text-sm font-mono truncate transition-colors"
                    title={ref}
                  >
                    {ref.length > 60 ? ref.slice(0, 60) + '...' : ref}
                  </a>
                  <a
                    href={ref.startsWith('http') ? ref : `https://media.divine.video/${ref}`}
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
      </div>
    {/if}

    <!-- CTA -->
    <a
      href="/submit?campaign={campaign.id}"
      class="block w-full text-center bg-purple-600 hover:bg-purple-500 text-white font-bold text-lg py-4 rounded-xl transition-colors shadow-lg shadow-purple-900/30"
    >
      Claim &amp; Clip
    </a>

    <!-- Fund Campaign Button -->
    {#if $isAuthenticated}
      <button
        onclick={openFundModal}
        class="block w-full text-center bg-green-600 hover:bg-green-500 text-white font-bold text-lg py-4 rounded-xl transition-colors shadow-lg shadow-green-900/30 mt-4"
      >
        Fund Campaign
      </button>
    {/if}

  </div>
</div>

<!-- Fund Campaign Modal -->
{#if showFundModal}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    onclick={(e) => { if (e.target === e.currentTarget) closeFundModal(); }}
  >
    <div class="bg-gray-900 border border-gray-700 rounded-2xl p-6 w-full max-w-md mx-4 shadow-2xl">
      <div class="flex items-center justify-between mb-5">
        <h2 class="text-xl font-bold text-white">Fund Campaign</h2>
        <button
          onclick={closeFundModal}
          class="text-gray-400 hover:text-white transition-colors text-2xl leading-none"
        >&times;</button>
      </div>

      {#if fundingStatus === 'success'}
        <!-- Success state -->
        <div class="text-center py-6">
          <div class="text-5xl mb-4">&#9889;</div>
          <h3 class="text-xl font-bold text-green-400 mb-2">Payment Received!</h3>
          <p class="text-gray-300">
            <span class="text-yellow-400 font-bold">{amountCredited.toLocaleString()}</span> sats credited to this campaign.
          </p>
          <button
            onclick={closeFundModal}
            class="mt-6 w-full bg-purple-600 hover:bg-purple-500 text-white font-semibold py-3 rounded-xl transition-colors"
          >
            Done
          </button>
        </div>
      {:else if fundingStatus === 'error'}
        <!-- Error state -->
        <div class="text-center py-4">
          <p class="text-red-400 mb-4">{fundingError}</p>
          <button
            onclick={() => { fundingStatus = 'idle'; }}
            class="w-full bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 rounded-xl transition-colors"
          >
            Try Again
          </button>
        </div>
      {:else if fundingStatus === 'invoice' || fundingStatus === 'polling'}
        <!-- Invoice display -->
        <div>
          <p class="text-sm text-gray-400 mb-3">
            Pay <span class="text-yellow-400 font-bold">{fundAmountSats.toLocaleString()}</span> sats via Lightning:
          </p>

          <div class="relative">
            <textarea
              readonly
              value={invoice}
              class="w-full bg-gray-950 border border-gray-700 rounded-xl p-3 text-xs text-gray-300 font-mono resize-none h-28 focus:outline-none focus:border-purple-500"
            ></textarea>
            <button
              onclick={copyInvoice}
              class="absolute top-2 right-2 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 px-3 py-1 rounded-lg transition-colors"
            >
              {copied ? 'Copied!' : 'Copy'}
            </button>
          </div>

          {#if fundingStatus === 'polling'}
            <div class="flex items-center gap-2 mt-4 text-sm text-gray-400">
              <div class="w-4 h-4 border-2 border-purple-400 border-t-transparent rounded-full animate-spin"></div>
              Waiting for payment...
            </div>
          {/if}

          <button
            onclick={closeFundModal}
            class="mt-4 w-full bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 rounded-xl transition-colors"
          >
            Cancel
          </button>
        </div>
      {:else}
        <!-- Amount input and get invoice -->
        <div>
          <label for="fund-amount" class="block text-sm text-gray-400 mb-2">Amount (sats)</label>
          <input
            id="fund-amount"
            type="number"
            min="1000"
            step="1000"
            bind:value={fundAmountSats}
            class="w-full bg-gray-950 border border-gray-700 rounded-xl px-4 py-3 text-white text-lg font-mono focus:outline-none focus:border-purple-500 transition-colors mb-4"
          />

          <div class="flex gap-2 mb-4">
            {#each [10000, 50000, 100000, 500000] as preset}
              <button
                onclick={() => { fundAmountSats = preset; }}
                class="flex-1 text-xs py-2 rounded-lg transition-colors {fundAmountSats === preset ? 'bg-purple-600 text-white' : 'bg-gray-800 text-gray-400 hover:bg-gray-700'}"
              >
                {(preset / 1000)}k
              </button>
            {/each}
          </div>

          <button
            onclick={getInvoice}
            disabled={fundingStatus === 'loading' || fundAmountSats < 1}
            class="w-full bg-green-600 hover:bg-green-500 disabled:bg-gray-700 disabled:text-gray-500 text-white font-bold py-3 rounded-xl transition-colors"
          >
            {fundingStatus === 'loading' ? 'Getting Invoice...' : 'Get Lightning Invoice'}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

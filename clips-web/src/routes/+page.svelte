<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { feedEvents, addFeedEvent } from '$lib/stores/feed';
	import { connectToFeed } from '$lib/nostr';
	import { api } from '$lib/api';
	import { formatSats, timeAgo } from '$lib/utils';
	import type { Campaign } from '$lib/types';

	const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3100';

	let recentCampaigns = $state<Campaign[]>([]);
	let loadingCampaigns = $state(true);
	let es: EventSource | null = null;
	let newItemIds = $state(new Set<string>());

	onMount(async () => {
		// Connect to SSE live feed
		es = connectToFeed(API_BASE, (data) => {
			addFeedEvent(data);
			// Mark new item for animation
			if (data.id) {
				newItemIds = new Set([...newItemIds, data.id]);
				setTimeout(() => {
					newItemIds = new Set([...newItemIds].filter(id => id !== data.id));
				}, 1000);
			}
		});

		// Load fallback recent campaigns
		try {
			recentCampaigns = await api.campaigns.list(10, 0);
		} catch {
			recentCampaigns = [];
		} finally {
			loadingCampaigns = false;
		}
	});

	onDestroy(() => {
		es?.close();
	});

	function truncateTitle(title: string, maxLen = 40): string {
		return title.length > maxLen ? title.slice(0, maxLen) + '...' : title;
	}
</script>

<!-- Hero Section -->
<section class="relative -mx-4 sm:-mx-6 lg:-mx-8 -mt-8 mb-12 px-4 sm:px-6 lg:px-8 py-20 bg-gradient-to-b from-purple-900 via-purple-950 to-gray-950 overflow-hidden">
	<!-- Background decoration -->
	<div class="absolute inset-0 overflow-hidden pointer-events-none">
		<div class="absolute -top-40 -right-40 w-96 h-96 bg-purple-700 rounded-full opacity-10 blur-3xl"></div>
		<div class="absolute -bottom-20 -left-20 w-80 h-80 bg-purple-500 rounded-full opacity-10 blur-3xl"></div>
	</div>

	<div class="relative max-w-3xl mx-auto text-center">
		<h1 class="text-5xl sm:text-6xl font-extrabold text-white tracking-tight mb-4">
			DiVine Clips
		</h1>
		<p class="text-xl sm:text-2xl text-purple-200 mb-3 font-medium">
			Earn Bitcoin by sharing 6-second loops
		</p>
		<p class="text-gray-400 mb-10 max-w-xl mx-auto">
			Discover campaigns, share clips on your platforms, and get paid in sats for every verified view.
		</p>
		<div class="flex flex-col sm:flex-row gap-4 justify-center">
			<a
				href="/campaigns"
				class="inline-block px-8 py-3 bg-purple-600 hover:bg-purple-500 text-white font-semibold rounded-lg transition-colors text-lg shadow-lg shadow-purple-900/50"
			>
				Browse Campaigns
			</a>
			<a
				href="/create"
				class="inline-block px-8 py-3 bg-gray-800 hover:bg-gray-700 text-white font-semibold rounded-lg transition-colors text-lg border border-gray-700"
			>
				Create Campaign
			</a>
		</div>
	</div>
</section>

<!-- Stats / Social Proof -->
<section class="mb-12">
	<div class="grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-3xl mx-auto text-center">
		<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
			<div class="text-3xl font-bold text-purple-400 mb-1">6s</div>
			<div class="text-gray-400 text-sm">Short loop format</div>
		</div>
		<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
			<div class="text-3xl font-bold text-purple-400 mb-1">Sats</div>
			<div class="text-gray-400 text-sm">Paid in Bitcoin</div>
		</div>
		<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
			<div class="text-3xl font-bold text-purple-400 mb-1">4</div>
			<div class="text-gray-400 text-sm">Platforms supported</div>
		</div>
	</div>
</section>

<!-- Live Activity Feed -->
<section class="max-w-2xl mx-auto">
	<h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2">
		<span class="inline-block w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
		Live Activity
	</h2>

	{#if $feedEvents.length > 0}
		<div class="space-y-3">
			{#each $feedEvents as event (event.id ?? event)}
				<div
					class="bg-gray-900 border border-gray-800 rounded-lg px-5 py-4 flex items-start gap-4 transition-all duration-500 {newItemIds.has(event.id) ? 'opacity-100 translate-y-0' : ''}"
					style="animation: slideIn 0.4s ease-out"
				>
					<div class="flex-shrink-0 w-8 h-8 rounded-full bg-purple-700 flex items-center justify-center text-white text-xs font-bold mt-0.5">
						N
					</div>
					<div class="flex-1 min-w-0">
						<p class="text-white text-sm font-medium">
							New campaign: <span class="text-purple-300">{truncateTitle(event.title ?? 'Untitled')}</span>
							{#if event.cpm_sats}
								— <span class="text-yellow-400">{formatSats(event.cpm_sats)}/1K views</span>
							{/if}
						</p>
						{#if event.created_at}
							<p class="text-gray-500 text-xs mt-0.5">{timeAgo(event.created_at)}</p>
						{/if}
					</div>
				</div>
			{/each}
		</div>

	{:else if loadingCampaigns}
		<div class="text-center py-12 text-gray-500">
			<div class="animate-spin w-8 h-8 border-2 border-purple-600 border-t-transparent rounded-full mx-auto mb-3"></div>
			Loading campaigns...
		</div>

	{:else if recentCampaigns.length > 0}
		<!-- Fallback: show recent campaigns from API -->
		<p class="text-gray-500 text-sm mb-4">No live events yet — showing recent campaigns</p>
		<div class="space-y-3">
			{#each recentCampaigns as campaign (campaign.id)}
				<a
					href="/campaigns/{campaign.id}"
					class="block bg-gray-900 border border-gray-800 hover:border-purple-700 rounded-lg px-5 py-4 transition-colors group"
				>
					<div class="flex items-start gap-4">
						<div class="flex-shrink-0 w-8 h-8 rounded-full bg-purple-800 flex items-center justify-center text-white text-xs font-bold mt-0.5">
							C
						</div>
						<div class="flex-1 min-w-0">
							<p class="text-white text-sm font-medium group-hover:text-purple-300 transition-colors">
								{truncateTitle(campaign.title)}
							</p>
							<p class="text-gray-500 text-xs mt-0.5">
								<span class="text-yellow-400">{formatSats(campaign.cpm_sats)}/1K views</span>
								· {formatSats(campaign.budget_remaining_sats)} remaining
								· {timeAgo(campaign.created_at)}
							</p>
						</div>
					</div>
				</a>
			{/each}
		</div>

	{:else}
		<div class="text-center py-16 text-gray-500 bg-gray-900 rounded-xl border border-gray-800">
			<p class="text-lg mb-2">No campaigns yet</p>
			<p class="text-sm">Be the first to <a href="/create" class="text-purple-400 hover:underline">create a campaign</a>.</p>
		</div>
	{/if}
</section>

<!-- Social Proof Footer Banner -->
<section class="mt-16 text-center py-10 border-t border-gray-800">
	<p class="text-gray-400 text-lg">
		Join thousands of clippers earning sats — one 6-second loop at a time.
	</p>
	<a href="/campaigns" class="mt-4 inline-block text-purple-400 hover:text-purple-300 text-sm font-medium transition-colors">
		View all campaigns &rarr;
	</a>
</section>

<style>
	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateY(-8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>

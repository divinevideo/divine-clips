<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';

	type LeaderboardEntry = { pubkey: string; trust_level: number; value: number };
	type SocialProof = { clippers_this_week: number; sats_earned_this_week: number };

	let activeTab = $state<'earnings' | 'views' | 'best_clip'>('earnings');
	let activePeriod = $state<'week' | 'month' | 'all'>('week');
	let entries = $state<LeaderboardEntry[]>([]);
	let socialProof = $state<SocialProof | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const tabs = [
		{ id: 'earnings', label: 'Top Earners' },
		{ id: 'views', label: 'Most Views' },
		{ id: 'best_clip', label: 'Best Single Clip' },
	] as const;

	const periods = [
		{ id: 'week', label: 'This Week' },
		{ id: 'month', label: 'This Month' },
		{ id: 'all', label: 'All Time' },
	] as const;

	async function loadLeaderboard() {
		loading = true;
		error = null;
		try {
			entries = await api.leaderboard(activeTab, activePeriod, 50);
		} catch (e) {
			// Network errors (API unreachable) show empty state; API errors show error message
			const msg = e instanceof Error ? e.message : 'Failed to load leaderboard';
			if (msg === 'Failed to fetch' || msg === 'Load failed') {
				entries = [];
			} else {
				error = msg;
			}
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		api.socialProof().then(s => { socialProof = s; }).catch(() => {});
	});

	$effect(() => {
		// Re-fetch when tab or period changes (runs on mount + any change)
		activeTab;
		activePeriod;
		loadLeaderboard();
	});

	function truncateNpub(pubkey: string): string {
		// Display as npub-style truncation (show first 8 + last 4 chars)
		if (pubkey.length <= 16) return pubkey;
		return `${pubkey.slice(0, 8)}...${pubkey.slice(-4)}`;
	}

	function trustBadge(level: number): { label: string; classes: string } {
		if (level >= 4) return { label: 'Gold', classes: 'bg-yellow-500 text-yellow-900' };
		if (level >= 3) return { label: 'Silver', classes: 'bg-gray-400 text-gray-900' };
		return { label: 'Bronze', classes: 'bg-amber-700 text-amber-100' };
	}

	function formatValue(value: number, metric: string): string {
		if (metric === 'earnings') {
			// Convert sats to approximate USD (rough: 1 BTC = $60k, 1 sat = $0.0006)
			const usd = (value * 0.0006).toFixed(2);
			return `${value.toLocaleString()} sats (~$${usd})`;
		}
		return value.toLocaleString() + ' views';
	}

	function formatSats(sats: number): string {
		if (sats >= 1_000_000) return `${(sats / 1_000_000).toFixed(1)}M sats`;
		if (sats >= 1_000) return `${(sats / 1_000).toFixed(1)}k sats`;
		return `${sats} sats`;
	}

	function rankMedal(rank: number): string {
		if (rank === 1) return '🥇';
		if (rank === 2) return '🥈';
		if (rank === 3) return '🥉';
		return `#${rank}`;
	}
</script>

<svelte:head>
	<title>Leaderboard — DiVine Clips</title>
</svelte:head>

<div class="space-y-6">
	<!-- Page header -->
	<div>
		<h1 class="text-3xl font-bold text-white">Leaderboard</h1>
		<p class="mt-1 text-gray-400">Top clippers earning sats on DiVine Clips</p>
	</div>

	<!-- Social proof banner -->
	{#if socialProof}
		<div class="bg-purple-900/40 border border-purple-700/50 rounded-xl px-6 py-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
			<div class="flex items-center gap-3">
				<span class="text-2xl">⚡</span>
				<span class="text-white font-medium">
					<span class="text-purple-300 font-bold">{socialProof.clippers_this_week}</span>
					{socialProof.clippers_this_week === 1 ? 'clipper' : 'clippers'} earned
					<span class="text-yellow-300 font-bold">{formatSats(socialProof.sats_earned_this_week)}</span>
					this week
				</span>
			</div>
			<a
				href="/submit"
				class="inline-block px-4 py-2 bg-purple-600 hover:bg-purple-500 text-white text-sm font-medium rounded-lg transition-colors text-center"
			>
				Start Clipping
			</a>
		</div>
	{/if}

	<!-- Controls row -->
	<div class="flex flex-col sm:flex-row gap-4">
		<!-- Tab buttons -->
		<div class="flex bg-gray-800 rounded-lg p-1 gap-1 flex-1">
			{#each tabs as tab}
				<button
					onclick={() => { activeTab = tab.id; }}
					class="flex-1 px-3 py-2 rounded-md text-sm font-medium transition-colors {activeTab === tab.id ? 'bg-purple-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-700'}"
				>
					{tab.label}
				</button>
			{/each}
		</div>

		<!-- Period toggle -->
		<div class="flex bg-gray-800 rounded-lg p-1 gap-1">
			{#each periods as period}
				<button
					onclick={() => { activePeriod = period.id; }}
					class="px-3 py-2 rounded-md text-sm font-medium transition-colors {activePeriod === period.id ? 'bg-gray-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-700'}"
				>
					{period.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Leaderboard table -->
	<div class="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
		{#if loading}
			<div class="flex items-center justify-center py-16">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
			</div>
		{:else if error}
			<div class="text-center py-16 text-red-400">
				<p class="text-lg font-medium">Failed to load leaderboard</p>
				<p class="text-sm mt-1">{error}</p>
				<button
					onclick={loadLeaderboard}
					class="mt-4 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-white rounded-lg text-sm transition-colors"
				>
					Try Again
				</button>
			</div>
		{:else if entries.length === 0}
			<div class="text-center py-16 text-gray-500">
				<p class="text-4xl mb-4">🏆</p>
				<p class="text-lg font-medium text-gray-300">No clippers ranked yet</p>
				<p class="text-sm mt-1">Submit clips to campaigns and start climbing the leaderboard</p>
				<a
					href="/campaigns"
					class="inline-block mt-4 px-4 py-2 bg-purple-600 hover:bg-purple-500 text-white text-sm font-medium rounded-lg transition-colors"
				>
					Browse Campaigns
				</a>
			</div>
		{:else}
			<table class="w-full">
				<thead>
					<tr class="border-b border-gray-800 text-left">
						<th class="px-6 py-4 text-xs font-semibold text-gray-400 uppercase tracking-wider w-16">Rank</th>
						<th class="px-6 py-4 text-xs font-semibold text-gray-400 uppercase tracking-wider">Clipper</th>
						<th class="px-6 py-4 text-xs font-semibold text-gray-400 uppercase tracking-wider">Trust</th>
						<th class="px-6 py-4 text-xs font-semibold text-gray-400 uppercase tracking-wider text-right">
							{activeTab === 'earnings' ? 'Earnings' : activeTab === 'views' ? 'Total Views' : 'Best Clip Views'}
						</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-gray-800">
					{#each entries as entry, i}
						{@const badge = trustBadge(entry.trust_level)}
						<tr class="hover:bg-gray-800/50 transition-colors {i < 3 ? 'bg-gray-800/20' : ''}">
							<td class="px-6 py-4 text-center">
								<span class="text-lg font-bold {i === 0 ? 'text-yellow-400' : i === 1 ? 'text-gray-300' : i === 2 ? 'text-amber-600' : 'text-gray-500'}">
									{rankMedal(i + 1)}
								</span>
							</td>
							<td class="px-6 py-4">
								<span class="font-mono text-sm text-gray-300">{truncateNpub(entry.pubkey)}</span>
							</td>
							<td class="px-6 py-4">
								<span class="inline-block px-2 py-0.5 rounded text-xs font-semibold {badge.classes}">
									{badge.label}
								</span>
							</td>
							<td class="px-6 py-4 text-right">
								<span class="text-white font-medium">{formatValue(entry.value, activeTab)}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>

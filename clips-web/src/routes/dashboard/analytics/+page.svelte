<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { formatSats, formatViews, platformLabel } from '$lib/utils';
	import { isAuthenticated, initAuth } from '$lib/stores/auth';
	import type { AnalyticsOverview } from '$lib/types';
	import ViewsChart from '$lib/components/charts/ViewsChart.svelte';

	let overview = $state<AnalyticsOverview | null>(null);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		initAuth();
		if (!$isAuthenticated) {
			loading = false;
			return;
		}
		try {
			overview = await api.analytics.overview();
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Failed to load analytics';
			// Show empty state instead of hard error
			overview = { daily_views: [], by_platform: [] };
		} finally {
			loading = false;
		}
	});

	let totalViews = $derived(
		overview?.by_platform.reduce((acc, p) => acc + p.views, 0) ?? 0
	);
	let totalEarned = $derived(
		overview?.by_platform.reduce((acc, p) => acc + p.earned_sats, 0) ?? 0
	);
</script>

<svelte:head>
	<title>Analytics — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white">
	<div class="max-w-5xl mx-auto px-4 py-10">

		<!-- Header -->
		<div class="flex items-center justify-between mb-8">
			<div>
				<h1 class="text-2xl font-bold text-white">Analytics</h1>
				<p class="text-gray-500 text-sm mt-1">Your clip performance over time</p>
			</div>
			<a
				href="/dashboard"
				class="text-purple-400 hover:text-purple-300 text-sm font-medium transition-colors"
			>
				← Back to Dashboard
			</a>
		</div>

		{#if loading}
			<div class="flex items-center justify-center py-32">
				<div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
			</div>

		{:else if !$isAuthenticated}
			<div class="bg-purple-600/10 border border-purple-500/30 rounded-xl p-8 text-center">
				<p class="text-white font-medium text-lg mb-2">Sign in to view analytics</p>
				<p class="text-gray-400 text-sm">Connect your account to see your clip performance data.</p>
			</div>

		{:else if overview}

			<!-- Summary cards -->
			<div class="grid grid-cols-2 gap-4 mb-8">
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Total Views</div>
					<div class="text-2xl font-bold text-white">{formatViews(totalViews)}</div>
					<div class="text-gray-600 text-xs">all platforms</div>
				</div>
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Total Earned</div>
					<div class="text-2xl font-bold text-purple-400">{formatSats(totalEarned)}</div>
					<div class="text-gray-600 text-xs">all time</div>
				</div>
			</div>

			<!-- Views over time chart -->
			<div class="bg-gray-900 rounded-xl p-5 mb-8">
				<h2 class="font-semibold text-white mb-4">Views Over Time (Last 30 Days)</h2>
				{#if overview.daily_views.length === 0}
					<div class="h-48 flex items-center justify-center text-gray-600 text-sm">
						No view data yet — keep clipping!
					</div>
				{:else}
					<ViewsChart data={overview.daily_views} />
				{/if}
			</div>

			<!-- Platform breakdown -->
			<div class="bg-gray-900 rounded-xl overflow-hidden mb-8">
				<div class="px-5 py-4 border-b border-gray-800">
					<h2 class="font-semibold text-white">Platform Breakdown</h2>
				</div>

				{#if overview.by_platform.length === 0}
					<div class="px-5 py-10 text-center text-gray-500 text-sm">
						No submissions yet across any platforms.
					</div>
				{:else}
					<div class="divide-y divide-gray-800/50">
						{#each overview.by_platform as platform}
							{@const pct = totalViews > 0 ? Math.round((platform.views / totalViews) * 100) : 0}
							<div class="px-5 py-4 flex items-center gap-4">
								<div class="w-28 flex-shrink-0">
									<span class="text-xs bg-gray-800 text-gray-300 px-2 py-1 rounded font-medium">
										{platformLabel(platform.platform)}
									</span>
								</div>
								<div class="flex-1">
									<div class="flex justify-between text-xs text-gray-500 mb-1">
										<span>{formatViews(platform.views)} views</span>
										<span>{pct}%</span>
									</div>
									<div class="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
										<div
											class="bg-purple-500 h-1.5 rounded-full"
											style="width: {pct}%"
										></div>
									</div>
								</div>
								<div class="text-right flex-shrink-0 w-24">
									<div class="text-purple-400 font-medium text-sm">{formatSats(platform.earned_sats)}</div>
									<div class="text-gray-600 text-xs">earned</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		{/if}
	</div>
</div>

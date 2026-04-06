<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { formatSats, formatViews, timeAgo, platformLabel } from '$lib/utils';
	import { isAuthenticated, initAuth } from '$lib/stores/auth';
	import { loginWithKeycast } from '$lib/auth';
	import type { DashboardData, Submission } from '$lib/types';

	let data: DashboardData | null = $state(null);
	let submissions: Submission[] = $state([]);
	let loading = $state(true);
	let error = $state('');

	const trustLevelColors: Record<number, { ring: string; label: string; text: string }> = {
		1: { ring: 'ring-amber-600', label: 'Level 1', text: 'text-amber-600' },
		2: { ring: 'ring-gray-400', label: 'Level 2', text: 'text-gray-400' },
		3: { ring: 'ring-yellow-500', label: 'Level 3', text: 'text-yellow-500' },
	};

	const statusStyles: Record<string, string> = {
		pending: 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30',
		active: 'bg-blue-500/20 text-blue-400 border border-blue-500/30',
		verified: 'bg-green-500/20 text-green-400 border border-green-500/30',
		rejected: 'bg-red-500/20 text-red-400 border border-red-500/30',
	};

	function getTrustBadge(level: number) {
		return trustLevelColors[level] ?? trustLevelColors[1];
	}

	function getStatusStyle(status: string): string {
		return statusStyles[status] ?? 'bg-gray-500/20 text-gray-400 border border-gray-500/30';
	}

	function truncateUrl(url: string, max = 40): string {
		try {
			const parsed = new URL(url);
			const short = parsed.hostname + parsed.pathname;
			return short.length > max ? short.slice(0, max) + '…' : short;
		} catch {
			return url.length > max ? url.slice(0, max) + '…' : url;
		}
	}

	function weeklyPercent(used: number, limit: number): number {
		if (limit === 0) return 0;
		return Math.min(100, Math.round((used / limit) * 100));
	}

	onMount(async () => {
		initAuth();
		if (!$isAuthenticated) {
			// Show empty dashboard UI without data
			data = {
				trust_level: 1,
				total_verified_views: 0,
				total_earned_sats: 0,
				balance_sats: 0,
				active_submissions: 0,
				weekly_views_used: 0,
				weekly_views_limit: 10_000,
			};
			loading = false;
			return;
		}
		try {
			const [dashData, subData] = await Promise.all([
				api.dashboard(),
				api.submissions.list(),
			]);
			data = dashData;
			submissions = subData;
		} catch {
			// API not available — show empty dashboard with defaults
			data = {
				trust_level: 1,
				total_verified_views: 0,
				total_earned_sats: 0,
				balance_sats: 0,
				active_submissions: 0,
				weekly_views_used: 0,
				weekly_views_limit: 10_000,
			};
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Dashboard — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white">
	<div class="max-w-5xl mx-auto px-4 py-10">

		{#if loading}
			<div class="flex items-center justify-center py-32">
				<div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
			</div>

		{:else if error}
			<div class="bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl p-6 text-center">
				{error}
			</div>

		{:else if data}
			{#if !$isAuthenticated}
				<div class="bg-purple-600/10 border border-purple-500/30 rounded-xl p-5 mb-8 flex items-center justify-between">
					<div>
						<p class="text-white font-medium">Sign in to track your clips and earnings</p>
						<p class="text-gray-400 text-sm mt-1">Connect your account to start clipping campaigns and earning sats.</p>
					</div>
					<button
						onclick={() => loginWithKeycast()}
						class="bg-purple-600 hover:bg-purple-500 transition-colors text-white font-semibold px-6 py-2.5 rounded-xl text-sm whitespace-nowrap"
					>
						Sign In
					</button>
				</div>
			{/if}

			<!-- Trust Level Badge -->
			<div class="flex items-center gap-4 mb-8">
				{#if data}
					{@const badge = getTrustBadge(data.trust_level)}
					<div class="flex items-center gap-3">
						<div class="w-14 h-14 rounded-full ring-4 {badge.ring} bg-gray-900 flex items-center justify-center">
							<span class="text-xl font-bold {badge.text}">{data.trust_level}</span>
						</div>
						<div>
							<div class="text-sm text-gray-500 uppercase tracking-wider">Trust</div>
							<div class="font-semibold {badge.text}">{badge.label}</div>
						</div>
					</div>
				{/if}
				<div class="ml-auto">
					<h1 class="text-2xl font-bold text-white text-right">Dashboard</h1>
				</div>
			</div>

			<!-- Stats Cards -->
			<div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Total Verified Views</div>
					<div class="text-2xl font-bold text-white">{formatViews(data.total_verified_views)}</div>
					<div class="text-gray-600 text-xs">all time</div>
				</div>
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Total Earned</div>
					<div class="text-2xl font-bold text-purple-400">{formatSats(data.total_earned_sats)}</div>
					<div class="text-gray-600 text-xs">lifetime</div>
				</div>
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Current Balance</div>
					<div class="text-2xl font-bold text-green-400">{formatSats(data.balance_sats)}</div>
					<div class="text-gray-600 text-xs">available</div>
				</div>
				<div class="bg-gray-900 rounded-xl p-5 flex flex-col gap-2">
					<div class="text-gray-500 text-xs uppercase tracking-wider">Active Submissions</div>
					<div class="text-2xl font-bold text-blue-400">{data.active_submissions}</div>
					<div class="text-gray-600 text-xs">in progress</div>
				</div>
			</div>

			<!-- Weekly Views Progress -->
			<div class="bg-gray-900 rounded-xl p-5 mb-8">
				<div class="flex justify-between items-center mb-3">
					<div class="text-sm font-medium text-gray-300">Weekly Views Progress</div>
					<div class="text-sm text-gray-500">
						{formatViews(data.weekly_views_used)} / {formatViews(data.weekly_views_limit)} views this week
					</div>
				</div>
				<div class="w-full bg-gray-800 rounded-full h-3 overflow-hidden">
					<div
						class="bg-purple-500 h-3 rounded-full transition-all duration-500"
						style="width: {weeklyPercent(data.weekly_views_used, data.weekly_views_limit)}%"
					></div>
				</div>
				<div class="text-right text-xs text-gray-600 mt-1">
					{weeklyPercent(data.weekly_views_used, data.weekly_views_limit)}%
				</div>
			</div>

			<!-- Active Submissions -->
			<div class="bg-gray-900 rounded-xl overflow-hidden">
				<div class="px-5 py-4 border-b border-gray-800">
					<h2 class="font-semibold text-white">Active Submissions</h2>
				</div>

				{#if submissions.length === 0}
					<div class="px-5 py-10 text-center text-gray-500">No submissions yet.</div>
				{:else}
					<div class="overflow-x-auto">
						<table class="w-full text-sm">
							<thead>
								<tr class="text-gray-500 text-xs uppercase tracking-wider border-b border-gray-800">
									<th class="text-left px-5 py-3">Platform / URL</th>
									<th class="text-left px-5 py-3">Status</th>
									<th class="text-right px-5 py-3">Views</th>
									<th class="text-right px-5 py-3">Earned</th>
								</tr>
							</thead>
							<tbody>
								{#each submissions as sub, i}
									<tr class="border-b border-gray-800/50 {i % 2 === 0 ? '' : 'bg-gray-800/20'} hover:bg-gray-800/40 transition-colors">
										<td class="px-5 py-4">
											<div class="flex items-center gap-2">
												<span class="text-gray-400 text-xs bg-gray-800 px-2 py-0.5 rounded font-medium">{platformLabel(sub.platform)}</span>
												<a
													href={sub.external_url}
													target="_blank"
													rel="noopener noreferrer"
													class="text-purple-400 hover:text-purple-300 underline underline-offset-2 truncate max-w-[200px]"
													title={sub.external_url}
												>
													{truncateUrl(sub.external_url)}
												</a>
											</div>
											<div class="text-gray-600 text-xs mt-1">{timeAgo(sub.submitted_at)}</div>
										</td>
										<td class="px-5 py-4">
											<span class="px-2.5 py-1 rounded-full text-xs font-medium {getStatusStyle(sub.status)}">
												{sub.status}
											</span>
										</td>
										<td class="px-5 py-4 text-right text-gray-300">{formatViews(sub.total_verified_views)}</td>
										<td class="px-5 py-4 text-right text-purple-400 font-medium">{formatSats(sub.total_paid_sats)}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		{/if}

	</div>
</div>

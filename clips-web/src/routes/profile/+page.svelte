<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated, authToken, logout } from '$lib/stores/auth';
	import { api } from '$lib/api';
	import { platformLabel, PLATFORMS, formatSats } from '$lib/utils';
	import type { DashboardData } from '$lib/types';

	let stats = $state<DashboardData | null>(null);
	let loadingStats = $state(true);
	let memberSince = $state<string | null>(null);

	// Derive a truncated npub from the auth token
	let displayIdentity = $derived.by(() => {
		const token = $authToken;
		if (!token) return null;
		if (token.startsWith('npub1')) {
			return token.slice(0, 16) + '...' + token.slice(-6);
		}
		// For MVP tokens that aren't proper npubs, still truncate
		if (token.length > 24) {
			return token.slice(0, 16) + '...' + token.slice(-6);
		}
		return token;
	});

	onMount(async () => {
		if (!$isAuthenticated) {
			goto('/');
			return;
		}

		try {
			stats = await api.dashboard();
			// Use today as a proxy for member since in MVP
			memberSince = new Date().toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
		} catch {
			stats = null;
		} finally {
			loadingStats = false;
		}
	});

	function handleSignOut() {
		logout();
		goto('/');
	}
</script>

<svelte:head>
	<title>Profile — DiVine Clips</title>
</svelte:head>

<div class="max-w-2xl mx-auto">
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-white mb-2">Profile</h1>
		<p class="text-gray-400">Your identity and connected accounts</p>
	</div>

	{#if !$isAuthenticated}
		<div class="bg-gray-900 border border-gray-800 rounded-xl p-8 text-center">
			<p class="text-gray-400 mb-4">You need to sign in to view your profile.</p>
			<a href="/" class="inline-block px-6 py-2 bg-purple-600 hover:bg-purple-500 text-white font-semibold rounded-lg transition-colors">
				Go Home
			</a>
		</div>
	{:else}
		<!-- Nostr Identity -->
		<div class="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6">
			<h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">Nostr Identity</h2>
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-full bg-purple-700 flex items-center justify-center text-white font-bold text-sm flex-shrink-0">
					N
				</div>
				<div>
					<p class="text-xs text-gray-500 mb-0.5">Connected as</p>
					<p class="text-white font-mono text-sm break-all">{displayIdentity ?? '—'}</p>
				</div>
			</div>
		</div>

		<!-- Social Accounts -->
		<div class="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6">
			<h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-1">Social Accounts</h2>
			<p class="text-gray-500 text-xs mb-5">Connect your social accounts to verify clips</p>
			<div class="space-y-4">
				{#each PLATFORMS as platform}
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<div class="w-8 h-8 rounded-lg bg-gray-800 flex items-center justify-center text-gray-400 text-xs font-bold">
								{platformLabel(platform).slice(0, 2).toUpperCase()}
							</div>
							<div>
								<p class="text-white text-sm font-medium">{platformLabel(platform)}</p>
								<p class="text-gray-500 text-xs">Not connected</p>
							</div>
						</div>
						<button
							type="button"
							disabled
							title="Coming soon"
							class="px-4 py-1.5 rounded-lg text-sm font-medium bg-gray-800 text-gray-500 border border-gray-700 cursor-not-allowed opacity-60"
						>
							Connect
						</button>
					</div>
				{/each}
			</div>
			<p class="text-gray-600 text-xs mt-5">Social account verification coming soon.</p>
		</div>

		<!-- Quick Stats -->
		<div class="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6">
			<h2 class="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">Stats</h2>
			{#if loadingStats}
				<div class="flex items-center gap-2 text-gray-500 text-sm">
					<svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
					</svg>
					Loading stats...
				</div>
			{:else if stats}
				<div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Total Views</p>
						<p class="text-white font-semibold text-lg">{stats.total_verified_views.toLocaleString()}</p>
					</div>
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Total Earned</p>
						<p class="text-yellow-400 font-semibold text-lg">{formatSats(stats.total_earned_sats)}</p>
					</div>
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Member Since</p>
						<p class="text-white font-semibold text-base">{memberSince ?? '—'}</p>
					</div>
				</div>
			{:else}
				<div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Total Views</p>
						<p class="text-white font-semibold text-lg">—</p>
					</div>
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Total Earned</p>
						<p class="text-yellow-400 font-semibold text-lg">—</p>
					</div>
					<div class="bg-gray-800 rounded-lg p-4">
						<p class="text-gray-400 text-xs mb-1">Member Since</p>
						<p class="text-white font-semibold text-base">{memberSince ?? '—'}</p>
					</div>
				</div>
			{/if}
		</div>

		<!-- Sign Out -->
		<div class="flex justify-end">
			<button
				type="button"
				onclick={handleSignOut}
				class="px-6 py-2.5 bg-gray-800 hover:bg-red-900/40 border border-gray-700 hover:border-red-700 text-gray-300 hover:text-red-300 font-medium rounded-lg transition-colors text-sm"
			>
				Sign Out
			</button>
		</div>
	{/if}
</div>

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated } from '$lib/stores/auth';
	import { api } from '$lib/api';
	import { PLATFORMS, platformLabel } from '$lib/utils';
	import type { CreateCampaignRequest } from '$lib/types';

	let title = $state('');
	let budgetSats = $state('');
	let cpmSats = $state('');
	let selectedPlatforms = $state<string[]>([]);
	let contentRefsRaw = $state('');
	let guidelines = $state('');
	let expiresAt = $state('');

	let submitting = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		// Redirect if not authenticated
		if (!$isAuthenticated) {
			goto('/');
		}
	});

	function togglePlatform(platform: string) {
		if (selectedPlatforms.includes(platform)) {
			selectedPlatforms = selectedPlatforms.filter(p => p !== platform);
		} else {
			selectedPlatforms = [...selectedPlatforms, platform];
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;

		if (!title.trim()) { error = 'Title is required.'; return; }
		if (!budgetSats || isNaN(Number(budgetSats)) || Number(budgetSats) <= 0) { error = 'Enter a valid budget in sats.'; return; }
		if (!cpmSats || isNaN(Number(cpmSats)) || Number(cpmSats) <= 0) { error = 'Enter a valid CPM rate in sats.'; return; }
		if (selectedPlatforms.length === 0) { error = 'Select at least one target platform.'; return; }

		const contentRefs = contentRefsRaw
			.split('\n')
			.map(s => s.trim())
			.filter(Boolean);

		if (contentRefs.length === 0) { error = 'Enter at least one content reference URL or ID.'; return; }

		const payload: CreateCampaignRequest = {
			title: title.trim(),
			budget_sats: Number(budgetSats),
			cpm_sats: Number(cpmSats),
			target_platforms: selectedPlatforms,
			content_refs: contentRefs,
		};

		if (guidelines.trim()) payload.guidelines = guidelines.trim();
		if (expiresAt) payload.expires_at = new Date(expiresAt).toISOString();

		submitting = true;
		try {
			const campaign = await api.campaigns.create(payload);
			goto(`/campaigns/${campaign.id}`);
		} catch (err: any) {
			error = err.message || 'Failed to create campaign. Please try again.';
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>Create Campaign — DiVine Clips</title>
</svelte:head>

<div class="max-w-2xl mx-auto">
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-white mb-2">Create Campaign</h1>
		<p class="text-gray-400">Set up a new campaign and start earning views with Bitcoin payouts.</p>
	</div>

	{#if !$isAuthenticated}
		<div class="bg-gray-900 border border-gray-800 rounded-xl p-8 text-center">
			<p class="text-gray-400 mb-4">You need to sign in to create a campaign.</p>
			<a href="/" class="inline-block px-6 py-2 bg-purple-600 hover:bg-purple-500 text-white font-semibold rounded-lg transition-colors">
				Go Home
			</a>
		</div>
	{:else}
		<form onsubmit={handleSubmit} class="space-y-6">
			<!-- Error Banner -->
			{#if error}
				<div class="bg-red-900/50 border border-red-700 text-red-300 rounded-lg px-4 py-3 text-sm">
					{error}
				</div>
			{/if}

			<!-- Title -->
			<div>
				<label for="title" class="block text-sm font-medium text-gray-300 mb-1">
					Campaign Title <span class="text-red-400">*</span>
				</label>
				<input
					id="title"
					type="text"
					bind:value={title}
					placeholder="e.g. Summer Vibes Promo"
					maxlength="120"
					class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition"
				/>
			</div>

			<!-- Budget + CPM side by side -->
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<div>
					<label for="budget" class="block text-sm font-medium text-gray-300 mb-1">
						Total budget in sats <span class="text-red-400">*</span>
					</label>
					<input
						id="budget"
						type="number"
						bind:value={budgetSats}
						placeholder="e.g. 100000"
						min="1"
						class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition"
					/>
				</div>
				<div>
					<label for="cpm" class="block text-sm font-medium text-gray-300 mb-1">
						Payout per 1,000 views (sats) <span class="text-red-400">*</span>
					</label>
					<input
						id="cpm"
						type="number"
						bind:value={cpmSats}
						placeholder="e.g. 100"
						min="1"
						class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition"
					/>
				</div>
			</div>

			<!-- Target Platforms -->
			<div>
				<fieldset>
					<legend class="block text-sm font-medium text-gray-300 mb-2">
						Target Platforms <span class="text-red-400">*</span>
					</legend>
					<div class="flex flex-wrap gap-3">
						{#each PLATFORMS as platform}
							<button
								type="button"
								onclick={() => togglePlatform(platform)}
								class="px-4 py-2 rounded-lg text-sm font-medium border transition-colors {selectedPlatforms.includes(platform)
									? 'bg-purple-600 border-purple-500 text-white'
									: 'bg-gray-900 border-gray-700 text-gray-400 hover:text-white hover:border-gray-500'}"
							>
								{platformLabel(platform)}
							</button>
						{/each}
					</div>
				</fieldset>
			</div>

			<!-- Content References -->
			<div>
				<label for="content-refs" class="block text-sm font-medium text-gray-300 mb-1">
					Content References <span class="text-red-400">*</span>
				</label>
				<p class="text-gray-500 text-xs mb-2">DiVine video URLs or IDs, one per line</p>
				<textarea
					id="content-refs"
					bind:value={contentRefsRaw}
					rows={4}
					placeholder="https://divine.example/v/abc123&#10;https://divine.example/v/def456"
					class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition resize-y font-mono text-sm"
				></textarea>
			</div>

			<!-- Guidelines (optional) -->
			<div>
				<label for="guidelines" class="block text-sm font-medium text-gray-300 mb-1">
					Guidelines <span class="text-gray-500 font-normal">(optional)</span>
				</label>
				<p class="text-gray-500 text-xs mb-2">Instructions for clippers on how to share your content</p>
				<textarea
					id="guidelines"
					bind:value={guidelines}
					rows={3}
					placeholder="e.g. Use the hashtag #MyBrand, post between 6–9pm for best reach..."
					class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition resize-y text-sm"
				></textarea>
			</div>

			<!-- Expiry Date (optional) -->
			<div>
				<label for="expires-at" class="block text-sm font-medium text-gray-300 mb-1">
					Expiry Date <span class="text-gray-500 font-normal">(optional)</span>
				</label>
				<input
					id="expires-at"
					type="date"
					bind:value={expiresAt}
					class="bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition [color-scheme:dark]"
				/>
			</div>

			<!-- Submit -->
			<div class="pt-2">
				<button
					type="submit"
					disabled={submitting}
					class="w-full sm:w-auto px-10 py-3 bg-purple-600 hover:bg-purple-500 disabled:opacity-60 disabled:cursor-not-allowed text-white font-semibold rounded-lg transition-colors text-base shadow-lg shadow-purple-900/40"
				>
					{#if submitting}
						<span class="flex items-center justify-center gap-2">
							<svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
							</svg>
							Creating...
						</span>
					{:else}
						Create Campaign
					{/if}
				</button>
			</div>
		</form>
	{/if}
</div>

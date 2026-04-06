<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated, authToken } from '$lib/stores/auth';
	import { api } from '$lib/api';
	import { PLATFORMS, platformLabel, CURRENCIES, currencyToSats, formatMoney, formatCpm, type Currency } from '$lib/utils';
	import type { CreateCampaignRequest } from '$lib/types';

	// Form fields
	let title = $state('');
	let budgetAmount = $state('');
	let budgetCurrency = $state<Currency>('USD');
	let cpmAmount = $state('');
	let cpmCurrency = $state<Currency>('USD');
	let selectedPlatforms = $state<string[]>([]);
	let guidelines = $state('');
	let expiresAt = $state('');

	// Video selection
	interface DiVineVideo {
		id: string;
		title: string;
		thumbnail?: string;
		created_at: string;
		d_tag: string;
	}
	let myVideos = $state<DiVineVideo[]>([]);
	let selectedVideoIds = $state<Set<string>>(new Set());
	let loadingVideos = $state(true);
	let videoError = $state<string | null>(null);

	// Hashtag promotion (alternative to specific videos)
	let promoteAll = $state(false);
	let hashtags = $state('');

	let submitting = $state(false);
	let error = $state<string | null>(null);

	// Computed sats equivalents for display
	let budgetSatsPreview = $derived(
		budgetAmount && !isNaN(Number(budgetAmount))
			? currencyToSats(Number(budgetAmount), budgetCurrency)
			: 0
	);
	let cpmSatsPreview = $derived(
		cpmAmount && !isNaN(Number(cpmAmount))
			? currencyToSats(Number(cpmAmount), cpmCurrency)
			: 0
	);

	onMount(async () => {
		if (!$isAuthenticated) {
			goto('/');
			return;
		}
		await loadMyVideos();
	});

	async function loadMyVideos() {
		loadingVideos = true;
		videoError = null;
		try {
			// Fetch creator's DiVine videos from the relay
			// Uses the funnelcake API — the auth token identifies the creator
			// Funnelcake API: filter by author pubkey
			const res = await fetch(`https://relay.divine.video/api/videos?sort=recent&limit=50&pubkey=${$authToken}`);
			if (res.ok) {
				const data = await res.json();
				const videos = Array.isArray(data) ? data : (data.videos || []);
				myVideos = videos.map((v: any) => ({
					id: v.id || v.d_tag,
					title: v.title || v.content || 'Untitled',
					thumbnail: v.thumbnail,
					created_at: v.created_at || v.published_at || '',
					d_tag: v.d_tag || v.id,
				}));
			} else {
				// Fallback: no videos found or API not available
				myVideos = [];
			}
		} catch {
			myVideos = [];
			videoError = 'Could not load your DiVine videos. You can still create a campaign with hashtags.';
		} finally {
			loadingVideos = false;
		}
	}

	function toggleVideo(id: string) {
		const next = new Set(selectedVideoIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedVideoIds = next;
	}

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
		if (!budgetAmount || isNaN(Number(budgetAmount)) || Number(budgetAmount) <= 0) { error = 'Enter a valid budget amount.'; return; }
		if (!cpmAmount || isNaN(Number(cpmAmount)) || Number(cpmAmount) <= 0) { error = 'Enter a valid payout rate.'; return; }
		if (selectedPlatforms.length === 0) { error = 'Select at least one target platform.'; return; }
		if (!promoteAll && selectedVideoIds.size === 0) { error = 'Select at least one video to promote, or choose "Promote all my content".'; return; }

		const contentRefs = promoteAll
			? (hashtags.trim() ? hashtags.split(/[,\s]+/).map(h => h.startsWith('#') ? h : `#${h}`).filter(Boolean) : ['*'])
			: Array.from(selectedVideoIds);

		const payload: CreateCampaignRequest = {
			title: title.trim(),
			budget_sats: budgetSatsPreview,
			cpm_sats: cpmSatsPreview,
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
		<h1 class="text-3xl font-bold text-white mb-2">Promote Your Content</h1>
		<p class="text-gray-400">Choose videos to promote and set your budget. Clippers will share your content across social media.</p>
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

			<!-- Select Videos to Promote -->
			<div>
				<label class="block text-sm font-medium text-gray-300 mb-2">
					What do you want to promote? <span class="text-red-400">*</span>
				</label>

				<!-- Toggle: specific videos vs all content -->
				<div class="flex gap-3 mb-4">
					<button
						type="button"
						onclick={() => { promoteAll = false; }}
						class="px-4 py-2 rounded-lg text-sm font-medium border transition-colors {!promoteAll
							? 'bg-purple-600 border-purple-500 text-white'
							: 'bg-gray-900 border-gray-700 text-gray-400 hover:text-white hover:border-gray-500'}"
					>
						Specific videos
					</button>
					<button
						type="button"
						onclick={() => { promoteAll = true; }}
						class="px-4 py-2 rounded-lg text-sm font-medium border transition-colors {promoteAll
							? 'bg-purple-600 border-purple-500 text-white'
							: 'bg-gray-900 border-gray-700 text-gray-400 hover:text-white hover:border-gray-500'}"
					>
						All my content
					</button>
				</div>

				{#if promoteAll}
					<div class="bg-gray-900 border border-gray-800 rounded-lg p-4">
						<p class="text-gray-300 text-sm mb-3">Clippers can share any of your DiVine videos. Optionally add hashtags to focus on specific themes:</p>
						<input
							type="text"
							bind:value={hashtags}
							placeholder="e.g. music, comedy, art (optional)"
							class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition text-sm"
						/>
					</div>
				{:else}
					<!-- Video grid -->
					{#if loadingVideos}
						<div class="bg-gray-900 border border-gray-800 rounded-lg p-8 text-center">
							<div class="animate-spin w-6 h-6 border-2 border-purple-600 border-t-transparent rounded-full mx-auto mb-2"></div>
							<p class="text-gray-500 text-sm">Loading your DiVine videos...</p>
						</div>
					{:else if videoError}
						<div class="bg-yellow-900/30 border border-yellow-700/50 text-yellow-300 rounded-lg px-4 py-3 text-sm">
							{videoError}
						</div>
					{:else if myVideos.length === 0}
						<div class="bg-gray-900 border border-gray-800 rounded-lg p-6 text-center">
							<p class="text-gray-400 mb-2">No DiVine videos found for your account.</p>
							<p class="text-gray-500 text-sm">Post some loops on <a href="https://divine.video" class="text-purple-400 hover:underline">divine.video</a> first, or switch to "All my content" mode.</p>
						</div>
					{:else}
						<div class="grid grid-cols-2 sm:grid-cols-3 gap-3 max-h-80 overflow-y-auto pr-1">
							{#each myVideos as video (video.id)}
								<button
									type="button"
									onclick={() => toggleVideo(video.id)}
									class="relative rounded-lg border-2 overflow-hidden transition-all {selectedVideoIds.has(video.id)
										? 'border-purple-500 ring-2 ring-purple-500/30'
										: 'border-gray-700 hover:border-gray-500'}"
								>
									{#if video.thumbnail}
										<img src={video.thumbnail} alt={video.title} class="w-full aspect-square object-cover" />
									{:else}
										<div class="w-full aspect-square bg-gray-800 flex items-center justify-center">
											<span class="text-gray-600 text-2xl">&#x25B6;</span>
										</div>
									{/if}
									<div class="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/80 to-transparent p-2">
										<p class="text-white text-xs truncate">{video.title}</p>
									</div>
									{#if selectedVideoIds.has(video.id)}
										<div class="absolute top-2 right-2 w-6 h-6 bg-purple-600 rounded-full flex items-center justify-center">
											<span class="text-white text-xs font-bold">&#x2713;</span>
										</div>
									{/if}
								</button>
							{/each}
						</div>
						<p class="text-gray-500 text-xs mt-2">{selectedVideoIds.size} video{selectedVideoIds.size !== 1 ? 's' : ''} selected</p>
					{/if}
				{/if}
			</div>

			<!-- Budget + CPM -->
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<div>
					<label for="budget" class="block text-sm font-medium text-gray-300 mb-1">
						Total budget <span class="text-red-400">*</span>
					</label>
					<div class="flex gap-2">
						<input
							id="budget"
							type="number"
							bind:value={budgetAmount}
							placeholder={budgetCurrency === 'USD' ? 'e.g. 50' : 'e.g. 100000'}
							min="0"
							step={budgetCurrency === 'USD' ? '0.01' : '1'}
							class="flex-1 bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition"
						/>
						<select
							bind:value={budgetCurrency}
							class="bg-gray-900 border border-gray-700 rounded-lg px-3 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-purple-600 transition"
						>
							{#each CURRENCIES as c}
								<option value={c}>{c}</option>
							{/each}
						</select>
					</div>
					{#if budgetSatsPreview > 0}
						<p class="text-gray-500 text-xs mt-1">= {budgetSatsPreview.toLocaleString()} sats ({formatMoney(budgetSatsPreview)})</p>
					{/if}
				</div>
				<div>
					<label for="cpm" class="block text-sm font-medium text-gray-300 mb-1">
						Payout per 1,000 views <span class="text-red-400">*</span>
					</label>
					<div class="flex gap-2">
						<input
							id="cpm"
							type="number"
							bind:value={cpmAmount}
							placeholder={cpmCurrency === 'USD' ? 'e.g. 3' : 'e.g. 3000'}
							min="0"
							step={cpmCurrency === 'USD' ? '0.01' : '1'}
							class="flex-1 bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition"
						/>
						<select
							bind:value={cpmCurrency}
							class="bg-gray-900 border border-gray-700 rounded-lg px-3 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-purple-600 transition"
						>
							{#each CURRENCIES as c}
								<option value={c}>{c}</option>
							{/each}
						</select>
					</div>
					{#if cpmSatsPreview > 0}
						<p class="text-gray-500 text-xs mt-1">= {formatCpm(cpmSatsPreview)}</p>
					{/if}
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

			<!-- Guidelines (optional) -->
			<div>
				<label for="guidelines" class="block text-sm font-medium text-gray-300 mb-1">
					Guidelines <span class="text-gray-500 font-normal">(optional)</span>
				</label>
				<textarea
					id="guidelines"
					bind:value={guidelines}
					rows={3}
					placeholder="Any tips for clippers on how to share your content..."
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
					class="bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-purple-600 focus:border-transparent transition [color-scheme:dark]"
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
						Launch Campaign
					{/if}
				</button>
			</div>
		</form>
	{/if}
</div>

<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { isAuthenticated, authToken } from '$lib/stores/auth';
	import { initAuth } from '$lib/stores/auth';
	import { loginWithKeycast, logout, restoreSession } from '$lib/auth';
	import { registerPushNotifications } from '$lib/push';

	let { children } = $props();

	let mobileMenuOpen = $state(false);
	let notifToast = $state<{ message: string; ok: boolean } | null>(null);

	onMount(async () => {
		try {
			// Try restoring session from @divinevideo/login tokens first
			const restored = await restoreSession();
			if (!restored) {
				// Fall back to localStorage token check
				initAuth();
			}
		} catch {
			// Auth restore failed — fall back to localStorage
			initAuth();
		}
	});

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function handleSignIn() {
		loginWithKeycast();
	}

	function handleSignOut() {
		logout();
	}

	function isActive(path: string): boolean {
		return $page.url.pathname === path;
	}

	async function handleNotificationBell() {
		const token = $authToken;
		if (!token) return;

		const ok = await registerPushNotifications(token);
		notifToast = { message: ok ? 'Notifications enabled!' : 'Could not enable notifications.', ok };
		setTimeout(() => { notifToast = null; }, 3000);
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white">
	<!-- Top navigation bar -->
	<nav class="bg-gray-900 border-b border-gray-800 sticky top-0 z-50">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between h-16">
				<!-- Logo / Brand -->
				<div class="flex items-center">
					<a href="/" class="text-xl font-bold text-white hover:text-purple-400 transition-colors">
						DiVine Clips
					</a>
				</div>

				<!-- Desktop nav links -->
				<div class="hidden md:flex items-center space-x-1">
					<a
						href="/"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Home
					</a>
					<a
						href="/campaigns"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/campaigns') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Campaigns
					</a>
					<a
						href="/guide"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/guide') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						How It Works
					</a>
					<a
						href="/dashboard"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/dashboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Dashboard
					</a>
					<a
						href="/leaderboard"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/leaderboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Leaderboard
					</a>
					<a
						href="/wallet"
						class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/wallet') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Wallet
					</a>
				</div>

				<!-- Right side: Notifications bell + Sign in / Sign out -->
				<div class="hidden md:flex items-center gap-2">
					{#if $isAuthenticated}
						<button
							onclick={handleNotificationBell}
							title="Enable push notifications"
							class="p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
							aria-label="Enable notifications"
						>
							<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
							</svg>
						</button>
						<button
							onclick={handleSignOut}
							class="px-4 py-2 rounded-md text-sm font-medium bg-gray-800 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
						>
							Sign Out
						</button>
					{:else}
						<button
							onclick={handleSignIn}
							class="px-4 py-2 rounded-md text-sm font-medium bg-purple-600 text-white hover:bg-purple-500 transition-colors"
						>
							Sign In
						</button>
					{/if}
				</div>

				<!-- Mobile hamburger button -->
				<div class="md:hidden flex items-center">
					<button
						onclick={toggleMobileMenu}
						class="p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
						aria-label="Toggle mobile menu"
					>
						{#if mobileMenuOpen}
							<!-- X icon -->
							<svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
							</svg>
						{:else}
							<!-- Hamburger icon -->
							<svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
							</svg>
						{/if}
					</button>
				</div>
			</div>
		</div>

		<!-- Mobile menu -->
		{#if mobileMenuOpen}
			<div class="md:hidden bg-gray-900 border-t border-gray-800 px-4 py-2 space-y-1">
				<a
					href="/"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					Home
				</a>
				<a
					href="/campaigns"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/campaigns') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					Campaigns
				</a>
				<a
					href="/guide"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/guide') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					How It Works
				</a>
				<a
					href="/dashboard"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/dashboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					Dashboard
				</a>
				<a
					href="/leaderboard"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/leaderboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					Leaderboard
				</a>
				<a
					href="/wallet"
					onclick={() => { mobileMenuOpen = false; }}
					class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/wallet') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
				>
					Wallet
				</a>
				<div class="pt-2 pb-1 border-t border-gray-800">
					{#if $isAuthenticated}
						<button
							onclick={() => { handleSignOut(); mobileMenuOpen = false; }}
							class="w-full text-left px-3 py-2 rounded-md text-sm font-medium bg-gray-800 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
						>
							Sign Out
						</button>
					{:else}
						<button
							onclick={() => { handleSignIn(); mobileMenuOpen = false; }}
							class="w-full text-left px-3 py-2 rounded-md text-sm font-medium bg-purple-600 text-white hover:bg-purple-500 transition-colors"
						>
							Sign In
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</nav>

	<!-- Page content -->
	<main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
		{@render children()}
	</main>
</div>

<!-- Push notification toast -->
{#if notifToast}
	<div
		class="fixed bottom-4 right-4 z-50 px-4 py-3 rounded-lg shadow-lg text-sm font-medium transition-all
			{notifToast.ok ? 'bg-green-700 text-green-100' : 'bg-red-700 text-red-100'}"
	>
		{notifToast.message}
	</div>
{/if}

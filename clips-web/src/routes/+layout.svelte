<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { isAuthenticated } from '$lib/stores/auth';
	import { initAuth } from '$lib/stores/auth';
	import { loginWithKeycast, logout, restoreSession } from '$lib/auth';

	let { children } = $props();

	let mobileMenuOpen = $state(false);

	onMount(async () => {
		// Try restoring session from @divinevideo/login tokens first
		const restored = await restoreSession();
		if (!restored) {
			// Fall back to localStorage token check
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
					{#if $isAuthenticated}
						<a
							href="/dashboard"
							class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/dashboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
						>
							Dashboard
						</a>
						<a
							href="/wallet"
							class="px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/wallet') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
						>
							Wallet
						</a>
					{/if}
				</div>

				<!-- Right side: Sign in / Sign out -->
				<div class="hidden md:flex items-center">
					{#if $isAuthenticated}
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
				{#if $isAuthenticated}
					<a
						href="/dashboard"
						onclick={() => { mobileMenuOpen = false; }}
						class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/dashboard') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Dashboard
					</a>
					<a
						href="/wallet"
						onclick={() => { mobileMenuOpen = false; }}
						class="block px-3 py-2 rounded-md text-sm font-medium transition-colors {isActive('/wallet') ? 'bg-purple-600 text-white' : 'text-gray-300 hover:text-white hover:bg-gray-800'}"
					>
						Wallet
					</a>
				{/if}
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

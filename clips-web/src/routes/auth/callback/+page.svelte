<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { handleAuthCallback } from '$lib/auth';

	let status = $state<'loading' | 'success' | 'error'>('loading');
	let errorMessage = $state('');

	onMount(async () => {
		const success = await handleAuthCallback(window.location.href);

		if (success) {
			status = 'success';
			// Redirect to dashboard after successful login
			setTimeout(() => goto('/dashboard'), 1000);
		} else {
			status = 'error';
			errorMessage = 'Login failed. Please try again.';
		}
	});
</script>

<svelte:head>
	<title>Signing in... — DiVine Clips</title>
</svelte:head>

<div class="max-w-md mx-auto mt-20 text-center">
	{#if status === 'loading'}
		<div class="animate-spin w-10 h-10 border-3 border-purple-600 border-t-transparent rounded-full mx-auto mb-4"></div>
		<p class="text-gray-300 text-lg">Signing you in...</p>
	{:else if status === 'success'}
		<div class="text-4xl mb-4">&#x2713;</div>
		<p class="text-green-400 text-lg font-medium">Welcome to DiVine Clips!</p>
		<p class="text-gray-500 text-sm mt-2">Redirecting to your dashboard...</p>
	{:else}
		<div class="text-4xl mb-4">&#x2717;</div>
		<p class="text-red-400 text-lg font-medium">{errorMessage}</p>
		<a href="/" class="mt-4 inline-block text-purple-400 hover:underline">Back to home</a>
	{/if}
</div>

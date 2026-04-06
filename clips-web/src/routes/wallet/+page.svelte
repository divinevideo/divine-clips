<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { formatSats, timeAgo } from '$lib/utils';
	import { isAuthenticated, initAuth } from '$lib/stores/auth';
	import { loginWithKeycast } from '$lib/auth';
	import type { TransactionRecord } from '$lib/types';

	let balance = $state(0);
	let transactions: TransactionRecord[] = $state([]);
	let loading = $state(true);
	let error = $state('');

	// Withdraw form state
	let invoice = $state('');
	let withdrawAmount = $state(0);
	let withdrawStatus = $state('');
	let withdrawError = $state('');
	let withdrawing = $state(false);

	// Rough USD estimate: 1 sat ≈ $0.0006
	const SAT_TO_USD = 0.0006;

	let usdEstimate = $derived((balance * SAT_TO_USD).toFixed(2));
	let canWithdraw = $derived(invoice.trim().length > 0 && withdrawAmount > 0 && withdrawAmount <= balance && !withdrawing);

	function setMax() {
		withdrawAmount = balance;
	}

	async function handleWithdraw() {
		if (!canWithdraw) return;
		withdrawStatus = '';
		withdrawError = '';
		withdrawing = true;
		try {
			const result = await api.wallet.withdraw(invoice.trim(), withdrawAmount);
			withdrawStatus = `Withdrawal of ${formatSats(result.amount_sats)} submitted successfully.`;
			balance -= result.amount_sats;
			invoice = '';
			withdrawAmount = 0;
			// Refresh history
			const histData = await api.wallet.history();
			transactions = histData.transactions;
		} catch (e: unknown) {
			withdrawError = e instanceof Error ? e.message : 'Withdrawal failed';
		} finally {
			withdrawing = false;
		}
	}

	onMount(async () => {
		initAuth();
		if (!$isAuthenticated) {
			loading = false;
			return;
		}
		try {
			const [balData, histData] = await Promise.all([
				api.wallet.balance(),
				api.wallet.history(),
			]);
			balance = balData.balance_sats;
			transactions = histData.transactions;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Failed to load wallet';
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Wallet — DiVine Clips</title>
</svelte:head>

<div class="min-h-screen bg-gray-950 text-white">
	<div class="max-w-2xl mx-auto px-4 py-10">

		{#if loading}
			<div class="flex items-center justify-center py-32">
				<div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
			</div>

		{:else if error}
			<div class="bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl p-6 text-center">
				{error}
			</div>

		{:else}
			{#if !$isAuthenticated}
				<div class="bg-purple-600/10 border border-purple-500/30 rounded-xl p-5 mb-8 flex items-center justify-between">
					<div>
						<p class="text-white font-medium">Sign in to access your wallet</p>
						<p class="text-gray-400 text-sm mt-1">Connect your account to view your balance, withdraw earnings, and see transaction history.</p>
					</div>
					<button
						onclick={() => loginWithKeycast()}
						class="bg-purple-600 hover:bg-purple-500 transition-colors text-white font-semibold px-6 py-2.5 rounded-xl text-sm whitespace-nowrap"
					>
						Sign In
					</button>
				</div>
			{/if}

			<!-- Balance Display -->
			<div class="text-center mb-10">
				<h1 class="text-gray-500 text-sm uppercase tracking-widest mb-3">Available Balance</h1>
				<div class="text-6xl font-bold text-white mb-2">{balance.toLocaleString()}</div>
				<div class="text-2xl text-gray-400 mb-1">sats</div>
				<div class="text-gray-500 text-sm">≈ ${usdEstimate} USD</div>
			</div>

			<!-- Withdraw Section -->
			<div class="bg-gray-900 rounded-xl p-6 mb-8">
				<h2 class="text-lg font-semibold text-white mb-5">Withdraw via Lightning</h2>

				<div class="flex flex-col gap-4">
					<!-- Lightning invoice input -->
					<div>
						<label for="invoice" class="block text-sm text-gray-400 mb-1.5">Lightning Invoice</label>
						<textarea
							id="invoice"
							bind:value={invoice}
							placeholder="lnbc..."
							rows="3"
							class="w-full bg-gray-800 border border-gray-700 focus:border-purple-500 focus:ring-1 focus:ring-purple-500 rounded-lg px-4 py-3 text-white text-sm placeholder-gray-600 resize-none outline-none transition-colors"
						></textarea>
					</div>

					<!-- Amount input -->
					<div>
						<label for="amount" class="block text-sm text-gray-400 mb-1.5">Amount (sats)</label>
						<div class="flex gap-2">
							<input
								id="amount"
								type="number"
								bind:value={withdrawAmount}
								min="1"
								max={balance}
								placeholder="0"
								class="flex-1 bg-gray-800 border border-gray-700 focus:border-purple-500 focus:ring-1 focus:ring-purple-500 rounded-lg px-4 py-3 text-white text-sm placeholder-gray-600 outline-none transition-colors"
							/>
							<button
								onclick={setMax}
								class="bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-300 text-sm font-medium px-4 py-3 rounded-lg transition-colors"
							>
								Max
							</button>
						</div>
						{#if withdrawAmount > balance}
							<div class="text-red-400 text-xs mt-1">Amount exceeds available balance.</div>
						{/if}
					</div>

					<!-- Withdraw button -->
					<button
						onclick={handleWithdraw}
						disabled={!canWithdraw}
						class="w-full bg-purple-600 hover:bg-purple-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed transition-colors text-white font-semibold py-3 rounded-xl text-sm"
					>
						{#if withdrawing}
							<span class="flex items-center justify-center gap-2">
								<span class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin inline-block"></span>
								Processing…
							</span>
						{:else}
							Withdraw
						{/if}
					</button>

					{#if withdrawStatus}
						<div class="bg-green-500/10 border border-green-500/30 text-green-400 rounded-lg px-4 py-3 text-sm">
							{withdrawStatus}
						</div>
					{/if}
					{#if withdrawError}
						<div class="bg-red-500/10 border border-red-500/30 text-red-400 rounded-lg px-4 py-3 text-sm">
							{withdrawError}
						</div>
					{/if}
				</div>
			</div>

			<!-- Transaction History -->
			<div class="bg-gray-900 rounded-xl overflow-hidden">
				<div class="px-5 py-4 border-b border-gray-800">
					<h2 class="font-semibold text-white">Transaction History</h2>
				</div>

				{#if transactions.length === 0}
					<div class="px-5 py-10 text-center text-gray-500 text-sm">No transactions yet.</div>
				{:else}
					<ul>
						{#each transactions as tx, i}
							<li class="flex items-center justify-between px-5 py-4 border-b border-gray-800/50 {i % 2 === 0 ? '' : 'bg-gray-800/20'}">
								<div class="flex flex-col gap-0.5">
									<span class="text-sm font-medium text-gray-300 capitalize">
										{tx.transaction_type === 'payout' ? 'Payout' : tx.transaction_type === 'withdrawal' ? 'Withdrawal' : tx.transaction_type}
									</span>
									<span class="text-xs text-gray-600">{timeAgo(tx.created_at)}</span>
								</div>
								<div class="text-sm font-semibold {tx.transaction_type === 'payout' ? 'text-green-400' : 'text-red-400'}">
									{tx.transaction_type === 'payout' ? '+' : '-'}{formatSats(Math.abs(tx.amount_sats))}
								</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{/if}

	</div>
</div>

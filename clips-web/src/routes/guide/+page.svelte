<script lang="ts">
	import { isAuthenticated } from '$lib/stores/auth';
	import { loginWithKeycast } from '$lib/auth';
</script>

<svelte:head>
	<title>How to Clip — DiVine Clips</title>
</svelte:head>

<div class="max-w-3xl mx-auto">
	<!-- Header -->
	<div class="mb-12 text-center">
		<h1 class="text-4xl sm:text-5xl font-extrabold text-white tracking-tight mb-4">
			How to Become a Clipper
		</h1>
		<p class="text-lg text-gray-400 max-w-2xl mx-auto">
			Clippers help grow DiVine by sharing its 6-second loops across social media. Here's everything you need to know to get started.
		</p>
	</div>

	<!-- What is DiVine -->
	<section class="mb-12 bg-gray-900 rounded-xl border border-gray-800 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">What is DiVine?</h2>
		<p class="text-gray-300 leading-relaxed mb-4">
			DiVine is a human-only 6-second looping video platform. Think Vine, but verified — every video on DiVine is proven to be made by a real person, not AI. It's built by a nonprofit collective and backed by Jack Dorsey.
		</p>
		<p class="text-gray-300 leading-relaxed">
			The 6-second loop format is unique. It's short enough to grab attention on any platform, and distinctive enough that people who see it want to know where it came from. That's where you come in.
		</p>
	</section>

	<!-- What is a Clipper -->
	<section class="mb-12 bg-gray-900 rounded-xl border border-gray-800 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">What's a Clipper?</h2>
		<p class="text-gray-300 leading-relaxed mb-4">
			A clipper is someone who takes DiVine videos and shares them on other platforms — TikTok, Instagram Reels, YouTube Shorts, and X. You're the bridge between DiVine and the rest of social media.
		</p>
		<p class="text-gray-300 leading-relaxed">
			You can share them as-is, compile multiple loops together, use them in reaction videos, or remix them with your own commentary. The 6-second format works great in all of these. No watermarks, no branding required — the format speaks for itself.
		</p>
	</section>

	<!-- Step by Step -->
	<section class="mb-12">
		<h2 class="text-2xl font-bold text-white mb-8 text-center">Getting Started — Step by Step</h2>

		<div class="space-y-6">
			<!-- Step 1 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					1
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Sign up</h3>
					<p class="text-gray-300 leading-relaxed mb-3">
						Create your account on DiVine Clips. You'll get a DiVine identity that tracks your reputation and earnings across everything you do.
					</p>
					{#if !$isAuthenticated}
						<button
							onclick={loginWithKeycast}
							class="px-5 py-2 bg-purple-600 hover:bg-purple-500 text-white text-sm font-semibold rounded-lg transition-colors"
						>
							Sign in with DiVine
						</button>
					{:else}
						<span class="inline-block px-4 py-2 bg-green-900/50 text-green-400 text-sm rounded-lg border border-green-800">
							You're signed in
						</span>
					{/if}
				</div>
			</div>

			<!-- Step 2 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					2
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Browse campaigns</h3>
					<p class="text-gray-300 leading-relaxed mb-3">
						Head to the <a href="/campaigns" class="text-purple-400 hover:underline">Campaigns</a> page. Each campaign is a creator (or DiVine itself) putting up Bitcoin to promote their videos. You'll see:
					</p>
					<ul class="text-gray-400 space-y-2 ml-4">
						<li class="flex items-start gap-2">
							<span class="text-purple-400 mt-1">&#x2022;</span>
							<span><strong class="text-gray-200">Payout rate</strong> — how many sats you earn per 1,000 views (e.g., 3,000 sats/1K views)</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="text-purple-400 mt-1">&#x2022;</span>
							<span><strong class="text-gray-200">Budget remaining</strong> — how much is left to earn from this campaign</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="text-purple-400 mt-1">&#x2022;</span>
							<span><strong class="text-gray-200">Target platforms</strong> — which platforms the creator wants their content shared on</span>
						</li>
					</ul>
				</div>
			</div>

			<!-- Step 3 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					3
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Download the video</h3>
					<p class="text-gray-300 leading-relaxed">
						Click into a campaign and hit <strong class="text-white">Download Source</strong>. You'll get the original DiVine loop — a clean, high-quality 6-second video ready to post anywhere. No watermarks, no overlays. The format itself is the message.
					</p>
				</div>
			</div>

			<!-- Step 4 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					4
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Post it on your platforms</h3>
					<p class="text-gray-300 leading-relaxed mb-3">
						Share the video however you want. Some ideas:
					</p>
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
						<div class="bg-gray-800 rounded-lg p-3">
							<div class="text-cyan-400 font-semibold text-sm mb-1">TikTok</div>
							<div class="text-gray-400 text-sm">Post as-is, or loop it with a caption. TikTok loves the short loop format.</div>
						</div>
						<div class="bg-gray-800 rounded-lg p-3">
							<div class="text-pink-400 font-semibold text-sm mb-1">Instagram Reels</div>
							<div class="text-gray-400 text-sm">Share as a Reel. The loop auto-plays and people watch it multiple times.</div>
						</div>
						<div class="bg-gray-800 rounded-lg p-3">
							<div class="text-red-400 font-semibold text-sm mb-1">YouTube Shorts</div>
							<div class="text-gray-400 text-sm">Upload as a Short. Great for compilations of multiple DiVine loops.</div>
						</div>
						<div class="bg-gray-800 rounded-lg p-3">
							<div class="text-gray-300 font-semibold text-sm mb-1">X (Twitter)</div>
							<div class="text-gray-400 text-sm">Post the video with a quote or reaction. The format sparks curiosity.</div>
						</div>
					</div>
					<p class="text-gray-400 text-sm mt-3">
						Use your own editing tools — CapCut, native editors, whatever you prefer. You can combine multiple DiVine loops into a compilation, add commentary, or just post them raw.
					</p>
				</div>
			</div>

			<!-- Step 5 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					5
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Submit your clip link</h3>
					<p class="text-gray-300 leading-relaxed mb-3">
						Once your video is posted, come back to DiVine Clips and <a href="/submit" class="text-purple-400 hover:underline">submit the link</a>. Paste the URL of your TikTok, Reel, Short, or X post. We'll start tracking views automatically.
					</p>
					<div class="bg-gray-800 rounded-lg p-3 text-sm text-gray-400">
						<span class="text-gray-500">Example:</span> https://tiktok.com/@yourname/video/7123456789
					</div>
				</div>
			</div>

			<!-- Step 6 -->
			<div class="flex gap-5">
				<div class="flex-shrink-0 w-12 h-12 rounded-full bg-purple-700 flex items-center justify-center text-white text-lg font-bold">
					6
				</div>
				<div class="flex-1 bg-gray-900 rounded-xl border border-gray-800 p-6">
					<h3 class="text-lg font-bold text-white mb-2">Watch your views grow — and get paid</h3>
					<p class="text-gray-300 leading-relaxed mb-3">
						We verify view counts every 6 hours. As views come in, sats accumulate in your <a href="/wallet" class="text-purple-400 hover:underline">wallet</a> automatically. Check your <a href="/dashboard" class="text-purple-400 hover:underline">dashboard</a> to track everything.
					</p>
					<p class="text-gray-300 leading-relaxed">
						When you're ready to cash out, withdraw to any Bitcoin Lightning wallet. If you don't have one yet, try <strong class="text-white">Wallet of Satoshi</strong> or <strong class="text-white">Phoenix</strong> — both are free and take seconds to set up.
					</p>
				</div>
			</div>
		</div>
	</section>

	<!-- Trust Levels -->
	<section class="mb-12 bg-gray-900 rounded-xl border border-gray-800 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">Trust Levels</h2>
		<p class="text-gray-300 leading-relaxed mb-6">
			As you clip more and build your track record, your trust level increases — unlocking higher earning limits and faster payouts.
		</p>
		<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
			<div class="bg-gray-800 rounded-lg p-5 border border-amber-900/30">
				<div class="flex items-center gap-2 mb-3">
					<div class="w-8 h-8 rounded-full bg-amber-700 flex items-center justify-center text-white text-sm font-bold">1</div>
					<span class="font-bold text-amber-500">New Clipper</span>
				</div>
				<ul class="text-gray-400 text-sm space-y-1">
					<li>Up to 50K views/week</li>
					<li>5 active clips</li>
					<li>48-hour payout hold</li>
				</ul>
			</div>
			<div class="bg-gray-800 rounded-lg p-5 border border-gray-600/30">
				<div class="flex items-center gap-2 mb-3">
					<div class="w-8 h-8 rounded-full bg-gray-500 flex items-center justify-center text-white text-sm font-bold">2</div>
					<span class="font-bold text-gray-300">Trusted Clipper</span>
				</div>
				<ul class="text-gray-400 text-sm space-y-1">
					<li>Up to 500K views/week</li>
					<li>20 active clips</li>
					<li>24-hour payout hold</li>
				</ul>
				<p class="text-gray-500 text-xs mt-2">After 100K verified views + 30 days</p>
			</div>
			<div class="bg-gray-800 rounded-lg p-5 border border-yellow-700/30">
				<div class="flex items-center gap-2 mb-3">
					<div class="w-8 h-8 rounded-full bg-yellow-600 flex items-center justify-center text-white text-sm font-bold">3</div>
					<span class="font-bold text-yellow-400">Pro Clipper</span>
				</div>
				<ul class="text-gray-400 text-sm space-y-1">
					<li>Unlimited views</li>
					<li>Unlimited clips</li>
					<li>Instant payouts</li>
				</ul>
				<p class="text-gray-500 text-xs mt-2">After 1M verified views + 90 days</p>
			</div>
		</div>
	</section>

	<!-- Tips -->
	<section class="mb-12 bg-gray-900 rounded-xl border border-gray-800 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">Tips for Great Clips</h2>
		<div class="space-y-4 text-gray-300 leading-relaxed">
			<div class="flex gap-3">
				<span class="text-purple-400 text-xl">&#x2713;</span>
				<p><strong class="text-white">Compilations perform well.</strong> String together 5-10 DiVine loops into a single TikTok or Reel. People love bingeing short loops.</p>
			</div>
			<div class="flex gap-3">
				<span class="text-purple-400 text-xl">&#x2713;</span>
				<p><strong class="text-white">Add your personality.</strong> React to the loops, add commentary, tell people why you like them. Authenticity drives views.</p>
			</div>
			<div class="flex gap-3">
				<span class="text-purple-400 text-xl">&#x2713;</span>
				<p><strong class="text-white">Post consistently.</strong> One great clip won't do as much as posting regularly. Build an audience that expects DiVine content from you.</p>
			</div>
			<div class="flex gap-3">
				<span class="text-purple-400 text-xl">&#x2713;</span>
				<p><strong class="text-white">Pick content you actually like.</strong> Your enthusiasm shows. Share loops that made you laugh, think, or want to watch again.</p>
			</div>
			<div class="flex gap-3">
				<span class="text-purple-400 text-xl">&#x2713;</span>
				<p><strong class="text-white">The format does the marketing.</strong> When people see a 6-second loop, they get curious about where it came from. You don't need to hard-sell DiVine — just share great content.</p>
			</div>
		</div>
	</section>

	<!-- Open Clipping -->
	<section class="mb-12 bg-gray-900 rounded-xl border border-gray-800 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">Clip Any DiVine Video</h2>
		<p class="text-gray-300 leading-relaxed mb-4">
			You don't need a campaign to start clipping. Every video on DiVine is eligible for sharing at a base rate of <strong class="text-yellow-400">$1.00 per 1,000 views</strong>, funded by the DiVine platform itself. Payments are made in Bitcoin (via Lightning), automatically converted at the current exchange rate.
		</p>
		<p class="text-gray-300 leading-relaxed mb-4">
			Browse <a href="https://divine.video" class="text-purple-400 hover:underline">divine.video</a>, find a loop you love, download it, post it, and submit the link. That's it. When creators set up their own campaigns with higher rates, you'll earn even more.
		</p>
		<p class="text-gray-400 text-sm">
			Think of campaigns as bonus rewards. The base rate means you can always earn by sharing content you believe in.
		</p>
	</section>

	<!-- For Creators -->
	<section class="mb-12 bg-purple-950/50 rounded-xl border border-purple-900/50 p-8">
		<h2 class="text-2xl font-bold text-white mb-4">For Creators</h2>
		<p class="text-gray-300 leading-relaxed mb-4">
			If you're a DiVine creator and want people to share your videos, you can <a href="/create" class="text-purple-400 hover:underline">create a campaign</a>. Set a budget in sats, choose a payout rate, and clippers will start spreading your content across social media.
		</p>
		<p class="text-gray-300 leading-relaxed">
			You don't need to manage anything — clippers download your videos, post them on their platforms, and earn sats for the views they generate. You just fund the campaign and watch your content spread.
		</p>
	</section>

	<!-- CTA -->
	<section class="text-center py-8">
		<a
			href="/campaigns"
			class="inline-block px-10 py-4 bg-purple-600 hover:bg-purple-500 text-white font-bold rounded-lg transition-colors text-lg shadow-lg shadow-purple-900/50"
		>
			Find Videos to Share
		</a>
	</section>
</div>

<script lang="ts">
  let { src, thumbnail, title, subtitle }: {
    src: string; thumbnail?: string; title: string; subtitle?: string;
  } = $props();

  let videoEl = $state<HTMLVideoElement | null>(null);
  let isHovering = $state(false);
  let hasLoaded = $state(false);

  function observe(node: HTMLElement) {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting && videoEl) {
          videoEl.pause();
          videoEl.removeAttribute('src');
          videoEl.load();
          hasLoaded = false;
        }
      },
      { rootMargin: '200px', threshold: 0.1 }
    );
    observer.observe(node);
    return { destroy: () => observer.disconnect() };
  }

  function startPlay() {
    isHovering = true;
    if (videoEl) {
      if (!hasLoaded) { videoEl.src = src; hasLoaded = true; }
      videoEl.play().catch(() => {});
    }
  }

  function stopPlay() {
    isHovering = false;
    if (videoEl) { videoEl.pause(); videoEl.currentTime = 0; }
  }
</script>

<div
  use:observe
  role="button"
  tabindex="0"
  onmouseenter={startPlay}
  onmouseleave={stopPlay}
  ontouchstart={startPlay}
  class="relative aspect-[9/16] bg-gray-900 rounded-xl overflow-hidden cursor-pointer group"
>
  {#if thumbnail}
    <img src={thumbnail} alt={title}
      class="absolute inset-0 w-full h-full object-cover transition-opacity duration-300"
      class:opacity-0={isHovering && hasLoaded}
      loading="lazy" />
  {/if}
  <video bind:this={videoEl} loop muted playsinline preload="none"
    class="absolute inset-0 w-full h-full object-cover"></video>
  <div class="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3">
    <p class="text-white text-sm font-medium truncate">{title}</p>
    {#if subtitle}<p class="text-gray-400 text-xs">{subtitle}</p>{/if}
  </div>
  {#if !isHovering}
    <div class="absolute inset-0 flex items-center justify-center">
      <div class="w-12 h-12 bg-black/50 rounded-full flex items-center justify-center backdrop-blur-sm">
        <span class="text-white text-lg ml-1">&#x25B6;</span>
      </div>
    </div>
  {/if}
  <div class="absolute top-2 right-2 bg-purple-600 text-white text-xs font-bold px-2 py-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity">
    Clip it
  </div>
</div>

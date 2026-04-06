<script lang="ts">
  import { getContext } from 'svelte';

  const { yScale, width } = getContext('LayerCake') as any;

  let { ticks = 4 }: { ticks?: number } = $props();

  let tickVals = $derived.by(() => {
    return $yScale.ticks ? $yScale.ticks(ticks) : $yScale.domain();
  });

  function formatTick(val: number): string {
    if (val >= 1_000_000) return (val / 1_000_000).toFixed(1) + 'M';
    if (val >= 1_000) return (val / 1_000).toFixed(0) + 'K';
    return String(val);
  }
</script>

<g class="axis axis-y">
  {#each tickVals as tick}
    <g transform="translate(0, {$yScale(tick)})">
      <line x1={0} x2={$width} stroke="#374151" stroke-dasharray="2,4" />
      <text
        x={-8}
        dy="0.32em"
        text-anchor="end"
        fill="#6b7280"
        font-size="11"
      >{formatTick(tick)}</text>
    </g>
  {/each}
</g>

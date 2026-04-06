<script lang="ts">
  import { getContext } from 'svelte';

  const { xScale, height, width } = getContext('LayerCake') as any;

  let { ticks = 4 }: { ticks?: number } = $props();

  let tickVals = $derived.by(() => {
    return $xScale.ticks ? $xScale.ticks(ticks) : $xScale.domain();
  });

  function formatTick(val: unknown): string {
    if (val instanceof Date) {
      return val.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
    if (typeof val === 'number') {
      const d = new Date(val * 1000);
      return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
    return String(val);
  }
</script>

<g class="axis axis-x">
  {#each tickVals as tick}
    <g transform="translate({$xScale(tick)}, 0)">
      <line y1={0} y2={$height} stroke="#374151" stroke-dasharray="2,4" />
      <text
        y={$height + 16}
        text-anchor="middle"
        fill="#6b7280"
        font-size="11"
      >{formatTick(tick)}</text>
    </g>
  {/each}
</g>

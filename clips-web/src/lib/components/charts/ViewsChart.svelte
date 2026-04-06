<script lang="ts">
  import { LayerCake, Svg } from 'layercake';
  import Line from './Line.svelte';
  import AxisX from './AxisX.svelte';
  import AxisY from './AxisY.svelte';

  let { data }: { data: { date: string; views: number }[] } = $props();

  // Convert date strings to Date objects for d3 scales
  let chartData = $derived(
    data.map((d) => ({ ...d, x: new Date(d.date), y: d.views }))
  );

  let hasData = $derived(data.length > 0);
</script>

<div class="w-full h-48 relative">
  {#if !hasData}
    <div class="absolute inset-0 flex items-center justify-center text-gray-600 text-sm">
      No view data yet
    </div>
  {:else}
    <LayerCake
      padding={{ top: 8, right: 16, bottom: 28, left: 36 }}
      x="x"
      y="y"
      data={chartData}
    >
      <Svg>
        <AxisX ticks={5} />
        <AxisY ticks={4} />
        <Line />
      </Svg>
    </LayerCake>
  {/if}
</div>

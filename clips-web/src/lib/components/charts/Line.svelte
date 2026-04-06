<script lang="ts">
  import { getContext } from 'svelte';
  import { line, curveMonotoneX } from 'd3-shape';

  const { data, xGet, yGet, xScale, yScale } = getContext('LayerCake') as any;

  let path = $derived.by(() => {
    const lineGen = line()
      .x((d: any) => $xGet(d))
      .y((d: any) => $yGet(d))
      .curve(curveMonotoneX);
    return lineGen($data) || '';
  });
</script>

<path
  class="path-line"
  d={path}
  fill="none"
  stroke="#a855f7"
  stroke-width="2"
  stroke-linejoin="round"
  stroke-linecap="round"
/>

<style>
  .path-line {
    stroke-dasharray: none;
  }
</style>

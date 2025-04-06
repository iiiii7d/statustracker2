<script lang="ts">
  import type { Category } from "../retrieve-data";
  import { serverData, lineColors } from "../stores";
  let percentages = $state<[Category, number][]>();
  serverData.subscribe(serverData => percentages = Array.from(serverData.y.get(0)?.entries() ?? []).map(([k, d]) => {
    return [
      k,
      Math.round(d.filter(v => !isNaN(v) && v !== 0).length / allCount * 100 * 100) / 100
    ]
  }));
  let allCount = $derived($serverData.y.get(0)?.get("all")?.filter(v => !isNaN(v)).length ?? 0);
  
</script>
<h3>% of the time that each category was online</h3>
{#each (percentages ?? []) as [k, p], i}
  <b style="color: {lineColors[i % lineColors.length]};">{k}: </b> {p}%
  {#if i !== percentages.length - 1}<b>&nbsp;|&nbsp;&nbsp;</b>{/if}
{/each}
<script lang="ts">
  import { data, lineColors } from "../stores";
  let percentages: [string, number][];
  $: allCount = $data.y.get(0)?.get("all")?.filter(v => !isNaN(v)).length ?? 0;
  $: percentages = Array.from($data.y.get(0)?.entries() ?? []).map(([k, d]) => {
    return [
      k,
      Math.round(d.filter(v => !isNaN(v) && v !== 0).length / allCount * 100 * 100) / 100
    ]
  })
</script>
<h3>% of the time that each category was online</h3>
{#each percentages as [k, p], i}
  <b style="color: {lineColors[i % lineColors.length]};">{k}: </b> {p}%
  {#if i != percentages.length - 1}<b>&nbsp;|&nbsp;&nbsp;</b>{/if}
{/each}
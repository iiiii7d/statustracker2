<script lang="ts">
  import { data, lineColors } from "./stores";
  
</script>
<h3>% of the time that each role was online</h3>
{#each Array.from($data.y.entries()) as [k, d], i}
  <b style="color: {lineColors[i % lineColors.length]};">{k}: </b> {Math.round(d.filter(v => !isNaN(v)).length / $data.x.length * 100 * 100) / 100}%
  <br>
{/each}
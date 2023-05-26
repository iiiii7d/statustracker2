<script lang="ts">
  import Chart from "./Chart.svelte";
  import Info from "./Info.svelte";
  import Input from "./Input.svelte";
  import { server } from "../retrieve-data";
  import RoleStats from "./RoleStats.svelte";
  
</script>

{#if server}
  <Chart />
  <Input />
  <hr />
  <RoleStats />
  <hr />
{/if}
<Info />
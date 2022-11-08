<script lang="ts">
  import { Chart, registerables } from 'chart.js';
  import 'chartjs-adapter-moment';
  import { onMount } from 'svelte';
  import { Line } from 'svelte-chartjs';
  import { retrievePlayerCounts } from "./retrieve-data";
  import { data } from "./stores";

  Chart.register(...registerables);
  
  onMount(retrievePlayerCounts);

  $: chartData = {
    labels: $data.x,
    datasets: [{
      label: "Players",
      data: $data.y,
      borderColor: ["#eee"]
    }]
  }
  let options: any = {
    scales: {
      x: {
        type: 'time',
        grid: {
          color: ["#555"]
        },
        time: {
          unit: "minute", 
        }
      },
      y: {
        grid: {
          color: ["#999"]
        },
        min: 0,
      },
    }
  };
</script>
<Line
  data={chartData} {options}
/>
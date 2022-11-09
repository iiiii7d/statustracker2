<script lang="ts">
  import { Chart, registerables } from 'chart.js';
  import 'chartjs-adapter-moment';
  import annotationPlugin from 'chartjs-plugin-annotation';
  import { onMount } from 'svelte';
  import { Line } from 'svelte-chartjs';
  import { retrievePlayerCounts } from "./retrieve-data";
  import { data, lineColors, playerActiveTimes } from "./stores";

  Chart.register(...registerables, annotationPlugin);
  
  onMount(retrievePlayerCounts);

  let chartData: any;
  $: chartData = {
    labels: $data.x,
    datasets: Array.from($data.y.entries()).flatMap(([k, d], i) => {
      console.error(k, d)
      return [{
        tension: .25, // TODO easter egg
        label: k,
        data: d,
        borderColor: lineColors[i % lineColors.length],
      }, {
        tension: .25, // TODO easter egg
        label: `${k} (rolling average 1h)`,
        data: d.map((datum, i) => {
          if (isNaN(datum)) return NaN;
          let slice = d.slice(Math.max(i - 30, 0), Math.min(i + 31, d.length)).filter(a => !isNaN(a));
          return slice.reduce((acc: number, dat: number) => acc + dat, 0) / slice.length
        }),
        borderColor: lineColors[i % lineColors.length]+"5",
        pointRadius: 0
      }]
    })
  }
  let options: any;
  $: options = {
    plugins: {
      annotation: {
        common: {
          drawTime: 'beforeDraw'
        },
        annotations: $playerActiveTimes.map(([from, to]) => {
          return {
            type: 'box',
            backgroundColor: '#333',
            borderWidth: 0,
            xMin: from,
            xMax: to,
            label: {
              drawTime: 'afterDatasetsDraw',
              display: false,
              content: `${from.local().format("HH:mm")} → ${to.local().format("HH:mm")}`,
              color: "#fc0",
            },
            enter({element}) {
              element.label.options.display = true;
              return true;
            },
            leave({element}) {
              element.label.options.display = false;
              return true;
            }
          }
        })
      }
    },
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

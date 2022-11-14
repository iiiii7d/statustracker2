<script lang="ts">
  import { Chart, registerables } from 'chart.js';
  import 'chartjs-adapter-moment';
  import annotationPlugin from 'chartjs-plugin-annotation';
  import { Line } from 'svelte-chartjs';
  import { data, lineColors, playerActiveTimes, rollingAverages } from "../stores";

  Chart.register(...registerables, annotationPlugin);

  function generateLine(k: string, d: number[], i: number, color: string, avgSpan?: [number, string]): any {
    return {
      tension: .25,
      label: `${k}${avgSpan ? ` (Rolling average ${avgSpan[1]})` : ""}`,
      data: avgSpan ? d.map((datum, i) => {
        if (isNaN(datum)) return NaN;
        let slice = d.slice(Math.max(i - avgSpan[0], 0), Math.min(i + avgSpan[0] + 1, d.length))
        .filter(a => !isNaN(a));
        return slice.reduce((acc: number, dat: number) => acc + dat, 0) / slice.length
      }) : d,
      borderColor: lineColors[i % lineColors.length] + color,
      pointRadius: 0,
      pointHitRadius: 5,
      spanGaps: true
    }
  }

  let chartData: any;
  $: chartData = {
    labels: $data.x,
    datasets: Array.from($data.y.entries()).flatMap(([k, d], i) => {
      let lines = [];
      const hex = "f8421";
      if ($rollingAverages[10080]) lines.push(generateLine(k, d, i, hex[lines.length], [5040, "7d"]))
      if ($rollingAverages[1440]) lines.push(generateLine(k, d, i, hex[lines.length], [720, "24h"]))
      if ($rollingAverages[720]) lines.push(generateLine(k, d, i, hex[lines.length], [360, "12h"]))
      if ($rollingAverages[60]) lines.push(generateLine(k, d, i, hex[lines.length], [30, "1h"]))
      if ($rollingAverages[0]) lines.push(generateLine(k, d, i, hex[lines.length]));
      return lines;
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
            enter({element}: any) {
              element.label.options.display = true;
              return true;
            },
            leave({element}: any) {
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

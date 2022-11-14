<script lang="ts">
  import { Chart, registerables } from 'chart.js';
  import 'chartjs-adapter-moment';
  import annotationPlugin from 'chartjs-plugin-annotation';
  import { Line } from 'svelte-chartjs';
  import { data, lineColors, playerActiveTimes } from "../stores";

  Chart.register(...registerables, annotationPlugin);

  function generateLine(k: string, d: number[], i: number, avgSpan?: [number, string, string]): any {
    return {
      tension: .25,
      label: `${k}${avgSpan ? ` (Rolling average ${avgSpan[1]})` : ""}`,
      data: avgSpan ? d.map((datum, i) => {
        if (isNaN(datum)) return NaN;
        let slice = d.slice(Math.max(i - avgSpan[0], 0), Math.min(i + avgSpan[0] + 1, d.length))
        .filter(a => !isNaN(a));
        return slice.reduce((acc: number, dat: number) => acc + dat, 0) / slice.length
      }) : d,
      borderColor: lineColors[i % lineColors.length] + (avgSpan?.at(2) ?? "1"),
      pointRadius: 0,
      pointHitRadius: 5,
      spanGaps: true
    }
  }

  let chartData: any;
  $: chartData = {
    labels: $data.x,
    datasets: Array.from($data.y.entries()).flatMap(([k, d], i) => {
      return [
        generateLine(k, d, i),
        generateLine(k, d, i, [30, "1h", "8"]),
        generateLine(k, d, i, [30*24, "1d", "f"])
      ]
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

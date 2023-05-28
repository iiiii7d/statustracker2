<script lang="ts">
  import { Chart, registerables, type ChartData, type CoreChartOptions, type DatasetChartOptions, type ElementChartOptions, type LineControllerChartOptions, type PluginChartOptions, type Point, type ScaleChartOptions, type ChartDataset } from 'chart.js';
  import type { _DeepPartialObject } from 'chart.js/dist/types/utils';
  import 'chartjs-adapter-moment';
  import annotationPlugin from 'chartjs-plugin-annotation';
  import { Line } from 'svelte-chartjs';
  import { data, lineColors, playerActiveTimes, type RollingAverage, rollingAverages } from "../stores";
  import type { Category } from '../retrieve-data';

  Chart.register(...registerables, annotationPlugin);

  const alpha = "f84210";

  function generateLine(cat: Category, y: number[], i: number, j: number, ra: RollingAverage): ChartDataset<"line", (number | Point)[]> {
    return {
      tension: .25,
      label: `${cat}${ra !== 0 ? ` (Rolling average ${rollingAverages[ra]})` : ""}`,
      data: y,
      borderColor: lineColors[j % lineColors.length] + alpha[i],
      pointRadius: 0,
      pointHitRadius: 5,
      spanGaps: true
    }
  }

  $: chartData = {
    labels: $data.x,
    datasets: Array.from($data.y.entries())
    .sort(([a, _], [b, __]) => b - a)
    .flatMap(([ra, m], i) => {
      return Array.from(m.entries()).map(([cat, y], j) => {
        return generateLine(cat, y, i, j, ra)
      })
    }),
  } as ChartData<"line", (number | Point)[], moment.Moment>;
  $: options = {
    animation: false,
    plugins: {
      annotation: {
        common: {
          drawTime: 'beforeDraw'
        },
        annotations: $playerActiveTimes.map(([from, to]) => {
          console.log(from, to);
          return {
            type: 'box',
            backgroundColor: '#333',
            borderWidth: 0,
            xMin: from as unknown as number, // prevent ts from erroring
            xMax: to as unknown as number, // prevent ts from erroring
            label: {
              drawTime: 'afterDatasetsDraw',
              display: false,
              content: `${from.local().format("HH:mm")} → ${to.local().format("HH:mm")}`,
              color: "#fc0",
            },
            enter({element}) {
              if (element.label) element.label.options.display = true;
              return true;
            },
            leave({element}) {
              if (element.label) element.label.options.display = false;
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
          // unit: "minute",
        }
      },
      y: {
        grid: {
          color: ["#999"]
        },
        min: 0,
      },
    }
  } as _DeepPartialObject<CoreChartOptions<"line"> & ElementChartOptions<"line"> & PluginChartOptions<"line"> & DatasetChartOptions<"line"> & ScaleChartOptions<"line"> & LineControllerChartOptions>;
</script>
<Line
  data={chartData} {options}
/>

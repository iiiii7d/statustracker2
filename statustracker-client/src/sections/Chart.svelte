<script lang="ts">
  import { Chart, registerables, type ChartData, type Point, type ChartDataset } from 'chart.js';
  import 'chartjs-adapter-moment';
  import annotationPlugin from 'chartjs-plugin-annotation';
  import { Line } from 'svelte-chartjs';
  import { serverData, playerActiveTimes, type RollingAverage, rollingAverages, lineColor } from "../stores";
  import type { Category } from '../retrieve-data';

  Chart.register(...registerables, annotationPlugin);

  const alpha = "f84210";

  function generateLine(cat: Category, y: number[], i: number, ra: RollingAverage): ChartDataset<"line", (number | Point)[]> {
    return {
      tension: .25,
      label: `${cat}${ra !== 0 ? ` (Rolling average ${rollingAverages[ra]})` : ""}`,
      data: y,
      borderColor: lineColor(cat) + alpha[i],
      pointRadius: 0,
      pointHitRadius: 5,
      spanGaps: ra !== 0
    }
  }

  let chartData = $state<ChartData<"line", (number | Point)[], moment.Moment>>();
  serverData.subscribe(serverData => chartData = ({
    labels: serverData.x,
    datasets: Array.from(serverData.y.entries())
      .sort(([a, _], [b, __]) => b - a)
      .flatMap(([ra, m], i) => {
        return Array.from(m.entries()).map(([cat, y]) => {
          return generateLine(cat, y, i, ra)
        })
      })}))
  let options = $state();
  playerActiveTimes.subscribe(playerActiveTimes => options = {
    animation: false,
    plugins: {
      annotation: {
        common: {
          drawTime: 'beforeDraw'
        },
        annotations: playerActiveTimes.map(([from, to]) => {
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
            enter({element}: any) {
              if (element.label) element.label.options.display = true;
              return true;
            },
            leave({element}: any) {
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
  });
</script>
<Line
  data={chartData} {options}
/>

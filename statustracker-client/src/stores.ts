import type moment from "moment";
import { writable } from "svelte/store";

export const data = writable<{
  x: moment.Moment[];
  y: Map<string, number[]>;
}>({
  x: [],
  y: new Map(),
});
export const playerActiveTimes = writable<[moment.Moment, moment.Moment][]>([]);

export const lineColors = ["#eee", "#8c0", "#8cf", "#c59", "#f80", "#088"];

export const rollingAverages = writable({
  0: true,
  60: true,
  720: false,
  1440: false,
  10080: false,
});
function totalRollingAverages(o: any) {
  return Object.values(o).filter((v) => v).length;
}

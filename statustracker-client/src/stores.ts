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

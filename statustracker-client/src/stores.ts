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

import type moment from "moment";
import { writable } from "svelte/store";
import type { Category } from "./retrieve-data";

export const data = writable<{
  x: moment.Moment[];
  y: Map<RollingAverage, Map<Category, number[]>>;
}>({
  x: [],
  y: new Map(),
});
export const playerActiveTimes = writable<[moment.Moment, moment.Moment][]>([]);

export const lineColors = [
  "#eee",
  "#8c0",
  "#8cf",
  "#c59",
  "#f80",
  "#088",
] as const;

export const rollingAverages = {
  0: "Raw",
  60: "1h",
  720: "12h",
  1440: "1d",
  10080: "7d",
} as const;

export type RollingAverage = keyof typeof rollingAverages;

export const rollingAverageSwitches = writable<Record<RollingAverage, boolean>>(
  {
    0: true,
    60: true,
    720: false,
    1440: false,
    10080: false,
  },
);

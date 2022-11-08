import type moment from "moment";
import { writable } from "svelte/store";

export const data = writable<{
  x: moment.Moment[],
  y: number[]
}>({
  x: [],
  y: []
})
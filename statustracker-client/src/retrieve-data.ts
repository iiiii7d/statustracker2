import { decode } from "@msgpack/msgpack";
import axios from "axios";
import moment from "moment";
import { data } from "./stores";

type Abs = void;
type Join = void;
type Leave = void;
type HourTimestamp = number;
type MinuteTimestamp = number;

export type Record<_T> = {
  all: number[],
  categories: Map<string, number[]>
}

export type Hour = {
  _id: HourTimestamp,
  init: Record<Abs>,
  start_min: number,
  deltas: Map<string, { joins: Record<Join>, leaves: Record<Leave> }>
}

const urlSearchParams = new URLSearchParams(window.location.search);
const server = decodeURIComponent(urlSearchParams.get("server"));

async function getMsgPack<T>(url: string): Promise<T | undefined> {
  let res = await axios
    .get<ArrayLike<number>>(url, { responseType: "arraybuffer" })
    .catch(console.error);
  if (!res) return undefined;
  return decode(res.data) as T;
}

export async function getNameMap(): Promise<string[]> {
  return await getMsgPack(`${server}/name_map`)
}

export async function getAllBetween(f: HourTimestamp, t: MinuteTimestamp): Promise<Hour[]> {
  return await getMsgPack(`${server}/?from=${f}&to=${t}`)
}

export async function retrievePlayerCounts(from: number = 0, to: number = 4294967295) {
  let x = [];
  let y = [];
  let hours = await getAllBetween(from, to);
  let current = null;
  for (let t = Math.min(...hours.map(h => h._id)); t <= Math.max(...hours.map(h => h._id)); t++) {
    let h: Hour | undefined = hours.find(h => h._id === t)
    for (let m = 0; m < 60; m++) {
      x.push(moment.unix(t * 3600 + m * 60).utc());
      if (!h) {
        y.push(current ?? 0);
      } else if (h.start_min === m) {
        current = h.init.all.length;
        y.push(current)
      } else if (current !== null) {
        current += (h.deltas[m.toString()]?.joins.all.length ?? 0) - (h.deltas[m.toString()]?.leaves.all.length ?? 0)
        y.push(current)
      } else {
        y.push(current ?? 0)
      }
    }
  }
  data.set({x, y})
}

export async function retrievePlayerData(player: string) {
  
}
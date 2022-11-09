import { decode } from "@msgpack/msgpack";
import axios from "axios";
import moment from "moment";
import { data } from "./stores";

type HourTimestamp = number;
type MinuteTimestamp = number;

export type Record =
  | {
      all: number[];
      categories: Map<string, number[]>;
    }
  | {
      joined: number[];
      joined_categories: Map<string, number[]>;
      left: number[];
      left_categories: Map<string, number[]>;
    };

export type Hour = {
  _id: HourTimestamp;
  tracked_mins: [number, number];
  deltas: Map<string, Record>;
};

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
  return await getMsgPack(`${server}/name_map`);
}

export async function getAllBetween(
  f: HourTimestamp,
  t: MinuteTimestamp,
): Promise<Hour[]> {
  return await getMsgPack(`${server}/?from=${f}&to=${t}`);
}

export async function retrievePlayerCounts(
  from: number = 0,
  to: number = 4294967295,
) {
  let x = [];
  let y = [];
  let hours = await getAllBetween(from, to);
  let current = null;
  for (
    let t = Math.min(...hours.map((h) => h._id));
    t <= Math.max(...hours.map((h) => h._id));
    t++
  ) {
    let h: Hour | undefined = hours.find((h) => h._id === t);
    console.error(h);
    for (let m = 0; m < 60; m++) {
      x.push(moment.unix(t * 3600 + m * 60).utc());
      if (
        h === undefined ||
        (m < 30
          ? h.tracked_mins[0] & (1 << m)
          : h.tracked_mins[1] & (1 << (m - 30))) === 0
      ) {
        y.push(0);
      } else {
        let record: Record | undefined = h.deltas[m.toString()];
        if (record === undefined) {
        } else if ("all" in record) {
          current = record.all.length;
        } else {
          current = current ?? 0 + record.joined.length - record.left.length;
        }
        y.push(current ?? 0);
      }
    }
  }
  data.set({ x, y });
}

export async function retrievePlayerData(player: string) {}

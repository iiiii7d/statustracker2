import { decode } from "@msgpack/msgpack";
import axios from "axios";
import moment from "moment";
import { data, playerActiveTimes } from "./stores";

type HourTimestamp = number;
type MinuteTimestamp = number;

export type Record =
  | {
      all: number[];
      categories: { [key: string]: number[] };
    }
  | {
      joined: number[];
      joined_categories: { [key: string]: number[] };
      left: number[];
      left_categories: { [key: string]: number[] };
    };

export type Hour = {
  _id: HourTimestamp;
  tracked_mins: [number, number];
  deltas: { [key: string]: Record };
};

const urlSearchParams = new URLSearchParams(window.location.search);
export const server = decodeURIComponent(urlSearchParams.get("server") ?? "");

async function getMsgPack<T>(url: string): Promise<T | undefined> {
  const res = await axios
    .get<ArrayLike<number>>(url, { responseType: "arraybuffer" })
    .catch(console.error);
  if (!res) return undefined;
  return decode(res.data) as T;
}

export async function getNameMap(): Promise<string[] | undefined> {
  return (await getMsgPack(`${server}/name_map`)) ?? undefined;
}

export async function getAllBetween(
  f: HourTimestamp,
  t: MinuteTimestamp,
): Promise<Hour[] | undefined> {
  return await getMsgPack(`${server}/?from=${f}&to=${t}`);
}

export async function getPlayerUuid(player: string): Promise<string | null> {
  return (await getMsgPack(`${server}/uuid/${player}`)) ?? null;
}

export async function retrievePlayerCounts(
  from = 0,
  to = 4294967295,
  player = "",
) {
  const x: moment.Moment[] = [];
  const y: Map<string, number[]> = new Map();
  let count = 0;
  const push = (k: string, n: number) => {
    if (!y.has(k)) {
      y.set(
        k,
        Array.from(Array(count).keys()).map((i) => {
          return isNaN(y.get("all")?.at(i) ?? NaN) ? NaN : 0;
        }),
      );
    }
    y.get(k)?.push(n);
  };
  const current: Map<string, number> = new Map();

  const hours: Hour[] | undefined = await getAllBetween(from, to);
  if (hours === undefined) {
    console.error("Error querying database for hours");
    return;
  }
  for (
    let t = Math.min(...hours.map((h) => h._id));
    t <= Math.max(...hours.map((h) => h._id));
    t++
  ) {
    const h: Hour | undefined = hours.find((h) => h._id === t);
    for (let m = 0; m < 60; m++) {
      x.push(moment.unix(t * 3600 + m * 60).utc());
      if (
        h === undefined ||
        (m < 30
          ? h.tracked_mins[0] & (1 << m)
          : h.tracked_mins[1] & (1 << (m - 30))) === 0
      ) {
        for (const cat of current.keys()) {
          push(cat, NaN);
        }
      } else {
        const record: Record | undefined = h.deltas[m.toString()];
        if (record === undefined) {
          /* empty */
        } else if ("all" in record) {
          for (const cat of current.keys()) {
            current.set(cat, 0);
          }
          current.set("all", record.all.length);
          for (const [cat, list] of Object.entries(record.categories)) {
            current.set(cat, list.length);
          }
        } else {
          current.set(
            "all",
            (current.get("all") ?? 0) +
              record.joined.length -
              record.left.length,
          );
          for (const [cat, list] of Object.entries(record.joined_categories)) {
            current.set(cat, (current.get(cat) ?? 0) + list.length);
          }
          for (const [cat, list] of Object.entries(record.left_categories)) {
            current.set(cat, (current.get(cat) ?? 0) - list.length);
          }
        }
        for (const [cat, n] of current.entries()) {
          push(cat, n);
        }
      }
      count++;
    }
  }
  data.set({ x, y });

  if (player) {
    await retrievePlayerData(hours, player);
  }
}

export async function retrievePlayerData(hours: Hour[], player: string) {
  const uuid = await getPlayerUuid(player);
  if (uuid === null) {
    console.error("No UUID found");
    playerActiveTimes.set([]);
    return;
  }
  const index = (await getNameMap())?.indexOf(uuid) ?? -1;
  const times: [moment.Moment, moment.Moment][] = [];
  let start: moment.Moment | null = null;

  const leave = (h: number, m = 0) => {
    if (start !== null) {
      times.push([start, moment.unix(h * 3600 + m * 60).utc()]);
      start = null;
    }
  };
  const join = (h: number, m = 0) => {
    if (start === null) start = moment.unix(h * 3600 + m * 60).utc();
  };

  for (
    let t = Math.min(...hours.map((h) => h._id));
    t <= Math.max(...hours.map((h) => h._id));
    t++
  ) {
    const h: Hour | undefined = hours.find((h) => h._id === t);
    if (h === undefined) {
      leave(t);
    } else
      for (let m = 0; m < 60; m++) {
        if (
          (m < 30
            ? h.tracked_mins[0] & (1 << m)
            : h.tracked_mins[1] & (1 << (m - 30))) === 0
        ) {
          leave(t, m);
        } else {
          const record: Record | undefined = h.deltas[m.toString()];
          if (record === undefined) {
            /* empty */
          } else if ("all" in record) {
            if (record.all.includes(index)) join(t, m);
          } else if (record.joined.includes(index)) {
            join(t, m);
          } else if (record.left.includes(index)) {
            leave(t, m);
          }
        }
      }
  }
  leave(4294967295);
  playerActiveTimes.set(times);
}

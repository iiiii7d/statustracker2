import { decode } from "@msgpack/msgpack";
import axios from "axios";
import moment from "moment";
import { data, playerActiveTimes } from "./stores";

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
  from: number = 0,
  to: number = 4294967295,
  player: string = "",
) {
  let x: moment.Moment[] = [];
  let y: number[] = [];
  let hours: Hour[] | undefined = await getAllBetween(from, to);
  if (hours === undefined) {
    console.error("Error querying database for hours");
    return;
  }
  let current: number | null = null;
  for (
    let t = Math.min(...hours.map((h) => h._id));
    t <= Math.max(...hours.map((h) => h._id));
    t++
  ) {
    let h: Hour | undefined = hours.find((h) => h._id === t);
    for (let m = 0; m < 60; m++) {
      x.push(moment.unix(t * 3600 + m * 60).utc());
      if (
        h === undefined ||
        (m < 30
          ? h.tracked_mins[0] & (1 << m)
          : h.tracked_mins[1] & (1 << (m - 30))) === 0
      ) {
        y.push(NaN);
      } else {
        let record: Record | undefined = h.deltas[m.toString()];
        if (record === undefined) {
        } else if ("all" in record) {
          current = record.all.length;
        } else {
          current = current ?? 0 + record.joined.length - record.left.length;
        }
        y.push(current ?? NaN);
      }
    }
  }
  data.set({ x, y });

  if (player) {
    await retrievePlayerData(hours, player);
  }
}

export async function retrievePlayerData(hours: Hour[], player: string) {
  let uuid = await getPlayerUuid(player);
  if (uuid === null) {
    console.error("No UUID found");
    playerActiveTimes.set([]);
    return;
  }
  let index = (await getNameMap())?.indexOf(uuid) ?? -1;
  let times: [moment.Moment, moment.Moment][] = [];
  let start: moment.Moment | null = null;

  let leave = (h: number, m: number = 0) => {
    if (start !== null) {
      times.push([start, moment.unix(h * 3600 + m * 60).utc()]);
      start = null;
    }
  };
  let join = (h: number, m: number = 0) => {
    start = moment.unix(h * 3600 + m * 60).utc();
  };

  for (
    let t = Math.min(...hours.map((h) => h._id));
    t <= Math.max(...hours.map((h) => h._id));
    t++
  ) {
    let h: Hour | undefined = hours.find((h) => h._id === t);
    if (h === undefined && start !== null) {
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
          let record: Record | undefined = h.deltas[m.toString()];
          if (record === undefined) {
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

import { decode } from "@msgpack/msgpack";
import axios from "axios";
import moment from "moment";
import { type RollingAverage } from "./stores";

export interface RollingAvgRecord {
  all: number;
  categories: Record<Category, number>;
}

export type MinuteTimestamp = number;
export type Category = string;

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

export async function getLine(
  f: MinuteTimestamp,
  t: MinuteTimestamp,
  range: RollingAverage = 0,
): Promise<(RollingAvgRecord | null)[] | undefined> {
  return await getMsgPack(`${server}/?from=${f}&to=${t}&range=${range / 2}`);
}

export async function getPlayerUuid(player: string): Promise<string | null> {
  return (await getMsgPack(`${server}/uuid/${player}`)) ?? null;
}

export async function getPlayerJoinTimes(
  from: number,
  to: number,
  player: string,
): Promise<[moment.Moment, moment.Moment][]> {
  const a =
    (await getMsgPack<[MinuteTimestamp, MinuteTimestamp][]>(
      `${server}/player/${player}?from=${from}&to=${to}`,
    )) ?? [];
  return a.map(([a, b]) => [moment.unix(a * 60), moment.unix(b * 60)]);
}

export async function getLines(
  from: number,
  to: number,
  rollingAverages: RollingAverage[] = [0],
): Promise<{
  x: moment.Moment[];
  y: Map<RollingAverage, Map<Category, number[]>>;
}> {
  const p = await Promise.all(
    rollingAverages.map((ra) => getLine(from, to, ra)),
  );
  const y = new Map<RollingAverage, Map<Category, number[]>>();
  for (const [i, r] of p.entries()) {
    if (r === undefined) continue;
    if (!y.has(rollingAverages[i])) y.set(rollingAverages[i], new Map());
    const m = y.get(rollingAverages[i])!;

    m.set(
      "all",
      r.map((s) => s?.all ?? NaN),
    );
    const categories = new Set(
      r.flatMap((s) => Object.keys(s?.categories ?? {})),
    );
    for (const cat of categories) {
      m.set(
        cat,
        r.map((s) => s?.categories[cat] ?? NaN),
      );
    }
  }
  const x = [];
  for (let i = from; i <= to; i++) {
    x.push(moment.unix(i * 60));
  }
  return { x, y };
}
